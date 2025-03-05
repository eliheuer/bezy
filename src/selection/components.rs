use bevy::prelude::*;

/// Component that marks an entity as selectable
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Selectable;

/// Component that marks an entity as currently selected
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Selected;

/// Component that marks an entity as being hovered
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Hovered;

/// Component that marks a glyph point's type (on-curve or off-curve)
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
    pub fn add_selected(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
            self.last_selected = Some(entity);
        }
    }

    /// Clear the selection state
    pub fn clear(&mut self) {
        self.selected_entities.clear();
        self.last_selected = None;
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.selected_entities.is_empty()
    }

    /// Get the number of selected entities
    pub fn count(&self) -> usize {
        self.selected_entities.len()
    }
}
