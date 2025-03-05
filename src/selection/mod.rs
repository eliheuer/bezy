mod components;
mod systems;

use crate::edit_mode_toolbar::select::SelectModeActive;
use bevy::prelude::*;
pub use components::*;
pub use systems::*;

/// Plugin to register all selection-related components and systems
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
            // Register resources
            .init_resource::<SelectionState>()
            // Register systems
            .add_systems(Update, selection_visualization)
            // Debug selection state
            .add_systems(
                Update,
                debug_selection_state
                    .run_if(resource_exists::<SelectModeActive>),
            )
            // Visual rendering systems for selection
            .add_systems(
                PostUpdate,
                (
                    render_selected_entities,
                    render_hovered_entities,
                    render_selection_rect,
                ),
            )
            // Run hover update system when select mode is active
            .add_systems(
                Update,
                update_hover_state
                    .run_if(resource_exists_and_equals(SelectModeActive(true))),
            )
            // Selection systems - only run when select mode is active
            .add_systems(
                Update,
                (
                    mark_selected_entities,
                    start_drag_selection,
                    update_drag_selection,
                    finish_drag_selection,
                )
                    .run_if(resource_exists_and_equals(SelectModeActive(true))),
            )
            // Keyboard shortcuts for selection (always active when selection mode is active)
            .add_systems(
                Update,
                handle_selection_shortcuts
                    .run_if(resource_exists_and_equals(SelectModeActive(true))),
            );
    }
}

/// Helper condition to check if a resource exists and equals a specific value
fn resource_exists_and_equals<T: Resource + PartialEq>(
    value: T,
) -> impl FnMut(Option<Res<T>>) -> bool {
    move |res: Option<Res<T>>| {
        if let Some(res) = res {
            *res == value
        } else {
            false
        }
    }
}
