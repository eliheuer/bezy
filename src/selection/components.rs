use bevy::prelude::*;

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

/// Component that marks an entity as selectable
///
/// Attach this component to any entity that should be interactable
/// via mouse clicks or rectangular selection.
///
/// This is a marker component with no data - its presence is what matters.
/// Selection systems query for entities with this component to determine
/// what can be selected. Only entities with this component will respond
/// to selection-related interactions.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Selectable;

/// Component that marks an entity as currently selected
///
/// This component is dynamically added to entities when they are selected
/// by the user and removed when deselected. It can be used by rendering
/// systems to change the visual appearance of selected elements.
///
/// Selection systems add/remove this component based on user interaction.
/// Rendering systems can query for this component to apply visual styles
/// for selected state (e.g., highlight colors, showing control handles).
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Selected;

/// Component that marks an entity as being hovered
///
/// This component is dynamically added to entities when the mouse cursor
/// is positioned over them. It enables visual feedback (like highlighting)
/// before the user actually clicks to select.
///
/// Hover detection systems add/remove this component as the mouse moves.
/// It provides immediate visual feedback to users about what would be
/// selected if they click, improving usability.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Hovered;

/// Component that marks a glyph point's type (on-curve or off-curve)
///
/// In font editing, points can be either on the actual curve path or
/// control points that influence the curve's shape but aren't on the path.
/// This distinction affects both behavior and visual representation.
///
/// This component works together with the selection system to determine:
/// - How points are rendered (different visual styles)
/// - How they behave when manipulated (e.g., on-curve points move segments,
///   off-curve points affect curvature)
/// - Selection grouping behavior (selecting a curve might select its points)
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub enum PointType {
    /// On-curve point (part of the actual path)
    OnCurve,
    /// Off-curve point (control point for curves)
    OffCurve,
}

impl Default for PointType {
    fn default() -> Self {
        PointType::OnCurve
    }
}

/// Component for rectangle selection in progress
///
/// This component represents the current selection rectangle drawn by the user
/// when performing a drag selection. It stores both the starting point (where
/// the mouse was first pressed) and the current end point (current mouse position).
/// The selection system uses these coordinates to determine which entities fall
/// within the selection rectangle.
///
/// This component is typically attached to a temporary entity created when
/// the user starts dragging. Selection systems update the `end` field as the
/// mouse moves and test `Selectable` entities to see if they're contained
/// within the rectangle. Once the drag operation completes, entities within
/// the rectangle have the `Selected` component added.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SelectionRect {
    /// Start position of the rectangle in world coordinates
    pub start: Vec2,
    /// Current position of the rectangle in world coordinates
    pub end: Vec2,
}

impl Default for SelectionRect {
    fn default() -> Self {
        Self {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
        }
    }
}

/// Resource to track selection state
///
/// This resource maintains the global state of the selection system, including
/// which entities are currently selected, whether multi-select mode is active,
/// and the status of drag selection operations. It's updated by selection-related
/// systems and queried by other systems that need to operate on selected entities.
///
/// As a Bevy resource, this state is globally accessible to any system that needs
/// to know about or modify the current selection. For example:
/// - Transformation systems that need to move all selected entities
/// - Property panels that display/edit properties of selected entities
/// - Commands that operate on the current selection (delete, duplicate, etc.)
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct SelectionState {
    /// Tracks if we're in multi-select mode (e.g., Shift is held)
    pub multi_select: bool,
    /// Last selected entity for double-click detection or group operations
    pub last_selected: Option<Entity>,
    /// List of currently selected entities
    #[reflect(ignore)]
    pub selected_entities: Vec<Entity>,
    /// Is a rectangular selection in progress
    pub drag_selecting: bool,
}

impl SelectionState {
    /// Add an entity to the selected list
    ///
    /// If the entity is not already selected, adds it to the selection and
    /// updates the last_selected reference. This method doesn't handle
    /// deselection logic or multi-select behavior - that's handled by the
    /// selection systems.
    ///
    /// This is typically called by selection systems after detecting a click
    /// on a selectable entity.
    pub fn add_selected(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.last_selected = Some(entity);
        }
    }

    /// Clear the selection state
    ///
    /// Removes all entities from the selection and resets the last_selected
    /// reference. This is typically called when clicking on empty space or
    /// when explicitly clearing the selection.
    ///
    /// Note: This method only updates the selection state resource.
    /// The system that calls this should also remove the `Selected` component
    /// from all previously selected entities.
    pub fn clear(&mut self) {
        self.selected_entities.clear();
        self.last_selected = None;
    }

    /// Check if the selection is empty
    ///
    /// Returns true if no entities are currently selected.
    /// This is useful for systems that need to know if there's an active selection
    /// before performing operations.
    pub fn is_empty(&self) -> bool {
        self.selected_entities.is_empty()
    }

    /// Get the number of selected entities
    ///
    /// Returns the count of currently selected entities.
    /// This can be used by UI systems to display selection info or by
    /// operation systems to optimize for single vs. multi-selection cases.
    pub fn count(&self) -> usize {
        self.selected_entities.len()
    }
}
