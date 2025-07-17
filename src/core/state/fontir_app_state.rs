//! FontIR-based application state
//!
//! This module provides the new AppState that directly uses FontIR structures
//! instead of custom data types. This enables multi-format support and
//! variable font handling.

use anyhow::Result;
use bevy::prelude::*;
use fontdrasil::coords::NormalizedLocation;
use fontir::ir::Glyph as FontIRGlyph;
use fontir::source::Source;
use kurbo::{BezPath, PathEl, Point};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use ufo2fontir::source::DesignSpaceIrSource;

/// The main application state using FontIR
#[derive(Resource, Clone)]
pub struct FontIRAppState {
    /// The FontIR source (handles UFO, designspace, etc.)
    pub source: Arc<DesignSpaceIrSource>,
    
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
        
        // Initialize with default location (for variable fonts)
        let current_location = NormalizedLocation::default();
        
        Ok(Self {
            source,
            glyph_cache: HashMap::new(),
            current_glyph: None,
            current_location,
            source_path: path,
        })
    }
    
    /// Get a glyph by name
    pub fn get_glyph(&self, name: &str) -> Option<&FontIRGlyph> {
        self.glyph_cache.get(name).map(|g| g.as_ref())
    }
    
    /// Get the current glyph's path at the current location
    pub fn get_current_glyph_paths(&self) -> Option<Vec<BezPath>> {
        let glyph_name = self.current_glyph.as_ref()?;
        let glyph = self.get_glyph(glyph_name)?;
        
        // Get the instance at our current location
        let instance = glyph.sources().get(&self.current_location)?;
        
        // Return the contours as BezPaths
        Some(instance.contours.clone())
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
        // This would iterate through the source and populate glyph_cache
        // For now, placeholder
        todo!("Implement glyph loading from FontIR source")
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