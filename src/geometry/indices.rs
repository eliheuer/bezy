//! Entity indexing and identification
//!
//! This module provides types for uniquely identifying and indexing points,
//! contours, and other geometry entities within a glyph.

use bevy::prelude::*;

/// Index of a contour within a glyph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContourIndex(usize);

impl ContourIndex {
    #[allow(dead_code)]
    pub fn new(index: usize) -> Self {
        ContourIndex(index)
    }

    #[allow(dead_code)]
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for ContourIndex {
    fn from(index: usize) -> Self {
        ContourIndex(index)
    }
}

/// Index of a point within a contour
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointIndex(usize);

impl PointIndex {
    #[allow(dead_code)]
    pub fn new(index: usize) -> Self {
        PointIndex(index)
    }

    #[allow(dead_code)]
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for PointIndex {
    fn from(index: usize) -> Self {
        PointIndex(index)
    }
}

/// Complete identification of a specific point in a glyph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointId {
    pub glyph_name: String,
    pub contour_index: ContourIndex,
    pub point_index: PointIndex,
}

impl PointId {
    #[allow(dead_code)]
    pub fn new(glyph_name: String, contour_index: ContourIndex, point_index: PointIndex) -> Self {
        Self {
            glyph_name,
            contour_index,
            point_index,
        }
    }
}

/// Constants for font editing defaults
pub struct GeometryConstants;

impl GeometryConstants {
    #[allow(dead_code)]
    pub const DEFAULT_ADVANCE_WIDTH: f64 = 600.0;

    /// Distance in pixels for selection hit testing
    #[allow(dead_code)]
    pub const SELECTION_MARGIN: f32 = 8.0;

    #[allow(dead_code)]
    pub const DEFAULT_NUDGE_INCREMENT: f32 = 1.0;

    #[allow(dead_code)]
    pub const LARGE_NUDGE_INCREMENT: f32 = 10.0;
} 