//! Font data structures
//!
//! This module contains the core font data structures that represent
//! the font in a thread-safe format optimized for real-time editing.

use std::collections::HashMap;
use std::path::PathBuf;

/// Thread-safe font data structure
#[derive(Clone, Default)]
pub struct FontData {
    /// All glyph data extracted from norad and stored thread-safely
    pub glyphs: HashMap<String, GlyphData>,
    /// Path to the UFO file (for saving)
    pub path: Option<PathBuf>,
}

/// Thread-safe glyph data
#[derive(Clone, Debug)]
pub struct GlyphData {
    /// Glyph name
    pub name: String,
    /// Advance width
    pub advance_width: f64,
    /// Advance height (optional)
    pub advance_height: Option<f64>,
    /// Unicode codepoints for this glyph
    pub unicode_values: Vec<char>,
    /// Glyph outline data
    pub outline: Option<OutlineData>,
}

/// Thread-safe outline data
#[derive(Clone, Debug)]
pub struct OutlineData {
    /// Contour data
    pub contours: Vec<ContourData>,
}

/// Thread-safe contour data
#[derive(Clone, Debug)]
pub struct ContourData {
    /// Points in this contour
    pub points: Vec<PointData>,
}

/// Thread-safe point data
#[derive(Clone, Debug)]
pub struct PointData {
    /// X coordinate
    pub x: f64,
    /// Y coordinate  
    pub y: f64,
    /// Point type
    pub point_type: PointTypeData,
}

/// Thread-safe point type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointTypeData {
    Move,
    Line,
    OffCurve,
    Curve,
    QCurve,
}

impl FontData {
    /// Get a glyph by name
    pub fn get_glyph(&self, name: &str) -> Option<&GlyphData> {
        self.glyphs.get(name)
    }
}
