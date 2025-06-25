//! Sort: Central concept representing a movable type piece
//!
//! A sort is a term from metal typesetting referring to the entire piece of metal
//! used in movable type. In Bezy, a sort mirrors the contents of a .glif file
//! from the UFO format and can contain compound glyphs with components and metadata.
//!
//! Sorts have two modes:
//! - Active: The sort being edited, shows editable outlines
//! - Inactive: Shows rendered outline, not editable
//!
//! Only one sort can be active at a time.

use bevy::prelude::*;
use norad::Glyph;

/// Core Sort entity - represents a single piece of movable type
#[derive(Component, Debug, Clone)]
pub struct Sort {
    /// The name of the glyph this sort represents (references virtual font)
    pub glyph_name: String,
    /// The sort's position in design space
    pub position: Vec2,
    /// The sort's advance width (cached from the glyph)
    pub advance_width: f32,
    /// Whether this sort is currently active for editing
    pub _is_active: bool,
    /// Unique identifier for this sort instance
    pub _id: SortId,
}

/// Unique identifier for sort instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SortId(pub u64);

impl SortId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Component to mark the active sort (only one can be active at a time)
#[derive(Component, Debug)]
pub struct ActiveSort;

/// Component to mark inactive sorts
#[derive(Component, Debug)]
pub struct InactiveSort;

/// Resource to track the currently active sort
#[derive(Resource, Default)]
pub struct ActiveSortState {
    /// The entity ID of the currently active sort, if any
    pub active_sort_entity: Option<Entity>,
}

impl Sort {
    /// Create a new sort from a glyph name
    pub fn new(glyph_name: String, position: Vec2, advance_width: f32) -> Self {
        Self {
            glyph_name,
            position,
            advance_width,
            _is_active: false,
            _id: SortId::new(),
        }
    }

    /// Create a new sort from a glyph (extracts name and advance width)
    pub fn _from_glyph(glyph: &Glyph, position: Vec2) -> Self {
        let advance_width = glyph.width as f32;

        Self::new(glyph.name().to_string(), position, advance_width)
    }

    /// Get the metrics box bounds for this sort
    /// This matches the backup implementation: from descender to ascender, full glyph width
    pub fn get_metrics_bounds(&self, font_metrics: &crate::core::state::FontMetrics) -> SortBounds {
        let width = self.advance_width;
        let ascender = font_metrics.ascender.unwrap_or(font_metrics.units_per_em * 0.8) as f32;
        let descender = font_metrics.descender.unwrap_or(-(font_metrics.units_per_em * 0.2)) as f32;

        SortBounds {
            min: self.position + Vec2::new(0.0, descender),
            max: self.position + Vec2::new(width, ascender),
        }
    }

    /// Check if this sort contains the given point
    pub fn contains_point(&self, point: Vec2, font_metrics: &crate::core::state::FontMetrics) -> bool {
        let bounds = self.get_metrics_bounds(font_metrics);
        point.x >= bounds.min.x
            && point.x <= bounds.max.x
            && point.y >= bounds.min.y
            && point.y <= bounds.max.y
    }
}

/// Bounds of a sort's metrics box
#[derive(Debug, Clone)]
pub struct SortBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl SortBounds {
    pub fn _width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn _height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn _center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
}

/// Events for sort management
#[derive(Event, Debug)]
pub enum SortEvent {
    /// Create a new sort
    #[allow(dead_code)]
    CreateSort {
        glyph_name: String,
        position: Vec2,
    },
    /// Activate a sort for editing
    ActivateSort {
        sort_entity: Entity,
    },
    /// Deactivate the current sort
    DeactivateSort,
    /// Move a sort to a new position
    _MoveSort {
        sort_entity: Entity,
        new_position: Vec2,
    },
    /// Delete a sort
    _DeleteSort {
        sort_entity: Entity,
    },
} 