#![allow(unused_imports)]

use crate::editing::edit_type::EditType;
use crate::editing::selection::systems::*;
use crate::editing::UndoPlugin;
use crate::core::state::AppState;
use bevy::prelude::*;

pub mod components;
pub mod coordinate_system;
pub mod nudge;
pub mod systems;

pub use components::*;
pub use nudge::*;
pub use systems::*;

use std::collections::HashMap;

/// Resource to track the drag selection state
#[derive(Resource, Default)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection (in design space)
    pub start_position: Option<crate::geometry::design_space::DPoint>,
    /// The current position of the drag selection (in design space)
    pub current_position: Option<crate::geometry::design_space::DPoint>,
    /// Whether this is a multi-select operation (shift is held)
    pub is_multi_select: bool,
    /// The previous selection before the drag started
    pub previous_selection: Vec<Entity>,
    pub selection_rect_entity: Option<Entity>,
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
            // Add events
            .add_event::<systems::AppStateChanged>()
            .add_event::<EditEvent>()
            .register_type::<EditType>()
            .register_type::<NudgeState>()
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

            // Configure system sets for proper ordering
            .configure_sets(
                Update,
                (
                    SelectionSystemSet::Input,
                    SelectionSystemSet::Processing,
                )
                    .chain(),
            )
            .configure_sets(
                PostUpdate,
                (
                    SelectionSystemSet::Render,
                ),
            )
            // Input systems - the process_selection_input_events system handles the actual selection logic
            // It's called by the centralized input consumer system when in select mode
            .add_systems(
                Update,
                systems::process_selection_input_events
                    .in_set(SelectionSystemSet::Input),
            )
            // Processing systems
            .add_systems(
                Update,
                (
                    sync_selected_components,
                    systems::update_glyph_data_from_selection,
                    systems::clear_selection_on_app_change,
                    systems::cleanup_click_resource,
                )
                    .in_set(SelectionSystemSet::Processing)
                    .after(SelectionSystemSet::Input),
            )
            // Add the new ECS-based point management systems
            .add_systems(
                Update,
                (
                    // systems::spawn_active_sort_points, // DISABLED: Causes duplicate point entities
                    systems::despawn_inactive_sort_points,
                    systems::sync_point_positions_to_sort,
                )
                    .after(systems::update_glyph_data_from_selection),
            )
            // Rendering systems - moved to PostUpdate to run after transform propagation
            .add_systems(
                PostUpdate,
                (
                    systems::render_selection_marquee,
                    systems::render_selected_entities,
                    systems::render_all_point_entities,
                    systems::render_control_handles,
                    systems::debug_print_selection_rects, // TEMP: debug system
                )
                    .in_set(SelectionSystemSet::Render),
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin);

        // Register debug validation system only in debug builds
        #[cfg(debug_assertions)]
        app.add_systems(
            PostUpdate,
            systems::debug_validate_point_entity_uniqueness.after(SelectionSystemSet::Render),
        );
    }
}

/// System sets for Selection
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SelectionSystemSet {
    #[allow(dead_code)]
    Input,
    #[allow(dead_code)]
    Processing,
    #[allow(dead_code)]
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
            info!(
                "Adding Selected component to entity {:?} from selection state",
                entity
            );
        }
    }

    // Then, ensure all entities with the Selected component are in the selection_state
    for entity in &selected_entities {
        if !selection_state.selected.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            info!("Removing Selected component from entity {:?} not in selection state", entity);
        }
    }
}

#[allow(dead_code)]
fn selection_drag_active(drag_state: Res<DragSelectionState>) -> bool {
    drag_state.is_dragging
} 