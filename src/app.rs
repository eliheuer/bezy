// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;

use crate::camera::{camera_zoom, camera_pan, CameraState};
use crate::debug_hud::{spawn_debug_hud, update_debug_text};
use crate::draw::draw_grid;
use crate::setup::setup;
use crate::stub::{
    animate_sprite, spawn_animated_sprite, spawn_debug_text, spawn_path_points,
    update_sprite_position,
};
use crate::theme::BACKGROUND_COLOR;
use crate::toolbar::{main_toolbar_button_system, handle_edit_mode, CurrentEditMode};

pub fn create_app() -> App {
    let mut app = App::new();

    let window_config = Window {
        title: "Bezy".into(),
        resolution: (1024. + 256., 768.).into(),
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
                spawn_path_points,
                spawn_animated_sprite,
                spawn_debug_text,
                spawn_debug_hud,
                draw_grid,
            ),
        )
        .add_systems(
            Update,
            (
                main_toolbar_button_system,
                animate_sprite,
                update_sprite_position,
                camera_zoom,
                camera_pan,
                update_debug_text,
                handle_edit_mode,
            ),
        );
    app
}
