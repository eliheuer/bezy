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

/// An identifier for a point or other entity in the glyph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct EntityId {
    /// The parent path or component ID
    parent: u32,
    /// The index within the parent
    index: u16,
    /// The type of entity
    kind: EntityKind,
}

/// The type of entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub enum EntityKind {
    /// A point in a path
    Point,
    /// A guide
    Guide,
    /// A component
    Component,
}

impl EntityId {
    /// Create a new entity ID for a point
    pub fn point(parent: u32, index: u16) -> Self {
        Self {
            parent,
            index,
            kind: EntityKind::Point,
        }
    }

    /// Create a new entity ID for a guide
    pub fn guide(index: u16) -> Self {
        Self {
            parent: 0,
            index,
            kind: EntityKind::Guide,
        }
    }

    /// Get the parent ID
    pub fn parent(&self) -> u32 {
        self.parent
    }

    /// Check if this is a guide
    pub fn is_guide(&self) -> bool {
        self.kind == EntityKind::Guide
    }
}
