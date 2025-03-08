use bevy::prelude::*;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::crypto_toolbar::CryptoToolbarPlugin;
use crate::edit_mode_toolbar::CurrentEditMode;
use crate::setup::setup;
use crate::ufo::{initialize_font_state, print_font_info_to_terminal};

/// Plugin to organize debug-related systems
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::ufo::LastCodepointPrinted>()
            .add_systems(
                Update,
                (
                    // Print UFO and codepoint info to terminal
                    print_font_info_to_terminal,
                ),
            );
    }
}

/// Plugin to organize camera-related systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_coordinate_display, toggle_camera_controls),
        );
    }
}

/// Plugin to organize toolbar-related plugins
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentEditMode>()
            .add_plugins(CryptoToolbarPlugin);
    }
}

/// Plugin to organize setup systems
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (initialize_font_state, setup));
    }
}

/// Main application plugin that bundles all internal plugins
pub struct BezySystems;

impl Plugin for BezySystems {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SetupPlugin,
            DebugPlugin,
            CameraPlugin,
            ToolbarPlugin,
        ));
    }
}
