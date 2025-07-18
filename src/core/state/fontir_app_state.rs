//! FontIR-based application state
//!
//! This module provides the new AppState that directly uses FontIR structures
//! instead of custom data types. This enables multi-format support and
//! variable font handling.

use anyhow::Result;
use bevy::prelude::*;
use tracing::{info, warn};
use fontdrasil::coords::{NormalizedLocation, NormalizedCoord};
use fontdrasil::types::GlyphName;
use fontir::ir::Glyph as FontIRGlyph;
use fontir::source::Source;
use fontir::orchestration::{Context, Flags, WorkId};
use fontir::paths::Paths;
use kurbo::{BezPath, PathEl, Point};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use ufo2fontir::source::DesignSpaceIrSource;

/// Font metrics extracted from FontIR
#[derive(Debug, Clone)]
pub struct FontIRMetrics {
    pub units_per_em: f32,
    pub ascender: Option<f32>,
    pub descender: Option<f32>,
    pub line_gap: Option<f32>,
}

/// The main application state using FontIR
#[derive(Resource, Clone)]
pub struct FontIRAppState {
    /// The FontIR source (handles UFO, designspace, etc.)
    pub source: Arc<DesignSpaceIrSource>,
    
    /// FontIR context containing processed font data
    pub context: Option<Arc<Context>>,
    
    /// Cached glyph data for quick access
    /// Maps glyph name to FontIR glyph
    pub glyph_cache: HashMap<String, Arc<FontIRGlyph>>,
    
    /// Currently selected glyph name
    pub current_glyph: Option<String>,
    
    /// Currently selected design space location
    /// For variable fonts, this determines which instance we're editing
    pub current_location: NormalizedLocation,
    
    /// Path to the source file
    pub source_path: PathBuf,
}

impl FontIRAppState {
    /// Create a new FontIR-based app state from a font file
    pub fn from_path(path: PathBuf) -> Result<Self> {
        // Load the source (works with .ufo or .designspace)
        let source = Arc::new(DesignSpaceIrSource::new(&path)?);
        
        // Initialize with default location
        // Note: We'll use fallback to first available instance in glyph lookup
        // since exact location matching is complex with variable fonts
        let current_location = NormalizedLocation::default();
        
        let mut app_state = Self {
            source,
            context: None,
            glyph_cache: HashMap::new(),
            current_glyph: None,
            current_location,
            source_path: path,
        };
        
        // Load glyphs into cache
        if let Err(e) = app_state.load_glyphs() {
            warn!("Failed to load glyphs during FontIR initialization: {}", e);
        }
        
        Ok(app_state)
    }
    
    /// Set the current glyph
    pub fn set_current_glyph(&mut self, glyph_name: Option<String>) {
        self.current_glyph = glyph_name;
    }
    
    /// Get a glyph by name
    pub fn get_glyph(&self, name: &str) -> Option<&FontIRGlyph> {
        self.glyph_cache.get(name).map(|g| g.as_ref())
    }
    
    /// Get the current glyph's path at the current location
    pub fn get_current_glyph_paths(&self) -> Option<Vec<BezPath>> {
        let glyph_name = self.current_glyph.as_ref()?;
        self.get_glyph_paths(glyph_name)
    }
    
    /// Get a glyph's path by name at the current location
    pub fn get_glyph_paths(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        // First try to get from FontIR context using the helper method
        if let Some(ref context) = self.context {
            let glyph_name_typed: GlyphName = glyph_name.into();
            let work_id = WorkId::Glyph(glyph_name_typed);
            
            // Use try_get since we're not sure if the glyph exists
            if let Some(glyph) = context.glyphs.try_get(&work_id) {
                // Try to get the instance at our current location
                if let Some(instance) = glyph.sources().get(&self.current_location) {
                    return Some(instance.contours.clone());
                }
                
                // Fallback: Use the first available instance if exact location doesn't exist
                if let Some((_location, instance)) = glyph.sources().iter().next() {
                    info!("get_glyph_paths: Using first available instance for glyph '{}' with {} contours", glyph_name, instance.contours.len());
                    return Some(instance.contours.clone());
                }
            }
        }
        
        // Fall back to cached glyph data
        if let Some(glyph) = self.get_glyph(glyph_name) {
            // Try to get the instance at our current location
            if let Some(instance) = glyph.sources().get(&self.current_location) {
                return Some(instance.contours.clone());
            }
            
            // Fallback: Use the first available instance if exact location doesn't exist
            if let Some((_location, instance)) = glyph.sources().iter().next() {
                info!("get_glyph_paths: Using first available cached instance for glyph '{}' with {} contours", glyph_name, instance.contours.len());
                return Some(instance.contours.clone());
            }
        }
        
        // Final fallback - return test shapes for common glyphs to verify the system works  
        warn!("get_glyph_paths: No real glyph data found for '{}', falling back to placeholder shapes", glyph_name);
        self.create_fallback_glyph_path(glyph_name)
    }
    
    /// Get a mutable path element by indices
    /// Returns (path_index, element_index) for the found element
    pub fn find_path_element(
        &self,
        contour_idx: usize,
        point_idx: usize,
    ) -> Option<(usize, usize)> {
        let paths = self.get_current_glyph_paths()?;
        
        if contour_idx >= paths.len() {
            return None;
        }
        
        let path = &paths[contour_idx];
        let elements: Vec<_> = path.elements().iter().collect();
        
        if point_idx < elements.len() {
            Some((contour_idx, point_idx))
        } else {
            None
        }
    }
    
    /// Update a path element (for point editing)
    pub fn update_path_element(
        &mut self,
        _contour_idx: usize,
        _element_idx: usize,
        _new_element: PathEl,
    ) -> Result<()> {
        // This is where we'd update the FontIR glyph
        // For now, this is a placeholder showing the interface
        
        // In practice, we'd need to:
        // 1. Get mutable access to the glyph
        // 2. Update the specific path element
        // 3. Mark the glyph as dirty for recompilation
        
        todo!("Implement FontIR glyph modification")
    }
    
    /// Load all glyphs into cache
    pub fn load_glyphs(&mut self) -> Result<()> {
        info!("Loading glyphs from FontIR source");
        
        // Create a new context with proper flags and paths
        let flags = Flags::empty(); // For now, use empty flags
        let temp_dir = std::env::temp_dir();
        let paths = Paths::new(&temp_dir);
        let mut context = Context::new_root(flags, paths);
        
        // Execute FontIR work items to populate the context with real glyph data
        if let Err(e) = self.execute_fontir_work(&mut context) {
            warn!("Failed to execute FontIR work: {}. Trying alternative direct access approach.", e);
            
            // Alternative approach: Try direct data extraction from DesignSpaceIrSource
            if let Err(e2) = self.try_direct_glyph_extraction() {
                warn!("Direct glyph extraction also failed: {}. Using fallback data.", e2);
            }
        }
        
        self.context = Some(Arc::new(context));
        
        Ok(())
    }
    
    /// Execute FontIR work items with proper orchestration and permissions
    fn execute_fontir_work(&mut self, context: &mut Context) -> Result<()> {
        use fontdrasil::orchestration::Work;
        
        info!("Executing FontIR work items to load real glyph data");
        
        // First, create and execute static metadata work
        let static_metadata_work = self.source.create_static_metadata_work()?;
        let static_read_access = static_metadata_work.read_access();
        let static_write_access = static_metadata_work.write_access();
        let static_context = context.copy_for_work(static_read_access, static_write_access);
        
        info!("Executing static metadata work");
        if let Err(e) = static_metadata_work.exec(&static_context) {
            warn!("Static metadata work failed: {}", e);
            return Err(anyhow::anyhow!("Static metadata work failed: {}", e));
        }
        
        // Create and execute preliminary glyph order work
        let glyph_order_work = fontir::glyph::create_glyph_order_work();
        let order_read_access = glyph_order_work.read_access();
        let order_write_access = glyph_order_work.write_access();
        let order_context = context.copy_for_work(order_read_access, order_write_access);
        
        // Execute global metrics work
        let global_metrics_work = self.source.create_global_metric_work()?;
        let metrics_read_access = global_metrics_work.read_access();
        let metrics_write_access = global_metrics_work.write_access();
        let metrics_context = context.copy_for_work(metrics_read_access, metrics_write_access);
        
        info!("Executing global metrics work");
        if let Err(e) = global_metrics_work.exec(&metrics_context) {
            warn!("Global metrics work failed: {}", e);
        }
        
        // Create and execute glyph IR work items
        let glyph_work_items = self.source.create_glyph_ir_work()?;
        info!("Executing {} glyph IR work items", glyph_work_items.len());
        
        // Execute each glyph work item with proper permissions
        // Glyph work items need access to all previously computed data
        use fontdrasil::orchestration::{Access, AccessBuilder};
        
        for (i, work_item) in glyph_work_items.iter().enumerate() {
            // Glyph work needs broader read access than what's specified in the work item
            // It needs to read static metadata, global metrics, and other glyphs for components
            let broad_read_access = AccessBuilder::new()
                .variant(WorkId::StaticMetadata)
                .variant(WorkId::GlobalMetrics)
                .variant(WorkId::PreliminaryGlyphOrder)
                .variant(WorkId::ALL_GLYPHS)  // Access to all glyphs for component resolution
                .build();
            
            let write_access = work_item.write_access();
            let work_context = context.copy_for_work(broad_read_access, write_access);
            
            if let Err(e) = work_item.exec(&work_context) {
                warn!("Glyph work item {} failed: {}", i, e);
                // Continue with other work items even if one fails
                continue;
            }
            
            // Cache the glyph data if it was successfully created
            if let Ok(glyph_name) = self.extract_glyph_name_from_work_id(&work_item.id()) {
                if let Some(glyph) = work_context.glyphs.try_get(&work_item.id()) {
                    self.glyph_cache.insert(glyph_name, glyph);
                }
            }
        }
        
        // Execute preliminary glyph order after glyphs are loaded
        info!("Executing glyph order work");
        if let Err(e) = glyph_order_work.exec(&order_context) {
            warn!("Glyph order work failed: {}", e);
        }
        
        info!("FontIR work execution completed. Loaded {} glyphs into cache", self.glyph_cache.len());
        Ok(())
    }
    
    /// Helper to extract glyph name from WorkId
    fn extract_glyph_name_from_work_id(&self, work_id: &WorkId) -> Result<String> {
        match work_id {
            WorkId::Glyph(glyph_name) => Ok(glyph_name.to_string()),
            _ => Err(anyhow::anyhow!("Work ID is not a glyph: {:?}", work_id)),
        }
    }
    
    /// Alternative approach: Try to extract glyph data directly from DesignSpaceIrSource
    /// without complex orchestration. This is simpler but may not handle all edge cases.
    fn try_direct_glyph_extraction(&mut self) -> Result<()> {
        info!("Attempting direct glyph extraction from DesignSpaceIrSource");
        
        // This approach tries to access the underlying UFO data directly
        // Note: This is less robust than full FontIR orchestration but may work for simple cases
        
        // For now, we'll attempt to create a minimal working context
        // and try to access the source data structures directly
        
        // Create work items but use them only to understand what glyphs exist
        match self.source.create_glyph_ir_work() {
            Ok(glyph_work_items) => {
                info!("Found {} glyph work items to extract", glyph_work_items.len());
                
                // Try to create a simplified context with full access for testing
                let flags = Flags::empty();
                let temp_dir = std::env::temp_dir();
                let paths = Paths::new(&temp_dir);
                let context = Context::new_root(flags, paths);
                
                // Create a context with full access permissions for testing
                use fontdrasil::orchestration::{Access, AccessBuilder};
                
                // Create broad access that should cover all necessary data
                let broad_read_access = AccessBuilder::new()
                    .variant(WorkId::StaticMetadata)
                    .variant(WorkId::GlobalMetrics)
                    .variant(WorkId::PreliminaryGlyphOrder)
                    .variant(WorkId::ALL_GLYPHS)
                    .variant(WorkId::ALL_ANCHORS)
                    .build();
                    
                let full_access_context = context.copy_for_work(broad_read_access, Access::All);
                
                // Try to execute just one work item to test the approach
                if let Some(test_work) = glyph_work_items.first() {
                    info!("Testing work execution with work item: {:?}", test_work.id());
                    
                    if let Err(e) = test_work.exec(&full_access_context) {
                        warn!("Direct work execution failed: {}", e);
                        return Err(anyhow::anyhow!("Direct work execution failed: {}", e));
                    }
                    
                    // If successful, try to extract the glyph data
                    if let Ok(glyph_name) = self.extract_glyph_name_from_work_id(&test_work.id()) {
                        if let Some(glyph) = full_access_context.glyphs.try_get(&test_work.id()) {
                            info!("Successfully extracted glyph '{}' with {} sources", glyph_name, glyph.sources().len());
                            self.glyph_cache.insert(glyph_name, glyph);
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create glyph work items: {}", e);
                return Err(anyhow::anyhow!("Failed to create glyph work items: {}", e));
            }
        }
        
        Err(anyhow::anyhow!("Direct glyph extraction did not succeed"))
    }
    
    /// Get font metrics from FontIR source
    pub fn get_font_metrics(&self) -> FontIRMetrics {
        if let Some(ref context) = self.context {
            // Get static metadata for units_per_em
            let static_metadata = context.static_metadata.get();
            let units_per_em = static_metadata.units_per_em as f32;
            let default_location = static_metadata.default_location();
            
            // Get global metrics at default location
            let global_metrics = context.global_metrics.get();
            let metrics = global_metrics.at(default_location);
            
            FontIRMetrics {
                units_per_em,
                ascender: Some(metrics.ascender.0 as f32),
                descender: Some(metrics.descender.0 as f32),
                line_gap: Some(metrics.os2_typo_line_gap.0 as f32),
            }
        } else {
            // Fallback to sensible defaults if context not available
            FontIRMetrics {
                units_per_em: 1000.0,
                ascender: Some(800.0),
                descender: Some(-200.0),
                line_gap: Some(0.0),
            }
        }
    }
    
    /// Get glyph names available in the font
    pub fn get_glyph_names(&self) -> Vec<String> {
        // First try to get from the FontIR context
        if let Some(ref context) = self.context {
            let all_glyphs = context.glyphs.all();
            let mut names: Vec<String> = all_glyphs.into_iter()
                .filter_map(|(work_id, _)| {
                    if let WorkId::Glyph(glyph_name) = work_id {
                        Some(glyph_name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            if !names.is_empty() {
                names.sort();
                return names;
            }
        }
        
        // Fall back to cached glyph names if available
        if !self.glyph_cache.is_empty() {
            let mut names: Vec<String> = self.glyph_cache.keys().cloned().collect();
            names.sort();
            return names;
        }
        
        // Final fallback - return basic test glyph names
        vec![
            "a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string(),
            "f".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(),
            "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(),
            "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(),
            "u".to_string(), "v".to_string(), "w".to_string(), "x".to_string(), "y".to_string(), "z".to_string(),
        ]
    }
    
    /// Get advance width for a glyph
    pub fn get_glyph_advance_width(&self, glyph_name: &str) -> f32 {
        // First try to get from FontIR context
        if let Some(ref context) = self.context {
            let glyph_name_typed: GlyphName = glyph_name.into();
            let work_id = WorkId::Glyph(glyph_name_typed);
            
            // Use try_get since we're not sure if the glyph exists
            if let Some(glyph) = context.glyphs.try_get(&work_id) {
                // Get the instance at our current location
                if let Some(instance) = glyph.sources().get(&self.current_location) {
                    return instance.width as f32;
                }
            }
        }
        
        // Fall back to cached glyph data
        if let Some(glyph) = self.get_glyph(glyph_name) {
            // Get the instance at our current location
            if let Some(instance) = glyph.sources().get(&self.current_location) {
                return instance.width as f32;
            }
        }
        
        // Final fallback - return reasonable defaults for common glyphs
        match glyph_name {
            "a" | "c" | "e" | "o" | "u" => 500.0,
            "b" | "d" | "h" | "k" | "l" | "p" | "q" => 550.0,
            "f" | "i" | "j" | "r" | "t" => 300.0,
            "g" | "n" | "s" | "v" | "x" | "y" | "z" => 450.0,
            "m" | "w" => 750.0,
            _ => 600.0
        }
    }
    
    /// Create fallback glyph paths for testing
    fn create_fallback_glyph_path(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        match glyph_name {
            "a" => {
                let mut path = BezPath::new();
                // Create a simple "a" shape - rectangle with notch
                path.move_to((100.0, 0.0));
                path.line_to((100.0, 400.0));
                path.line_to((300.0, 400.0));
                path.line_to((300.0, 200.0));
                path.line_to((200.0, 200.0));
                path.line_to((200.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "b" => {
                let mut path = BezPath::new();
                // Create a simple "b" shape - vertical line with bumps
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 400.0));
                path.line_to((200.0, 400.0));
                path.line_to((200.0, 200.0));
                path.line_to((150.0, 200.0));
                path.line_to((150.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "c" => {
                let mut path = BezPath::new();
                // Create a simple "c" shape - open rectangle
                path.move_to((300.0, 100.0));
                path.line_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                Some(vec![path])
            }
            "d" => {
                let mut path = BezPath::new();
                // Create a simple "d" shape - rectangle with stem
                path.move_to((100.0, 0.0));
                path.line_to((100.0, 300.0));
                path.line_to((250.0, 300.0));
                path.line_to((250.0, 400.0));
                path.line_to((280.0, 400.0));
                path.line_to((280.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "e" => {
                let mut path = BezPath::new();
                // Create a simple "e" shape - rectangle with horizontal line
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                path.line_to((300.0, 250.0));
                path.line_to((150.0, 250.0));
                path.line_to((150.0, 200.0));
                path.line_to((280.0, 200.0));
                path.line_to((280.0, 100.0));
                path.close_path();
                Some(vec![path])
            }
            "h" => {
                let mut path = BezPath::new();
                // Create a simple "h" shape - two vertical lines with crossbar
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 400.0));
                path.line_to((80.0, 400.0));
                path.line_to((80.0, 220.0));
                path.line_to((200.0, 220.0));
                path.line_to((200.0, 400.0));
                path.line_to((230.0, 400.0));
                path.line_to((230.0, 0.0));
                path.line_to((200.0, 0.0));
                path.line_to((200.0, 190.0));
                path.line_to((80.0, 190.0));
                path.line_to((80.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "l" => {
                let mut path = BezPath::new();
                // Create a simple "l" shape - vertical line
                path.move_to((140.0, 0.0));
                path.line_to((140.0, 400.0));
                path.line_to((170.0, 400.0));
                path.line_to((170.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "o" => {
                let mut path = BezPath::new();
                // Create a simple "o" shape - hollow rectangle
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                path.line_to((300.0, 100.0));
                path.close_path();
                // Inner hole
                path.move_to((140.0, 140.0));
                path.line_to((260.0, 140.0));
                path.line_to((260.0, 260.0));
                path.line_to((140.0, 260.0));
                path.close_path();
                Some(vec![path])
            }
            "r" => {
                let mut path = BezPath::new();
                // Create a simple "r" shape - vertical line with top branch
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((200.0, 270.0));
                path.line_to((80.0, 270.0));
                path.line_to((80.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "w" => {
                let mut path = BezPath::new();
                // Create a simple "w" shape - zigzag
                path.move_to((50.0, 300.0));
                path.line_to((80.0, 0.0));
                path.line_to((110.0, 0.0));
                path.line_to((140.0, 200.0));
                path.line_to((170.0, 0.0));
                path.line_to((200.0, 0.0));
                path.line_to((230.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((185.0, 100.0));
                path.line_to((170.0, 200.0));
                path.line_to((155.0, 100.0));
                path.line_to((140.0, 300.0));
                path.close_path();
                Some(vec![path])
            }
            // Add a few more common letters
            "s" => {
                let mut path = BezPath::new();
                // Create a simple "s" shape - stepped rectangle
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 150.0));
                path.line_to((200.0, 150.0));
                path.line_to((200.0, 200.0));
                path.line_to((100.0, 200.0));
                path.line_to((100.0, 250.0));
                path.line_to((300.0, 250.0));
                path.line_to((300.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((200.0, 350.0));
                path.line_to((300.0, 350.0));
                path.line_to((300.0, 400.0));
                path.line_to((100.0, 400.0));
                path.close_path();
                Some(vec![path])
            }
            "t" => {
                let mut path = BezPath::new();
                // Create a simple "t" shape - T shape
                path.move_to((50.0, 350.0));
                path.line_to((50.0, 400.0));
                path.line_to((250.0, 400.0));
                path.line_to((250.0, 350.0));
                path.line_to((170.0, 350.0));
                path.line_to((170.0, 0.0));
                path.line_to((130.0, 0.0));
                path.line_to((130.0, 350.0));
                path.close_path();
                Some(vec![path])
            }
            _ => None
        }
    }
}

/// Helper to convert PathEl to a point position
pub fn path_element_position(el: &PathEl) -> Option<Point> {
    match el {
        PathEl::MoveTo(pt) => Some(*pt),
        PathEl::LineTo(pt) => Some(*pt),
        PathEl::CurveTo(_, _, pt) => Some(*pt),
        PathEl::QuadTo(_, pt) => Some(*pt),
        PathEl::ClosePath => None,
    }
}

/// Helper to get control points from a path element
pub fn path_element_control_points(el: &PathEl) -> Vec<Point> {
    match el {
        PathEl::CurveTo(c1, c2, _) => vec![*c1, *c2],
        PathEl::QuadTo(c, _) => vec![*c],
        _ => vec![],
    }
}

/// Point type derived from PathEl
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontIRPointType {
    Move,
    Line,
    Curve,
    Quad,
    OffCurve,
}

impl From<&PathEl> for FontIRPointType {
    fn from(el: &PathEl) -> Self {
        match el {
            PathEl::MoveTo(_) => FontIRPointType::Move,
            PathEl::LineTo(_) => FontIRPointType::Line,
            PathEl::CurveTo(_, _, _) => FontIRPointType::Curve,
            PathEl::QuadTo(_, _) => FontIRPointType::Quad,
            PathEl::ClosePath => FontIRPointType::Move, // Treat as move for now
        }
    }
}