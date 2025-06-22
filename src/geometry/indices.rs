//! Type-safe indices for font editing operations
//! 
//! This module provides wrapper types around `usize` to prevent index confusion
//! and make the code more readable for junior developers.

use bevy::prelude::*;

/// A type-safe wrapper for contour indices
/// 
/// This prevents accidentally using a point index where a contour index is expected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ContourIndex(pub usize);

impl ContourIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
    
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for ContourIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

/// A type-safe wrapper for point indices within a contour
/// 
/// This prevents accidentally using a contour index where a point index is expected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct PointIndex(pub usize);

impl PointIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
    
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for PointIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

/// A unique identifier for a specific point in a glyph
/// 
/// This combines glyph name, contour index, and point index for unambiguous identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub struct PointId {
    pub glyph_name: String,
    pub contour_index: ContourIndex,
    pub point_index: PointIndex,
}

impl PointId {
    pub fn new(glyph_name: String, contour_index: ContourIndex, point_index: PointIndex) -> Self {
        Self {
            glyph_name,
            contour_index,
            point_index,
        }
    }
}

/// Common constants for font editing
pub mod constants {
    /// Default advance width for new glyphs (in font units)
    pub const DEFAULT_ADVANCE_WIDTH: f64 = 600.0;
    
    /// Selection margin for point picking (in screen pixels)
    pub const SELECTION_MARGIN: f32 = 8.0;
    
    /// Default nudge increment (in font units)
    pub const DEFAULT_NUDGE_INCREMENT: f32 = 1.0;
    
    /// Large nudge increment when Shift is held (in font units)
    pub const LARGE_NUDGE_INCREMENT: f32 = 10.0;
} 