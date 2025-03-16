pub mod components;
pub mod nudge;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
pub use nudge::*;

/// Resource to track the drag selection state
#[derive(Resource, Default)]
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
    pub previous_selection: Vec<Entity>,
}

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
            .register_type::<GlyphPointReference>()
            .register_type::<LastEditType>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<DragSelectionState>()
            // Add systems for selection
            .add_systems(
                Update,
                (
                    systems::handle_mouse_input,
                    systems::handle_selection_shortcuts,
                    systems::handle_key_releases,
                    // Hover functionality disabled per user request
                    // systems::update_hover_state,
                    systems::render_selection_rect,
                    systems::render_selected_entities,
                    // systems::render_hovered_entities,
                    // Add the new system to update glyph data
                    systems::update_glyph_data_from_selection,
                ),
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin);
    }
}
