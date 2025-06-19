//! Point list management for glyph contours
//!
//! This module provides the `PointList` component for working with sequences of points
//! that form the outline of a glyph.

use bevy::prelude::*;
use norad::Contour;

use crate::geometry::point::EditPoint;

/// A list of points that form a contour in a glyph's outline
#[derive(Component, Debug, Clone)]
pub struct PointList {
    /// The points that make up this contour, in order
    #[allow(dead_code)]
    points: Vec<EditPoint>,
    /// Whether this contour is closed (end connects to start)
    #[allow(dead_code)]
    closed: bool,
}

impl PointList {
    /// Creates a new, empty point list
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: true, // UFO contours are typically closed
        }
    }

    /// Creates a point list from UFO contour data
    #[allow(dead_code)]
    pub fn from_contour(contour: &Contour) -> Self {
        let points: Vec<EditPoint> =
            contour.points.iter().map(EditPoint::from).collect();

        Self {
            points,
            closed: true, // UFO contours are always closed
        }
    }

    /// Gets a reference to all points in this contour
    #[allow(dead_code)]
    pub fn points(&self) -> &[EditPoint] {
        &self.points
    }

    /// Checks if this contour is closed
    #[allow(dead_code)]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Gets the number of points in this contour
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Checks if this contour is empty (has no points)
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Default for PointList {
    fn default() -> Self {
        Self::new()
    }
} 