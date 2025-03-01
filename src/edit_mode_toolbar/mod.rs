use bevy::prelude::*;

mod hyper;
mod knife;
mod measure;
mod pan;
mod pen;
mod primitives;
mod select;
mod text;
mod ui;

pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use measure::MeasureMode;
pub use pan::PanMode;
pub use pen::PenMode;
pub use primitives::PrimitivesMode;
pub use select::{
    draw_selected_points_system, select_point_system, SelectMode,
};
pub use text::TextMode;
pub use ui::*;

// Trait that all edit modes must implement
pub trait EditModeSystem: Send + Sync + 'static {
    fn update(&self, commands: &mut Commands);

    // Default implementations for lifecycle methods
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}

/// Plugin that adds all the toolbar functionality
pub struct EditModeToolbarPlugin;

impl Plugin for EditModeToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
            // First add the selection plugin which initializes resources
            .add_plugins(select::SelectPlugin)
            // Then add our systems to the update schedule
            .add_systems(
                Update,
                (
                    // Selection systems
                    select_point_system,
                    draw_selected_points_system,
                    select::debug_selection_state,
                    select::debug_camera_info,
                    select::debug_scene_entities,
                    // UI systems
                    handle_toolbar_mode_selection,
                    update_current_edit_mode,
                ),
            );

        info!("EditModeToolbarPlugin initialized with selection functionality");
    }
}
