//! HarfBuzz text shaping system with FontIR integration
//!
//! This module provides real-time HarfBuzz text shaping for complex scripts like Arabic.
//! It compiles fonts from FontIR using fontc and performs professional text shaping.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::{SortLayoutMode, TextEditorState};
use crate::systems::text_shaping::{ShapedGlyph, ShapedText, TextDirection};
use bevy::prelude::*;
use harfrust::{Direction, FontRef, GlyphBuffer, Language, Script, ShaperData, ShaperInstance, UnicodeBuffer};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;

impl From<TextDirection> for Direction {
    fn from(direction: TextDirection) -> Self {
        match direction {
            TextDirection::LeftToRight => Direction::LeftToRight,
            TextDirection::RightToLeft => Direction::RightToLeft,
            TextDirection::TopToBottom => Direction::TopToBottom,
            TextDirection::BottomToTop => Direction::BottomToTop,
        }
    }
}

/// Resource for managing font compilation and HarfBuzz shaping cache
#[derive(Resource)]
pub struct HarfBuzzShapingCache {
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

impl Default for HarfBuzzShapingCache {
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

/// Compile font from FontIR using fontc for HarfBuzz shaping
pub fn compile_font_for_shaping(
    fontir_state: &FontIRAppState,
    cache: &mut HarfBuzzShapingCache,
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

/// Shape text using HarfBuzz with compiled font
pub fn shape_text_with_harfbuzz(
    text: &str,
    direction: TextDirection,
    cache: &mut HarfBuzzShapingCache,
    fontir_state: &FontIRAppState,
) -> Result<ShapedText, String> {
    // Check cache first
    let cache_key = format!("{}_{:?}", text, direction);
    if let Some(cached) = cache.shaped_cache.get(&cache_key) {
        return Ok(cached.clone());
    }
    
    // Compile font with fontc for HarfBuzz shaping
    let font_bytes = compile_font_for_shaping(fontir_state, cache)?;
    info!("Font compiled for HarfBuzz ({} bytes)", font_bytes.len());
    
    // Shape text with HarfBuzz
    let result = perform_harfbuzz_shaping(text, direction, &font_bytes, fontir_state)?;
    
    // Cache the result
    cache.shaped_cache.insert(cache_key, result.clone());
    Ok(result)
}

/// Perform actual HarfBuzz text shaping using harfrust
fn perform_harfbuzz_shaping(
    text: &str,
    direction: TextDirection,
    font_bytes: &[u8],
    fontir_state: &FontIRAppState,
) -> Result<ShapedText, String> {
    // Create harfrust font from compiled font bytes
    let font_ref = FontRef::from_index(font_bytes, 0)
        .map_err(|e| format!("Failed to create harfrust FontRef: {:?}", e))?;
    
    // Create shaper data and instance
    let shaper_data = ShaperData::new(&font_ref);
    let shaper_instance = ShaperInstance::from_variations(&font_ref, &[] as &[harfrust::Variation]);
    let shaper = shaper_data
        .shaper(&font_ref)
        .instance(Some(&shaper_instance))
        .build();
    
    // Create buffer and add text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    
    // Set buffer properties
    buffer.set_direction(direction.into());
    buffer.set_script(get_script_for_text(text));
    buffer.set_language(Language::from_str("ar").unwrap_or(Language::from_str("en").unwrap()));
    
    // Guess remaining properties automatically
    buffer.guess_segment_properties();
    
    // Perform HarfBuzz shaping
    let glyph_buffer = shaper.shape(buffer, &[]);
    
    // Extract shaped glyph information
    let input_codepoints: Vec<char> = text.chars().collect();
    let mut shaped_glyphs = Vec::new();
    
    let glyph_infos = glyph_buffer.glyph_infos();
    let glyph_positions = glyph_buffer.glyph_positions();
    
    for (i, glyph_info) in glyph_infos.iter().enumerate() {
        // Get glyph name from glyph ID
        let glyph_name = get_glyph_name_from_id(glyph_info.glyph_id, fontir_state);
        
        // Get original codepoint from cluster index
        let codepoint = if (glyph_info.cluster as usize) < input_codepoints.len() {
            input_codepoints[glyph_info.cluster as usize]
        } else {
            '\u{FFFD}' // Replacement character
        };
        
        // Get glyph position info
        let pos = glyph_positions.get(i).cloned().unwrap_or_default();
        
        // harfrust uses units per em directly (no scaling needed)
        let advance_width = pos.x_advance as f32;
        
        shaped_glyphs.push(ShapedGlyph {
            glyph_id: glyph_info.glyph_id,
            codepoint,
            glyph_name,
            advance_width,
            x_offset: pos.x_offset as f32,
            y_offset: pos.y_offset as f32,
            cluster: glyph_info.cluster,
        });
    }
    
    info!("HarfBuzz shaped '{}' into {} glyphs", text, shaped_glyphs.len());
    
    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: true,
    })
}

/// Get glyph name from glyph ID using FontIR
fn get_glyph_name_from_id(glyph_id: u32, fontir_state: &FontIRAppState) -> String {
    // CRITICAL FIX: We need to properly map HarfBuzz glyph IDs to FontIR glyph names
    // For now, as a working solution, let HarfBuzz handle the shaping logic
    // and use a simpler approach based on the actual font structure
    
    // TODO: Implement proper glyph ID mapping from the compiled font
    // For now, we should use the original codepoint-based approach
    // since the glyph ID mapping is complex and font-specific
    
    // Fallback: use glyph ID directly - this will be improved later
    // The real solution needs font's cmap and post table parsing
    format!("gid{}", glyph_id)
}

/// Get appropriate script for HarfBuzz based on text content
fn get_script_for_text(text: &str) -> Script {
    use harfrust::script;
    
    if text.chars().any(|ch| {
        let code = ch as u32;
        (0x0600..=0x06FF).contains(&code) // Arabic block
    }) {
        script::ARABIC
    } else {
        script::LATIN
    }
}

/// System for HarfBuzz text shaping with font compilation
pub fn harfbuzz_shaping_system(
    mut text_editor_state: ResMut<TextEditorState>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut hb_cache: ResMut<HarfBuzzShapingCache>,
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
    
    info!("🔤 HarfBuzz: Detected text that needs complex shaping!");
    
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
        info!("🔤 HarfBuzz: Attempting to shape text '{}' with direction {:?}", text, direction);
        match shape_text_with_harfbuzz(&text, direction, &mut hb_cache, &fontir_state) {
            Ok(shaped) => {
                info!("🔤 HarfBuzz: Successfully shaped '{}' into {} glyphs", text, shaped.shaped_glyphs.len());
                // Update buffer with shaped results
                for (buffer_idx, shaped_glyph) in indices.iter().zip(shaped.shaped_glyphs.iter()) {
                    if let Some(entry) = text_editor_state.buffer.get_mut(*buffer_idx) {
                        if let crate::core::state::text_editor::buffer::SortKind::Glyph { 
                            glyph_name, advance_width, .. 
                        } = &mut entry.kind {
                            let old_name = glyph_name.clone();
                            *glyph_name = shaped_glyph.glyph_name.clone();
                            *advance_width = shaped_glyph.advance_width;
                            info!("🔤 HarfBuzz: Updated glyph U+{:04X} from '{}' to '{}'", 
                                  shaped_glyph.codepoint as u32, old_name, shaped_glyph.glyph_name);
                        }
                    }
                }
                
                info!("🔤 HarfBuzz: Professionally shaped text: '{}' → {} glyphs", text, shaped.shaped_glyphs.len());
            }
            Err(e) => {
                error!("🔤 HarfBuzz: Professional shaping failed for '{}': {}", text, e);
            }
        }
    }
}

/// Plugin for HarfBuzz text shaping with FontIR integration
pub struct HarfBuzzShapingPlugin;

impl Plugin for HarfBuzzShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HarfBuzzShapingCache>()
            .add_systems(Update, harfbuzz_shaping_system.in_set(crate::editing::FontEditorSets::TextBuffer));
    }
}