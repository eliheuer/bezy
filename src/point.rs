use bevy::prelude::*;
use kurbo::Point;
use norad::glyph::{ContourPoint, PointType};

/// A point in a glyph's outline
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EditPoint {
    /// The position of the point in glyph space
    pub position: Point,
    /// The type of the point (move, line, curve, etc.)
    pub point_type: PointType,
}

impl EditPoint {
    /// Create a new point with the given position and type
    pub fn new(position: Point, point_type: PointType) -> Self {
        Self {
            position,
            point_type,
        }
    }
}

impl From<&ContourPoint> for EditPoint {
    fn from(point: &ContourPoint) -> Self {
        EditPoint {
            position: Point::new(point.x as f64, point.y as f64),
            point_type: point.typ.clone(),
        }
    }
} 