pub mod components;
pub mod nudge;

use bevy::prelude::*;
pub use components::*;
pub use nudge::*;
use std::collections::HashMap;

/// Resource to track the drag selection state
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection
    pub start_position: Option<Vec2>,
    /// The current position of the drag selection
    pub current_position: Option<Vec2>,
    /// Whether this is a multi-select operation (shift is held)
    pub is_multi_select: bool,
    /// The previous selection before the drag started
    #[reflect(ignore)]
    pub previous_selection: Vec<Entity>,
}

/// Resource to track the state of dragging points
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DragPointState {
    /// Whether a point drag is in progress
    pub is_dragging: bool,
    /// The start position of the drag
    pub start_position: Option<Vec2>,
    /// The current position of the drag
    pub current_position: Option<Vec2>,
    /// The entities being dragged
    #[reflect(ignore)]
    pub dragged_entities: Vec<Entity>,
    /// The original positions of the dragged entities
    #[reflect(ignore)]
    pub original_positions: HashMap<Entity, Vec2>,
}

/// Plugin to add selection functionality to the font editor
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register components
            .register_type::<Selectable>()
            .register_type::<Selected>()
            .register_type::<Hovered>()
            .register_type::<SelectionRect>()
            .register_type::<PointType>()
            .register_type::<GlyphPointReference>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<DragSelectionState>()
            .init_resource::<DragPointState>()
            // Add the nudge plugin
            .add_plugins(NudgePlugin);
    }
}

/// System sets for Selection
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SelectionSystemSet {
    Input,
    Processing,
    Render,
}

/// System to ensure Selected components are synchronized with SelectionState
pub fn sync_selected_components(
    mut commands: Commands,
    selection_state: Res<SelectionState>,
    selected_entities: Query<Entity, With<Selected>>,
    entities: Query<Entity>,
) {
    // Always run this system to ensure components stay synchronized
    debug!(
        "Synchronizing Selected components with SelectionState (current: {})",
        selection_state.selected.len()
    );

    // First, ensure all entities in the selection_state have the Selected component
    for &entity in &selection_state.selected {
        // Only add the component if the entity is valid
        if entities.contains(entity) && !selected_entities.contains(entity) {
            commands.entity(entity).insert(Selected);
            debug!(
                "Adding Selected component to entity {:?} from selection state",
                entity
            );
        }
    }

    // Then, ensure all entities with the Selected component are in the selection_state
    for entity in &selected_entities {
        if !selection_state.selected.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            debug!("Removing Selected component from entity {:?} not in selection state", entity);
        }
    }
} 