use bevy::prelude::*;

use crate::cameras::toggle_camera_controls;
use crate::crypto_toolbar::CryptoToolbarPlugin;
use crate::edit_mode_toolbar::CurrentEditMode;
use crate::setup::setup;
use crate::theme::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::ufo::{initialize_font_state, print_font_info_to_terminal};

/// Configure the default Bevy plugins with custom settings
pub fn configure_default_plugins() -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                // Tell wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        // Disable Bevy's default LogPlugin since we're using our own custom logger
        .build()
        .disable::<bevy::log::LogPlugin>()
}

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
        app.add_systems(Update, toggle_camera_controls);
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
