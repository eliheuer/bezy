pub mod components;
pub mod nudge;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
pub use nudge::*;
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
            .register_type::<GlyphPointReference>()
            .register_type::<LastEditType>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<systems::DragSelectionState>()
            // Add systems for selection
            .add_systems(
                Update,
                (
                    systems::handle_mouse_input,
                    systems::handle_selection_shortcuts,
                    // Temporarily disable update_hover_state system to focus on nudging
                    // systems::update_hover_state,
                    systems::render_selection_rect,
                    systems::render_selected_entities,
                    systems::render_hovered_entities,
                    // Add the new system to update glyph data
                    systems::update_glyph_data_from_selection,
                ),
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin);
    }
}
