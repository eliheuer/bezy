// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::debug_hud::{
    spawn_debug_text, spawn_main_toolbar_debug, update_main_toolbar_debug,
};
use crate::setup::setup;
use crate::theme::BACKGROUND_COLOR;
use crate::toolbar::{
    handle_toolbar_mode_selection, update_current_edit_mode, CurrentEditMode,
};
use crate::world_space::spawn_debug_path;

// Create the app and add the plugins and systems
pub fn create_app() -> App {
    let mut app = App::new();

    let window_config = Window {
        title: "Bezy".into(),
        resolution: (256. * 5., 256. * 3.).into(),
        ..default()
    };

    let window_plugin = WindowPlugin {
        primary_window: Some(window_config),
        ..default()
    };

    // Sequence of events to start and run the app
    // Pay attention to the order of the systems
    app.insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CurrentEditMode::default())
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin),
        )
        .add_plugins(PanCamPlugin::default())
        // When the app starts, run the setup system and spawn everything
        .add_systems(
            Startup,
            (
                setup,
                spawn_main_toolbar_debug,
                spawn_debug_text,
                spawn_debug_path,
            ),
        )
        // Update the app and get input
        .add_systems(
            Update,
            (
                handle_toolbar_mode_selection,
                update_main_toolbar_debug,
                update_current_edit_mode,
                update_coordinate_display,
                toggle_camera_controls,
            ),
        );
    app
}
