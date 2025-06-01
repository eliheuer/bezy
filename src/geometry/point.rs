//! Point and entity management for glyph editing
//!
//! This module provides the core structures for working with individual points
//! and entities within a glyph's outline.

use bevy::prelude::*;
use kurbo::Point;
use norad::glyph::{ContourPoint, PointType};

/// A point in a glyph's outline that can be edited
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EditPoint {
    pub position: Point,       // Position in glyph coordinate space
    pub point_type: PointType, // Point type (move, line, curve, etc.)
}

impl EditPoint {
    /// Creates a new editable point
    #[allow(dead_code)]
    pub fn new(position: Point, point_type: PointType) -> Self {
        Self {
            position,
            point_type,
        }
    }
}

impl From<&ContourPoint> for EditPoint {
    /// Converts a UFO contour point to an editable point
    fn from(point: &ContourPoint) -> Self {
        EditPoint {
            position: Point::new(point.x as f64, point.y as f64),
            point_type: point.typ.clone(),
        }
    }
}

/// Unique identifier for entities in a glyph (points, guides, components)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Reflect)]
pub struct EntityId {
    parent: u32,      // The parent path or component ID
    index: u16,       // The index within the parent
    kind: EntityKind, // The type of entity this ID refers to
}

/// The different types of entities that can exist in a glyph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Reflect)]
pub enum EntityKind {
    Point,     // A point in a contour path
    Guide,     // A guide line for alignment
    Component, // A component reference to another glyph
}

impl EntityId {
    /// Creates an entity ID for a point in a specific contour
    #[allow(dead_code)]
    pub fn point(parent: u32, index: u16) -> Self {
        Self {
            parent,
            index,
            kind: EntityKind::Point,
        }
    }

    /// Creates an entity ID for a guide
    #[allow(dead_code)]
    pub fn guide(index: u16) -> Self {
        Self {
            parent: 0,
            index,
            kind: EntityKind::Guide,
        }
    }

    /// Gets the parent container ID
    #[allow(dead_code)]
    pub fn parent(&self) -> u32 {
        self.parent
    }

    /// Gets the index within the parent container
    #[allow(dead_code)]
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Checks if this entity is a guide
    #[allow(dead_code)]
    pub fn is_guide(&self) -> bool {
        self.kind == EntityKind::Guide
    }

    /// Checks if this entity is a point
    #[allow(dead_code)]
    pub fn is_point(&self) -> bool {
        self.kind == EntityKind::Point
    }

    /// Checks if this entity is a component
    #[allow(dead_code)]
    pub fn is_component(&self) -> bool {
        self.kind == EntityKind::Component
    }
}
