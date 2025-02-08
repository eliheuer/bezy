// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;

use crate::camera::{camera_pan, camera_zoom, CameraState};
use crate::debug_hud::{spawn_debug_hud, update_debug_text};
use crate::setup::setup;
use crate::stub::{debug_points_basic_spawn, spawn_debug_text};
use crate::theme::BACKGROUND_COLOR;
use crate::toolbar::{handle_edit_mode, main_toolbar_button_system, CurrentEditMode};

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

    app.insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CameraState::default())
        .insert_resource(CurrentEditMode::default())
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin),
        )
        .add_systems(
            Startup,
            (
                setup,
                debug_points_basic_spawn,
                spawn_debug_text,
                spawn_debug_hud,
            ),
        )
        .add_systems(
            Update,
            (
                main_toolbar_button_system,
                camera_zoom,
                camera_pan,
                update_debug_text,
                handle_edit_mode,
            ),
        );
    app
}
