//! Professional text shaping system with FontIR integration
//!
//! This module provides the architecture for real-time font compilation and 
//! HarfBuzz text shaping, similar to how professional font editors like Glyphs work.
//! 
//! Currently uses simplified Arabic shaping with the FontIR → font binary pipeline
//! ready for full HarfBuzz integration.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::{SortLayoutMode, TextEditorState};
use crate::systems::arabic_shaping::{shape_arabic_text, ArabicShapingCache};
use crate::systems::text_shaping::{ShapedText, TextDirection};
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

/// Resource for managing font compilation and shaping cache
#[derive(Resource)]
pub struct ProfessionalShapingCache {
    /// Temporary directory for compiled fonts
    temp_dir: Option<TempDir>,
    /// Path to last compiled font binary
    compiled_font_path: Option<PathBuf>,
    /// Last compilation timestamp for cache invalidation
    last_compiled: Option<std::time::Instant>,
    /// Whether HarfBuzz is available for use (future feature)
    #[allow(dead_code)]
    harfbuzz_available: bool,
    /// Cache of shaped text results
    shaped_cache: HashMap<String, ShapedText>,
}

impl Default for ProfessionalShapingCache {
    fn default() -> Self {
        Self {
            temp_dir: None,
            compiled_font_path: None,
            last_compiled: None,
            harfbuzz_available: false, // Will be set to true when HarfBuzz works
            shaped_cache: HashMap::new(),
        }
    }
}

/// Compile font from FontIR using fontc for professional shaping
pub fn compile_font_for_shaping(
    fontir_state: &FontIRAppState,
    cache: &mut ProfessionalShapingCache,
) -> Result<Vec<u8>, String> {
    // Check if we already have compiled font bytes
    if let Some(path) = &cache.compiled_font_path {
        if path.exists() {
            return std::fs::read(path).map_err(|e| e.to_string());
        }
    }
    
    info!("Compiling font for professional text shaping using fontc");
    
    // Create fontc input from the source path
    let input = fontc::Input::new(&fontir_state.source_path)
        .map_err(|e| format!("Failed to create fontc input: {}", e))?;
    
    // Create temp directory for build
    if cache.temp_dir.is_none() {
        cache.temp_dir = Some(TempDir::new().map_err(|e| e.to_string())?);
    }
    
    let temp_dir = cache.temp_dir.as_ref().unwrap();
    let build_dir = temp_dir.path().join("build");
    std::fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;
    
    // Use fontc to compile the font (same as file menu export)
    let flags = fontc::Flags::default();
    
    match fontc::generate_font(
        &input,
        &build_dir,
        None, // No specific output path
        flags,
        false, // Not watching for changes
    ) {
        Ok(font_bytes) => {
            // Cache the compiled font for future use
            let output_path = build_dir.join("font.ttf");
            if let Err(e) = std::fs::write(&output_path, &font_bytes) {
                warn!("Failed to cache compiled font: {}", e);
            } else {
                cache.compiled_font_path = Some(output_path);
            }
            
            cache.last_compiled = Some(std::time::Instant::now());
            info!("✅ Font compiled successfully with fontc for text shaping");
            Ok(font_bytes)
        }
        Err(e) => {
            error!("fontc compilation failed: {}", e);
            Err(format!("fontc compilation failed: {}", e))
        }
    }
}

/// Shape text using the best available method
pub fn shape_text_professionally(
    text: &str,
    direction: TextDirection,
    cache: &mut ProfessionalShapingCache,
    fontir_state: &FontIRAppState,
) -> Result<ShapedText, String> {
    // Check cache first
    let cache_key = format!("{}_{:?}", text, direction);
    if let Some(cached) = cache.shaped_cache.get(&cache_key) {
        return Ok(cached.clone());
    }
    
    // Try to compile font with fontc for shaping
    match compile_font_for_shaping(fontir_state, cache) {
        Ok(font_bytes) => {
            info!("Font compiled successfully with fontc ({} bytes)", font_bytes.len());
            
            // TODO: When HarfBuzz integration is working, use font_bytes here:
            // let hb_font = HBFont::from_data(&font_bytes, 0)?;
            // ... actual HarfBuzz shaping ...
            
            // For now, fall back to our Arabic shaping but mark as professionally compiled
            let result = shape_arabic_text(text, direction, fontir_state)?;
            cache.shaped_cache.insert(cache_key, result.clone());
            Ok(result)
        }
        Err(e) => {
            // Use our simplified Arabic shaping as fallback
            warn!("Font compilation failed: {}", e);
            let result = shape_arabic_text(text, direction, fontir_state)?;
            cache.shaped_cache.insert(cache_key, result.clone());
            Ok(result)
        }
    }
}

/// System for professional text shaping with font compilation
pub fn professional_shaping_system(
    mut text_editor_state: ResMut<TextEditorState>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut prof_cache: ResMut<ProfessionalShapingCache>,
    _arabic_cache: ResMut<ArabicShapingCache>,
) {
    let Some(fontir_state) = fontir_state else {
        return;
    };
    
    // Detect if we have text that needs complex shaping
    let needs_shaping = text_editor_state.buffer.iter().any(|entry| {
        if let crate::core::state::text_editor::buffer::SortKind::Glyph { codepoint: Some(ch), .. } = &entry.kind {
            let code = *ch as u32;
            // Arabic, Hebrew, or other complex scripts
            (0x0600..=0x06FF).contains(&code) || 
            (0x0590..=0x05FF).contains(&code) ||
            (0x0900..=0x097F).contains(&code)  // Devanagari
        } else {
            false
        }
    });
    
    if !needs_shaping {
        return;
    }
    
    // Collect text runs for shaping
    let mut text_runs = Vec::new();
    let mut current_run = String::new();
    let mut run_indices = Vec::new();
    let mut run_direction = TextDirection::LeftToRight;
    
    for (i, entry) in text_editor_state.buffer.iter().enumerate() {
        match &entry.kind {
            crate::core::state::text_editor::buffer::SortKind::Glyph { codepoint: Some(ch), .. } => {
                if current_run.is_empty() {
                    run_direction = match entry.layout_mode {
                        SortLayoutMode::RTLText => TextDirection::RightToLeft,
                        _ => TextDirection::LeftToRight,
                    };
                }
                current_run.push(*ch);
                run_indices.push(i);
            }
            crate::core::state::text_editor::buffer::SortKind::LineBreak => {
                if !current_run.is_empty() {
                    text_runs.push((current_run.clone(), run_indices.clone(), run_direction));
                    current_run.clear();
                    run_indices.clear();
                }
            }
            _ => {}
        }
    }
    
    // Process the last run
    if !current_run.is_empty() {
        text_runs.push((current_run, run_indices, run_direction));
    }
    
    // Shape each text run
    for (text, indices, direction) in text_runs {
        match shape_text_professionally(&text, direction, &mut prof_cache, &fontir_state) {
            Ok(shaped) => {
                // Update buffer with shaped results
                for (buffer_idx, shaped_glyph) in indices.iter().zip(shaped.shaped_glyphs.iter()) {
                    if let Some(entry) = text_editor_state.buffer.get_mut(*buffer_idx) {
                        if let crate::core::state::text_editor::buffer::SortKind::Glyph { 
                            glyph_name, advance_width, .. 
                        } = &mut entry.kind {
                            *glyph_name = shaped_glyph.glyph_name.clone();
                            *advance_width = shaped_glyph.advance_width;
                        }
                    }
                }
                
                info!("Professionally shaped text: '{}' → {} glyphs", text, shaped.shaped_glyphs.len());
            }
            Err(e) => {
                warn!("Professional shaping failed: {}", e);
            }
        }
    }
}

/// Plugin for professional text shaping with FontIR integration
pub struct ProfessionalShapingPlugin;

impl Plugin for ProfessionalShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProfessionalShapingCache>()
            .add_systems(Update, professional_shaping_system);
    }
}