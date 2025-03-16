use bevy::prelude::*;
use std::collections::BTreeSet;

//-----------------------------------------------------------------------
// Selection System Components
//-----------------------------------------------------------------------
// This file defines the core components and state used by the selection
// system, which handles user interactions with on-screen elements such as
// glyph points, curves, and other UI elements. The system allows users to:
// - Select individual entities
// - Multi-select using modifier keys
// - Perform rectangular selection via click and drag
// - Track hover state for improved UX feedback
//
// ARCHITECTURE OVERVIEW:
// The selection system uses a component-based approach where:
// 1. `Selectable` marks entities that can be interacted with
// 2. `Selected` and `Hovered` are added/removed dynamically to track state
// 3. `SelectionState` resource manages the global selection state
// 4. `SelectionRect` represents an in-progress rectangular selection
//
// Typical workflow:
// - Mouse hover -> Add `Hovered` component
// - Mouse click on entity -> Add `Selected` component + update `SelectionState`
// - Shift+click -> Toggle selection state (multi-select)
// - Click and drag -> Create `SelectionRect` and select all contained entities
// - Click on empty space -> Clear selection

/// Marker component for entities that can be selected
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Selectable;

/// Marker component for entities that are currently selected
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Selected;

/// Marker component for entities that are currently hovered
/// Disabled per user request - hover functionality removed
#[allow(dead_code)]
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Hovered;

/// Component for the selection rectangle during drag operations
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SelectionRect {
    /// The start position of the selection rectangle
    pub start: Vec2,
    /// The end position of the selection rectangle
    pub end: Vec2,
}

/// The type of point (on-curve or off-curve)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct PointType {
    /// Whether this is an on-curve point
    pub is_on_curve: bool,
}

/// Component that links an entity to a specific point in a glyph outline
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct GlyphPointReference {
    /// Name of the glyph this point belongs to
    pub glyph_name: String,
    /// Index of the contour within the glyph outline
    pub contour_index: usize,
    /// Index of the point within the contour
    pub point_index: usize,
}

impl Default for PointType {
    fn default() -> Self {
        Self { is_on_curve: true }
    }
}

/// Resource to track the global selection state
#[derive(Resource, Debug, Default)]
pub struct SelectionState {
    /// Whether multi-select mode is active (e.g., shift key is held)
    pub multi_select: bool,
    /// The currently selected entities
    pub selected: BTreeSet<Entity>,
}

/// A collection of selected entities
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Selection(BTreeSet<Entity>);

impl Selection {
    /// Create a new empty selection
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// Check if the selection is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of selected entities
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the selection contains the given entity
    #[allow(dead_code)]
    pub fn contains(&self, entity: &Entity) -> bool {
        self.0.contains(entity)
    }

    /// Insert an entity into the selection
    #[allow(dead_code)]
    pub fn insert(&mut self, entity: Entity) -> bool {
        self.0.insert(entity)
    }

    /// Remove an entity from the selection
    #[allow(dead_code)]
    pub fn remove(&mut self, entity: &Entity) -> bool {
        self.0.remove(entity)
    }

    /// Clear the selection
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Select a single entity, clearing any previous selection
    #[allow(dead_code)]
    pub fn select_one(&mut self, entity: Entity) {
        self.clear();
        self.insert(entity);
    }
}

impl FromIterator<Entity> for Selection {
    fn from_iter<I: IntoIterator<Item = Entity>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
