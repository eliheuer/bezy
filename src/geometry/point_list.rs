use bevy::prelude::*;
use norad::glyph::Contour;

use crate::geometry::point::EditPoint;

/// A list of points forming a contour in a glyph's outline
#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct PointList {
    /// The points in the contour
    points: Vec<EditPoint>,
    /// Whether the contour is closed
    closed: bool,
}

impl PointList {
    /// Create a new empty point list
    #[allow(dead_code)]
    pub fn new() -> Self {
        PointList {
            points: Vec::new(),
            closed: true, // UFO contours are always closed
        }
    }

    /// Create a point list from a norad Contour
    #[allow(dead_code)]
    pub fn from_contour(contour: &Contour) -> Self {
        let points = contour.points.iter().map(EditPoint::from).collect();

        PointList {
            points,
            closed: true,
        }
    }

    /// Get the points in this contour
    #[allow(dead_code)]
    pub fn points(&self) -> &[EditPoint] {
        &self.points
    }
}
