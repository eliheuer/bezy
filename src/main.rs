// A font editor made with the Bevy game engine.

mod camera;
mod draw;
mod hud;
mod setup;
mod stub;
mod theme;

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use theme::*;

use crate::camera::{camera_zoom, CameraState};
use crate::draw::draw_grid;
use crate::hud::button_system;
use crate::setup::setup;
use crate::stub::{
    animate_sprite, spawn_animated_sprite, spawn_path_points, update_sprite_position,
};

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CameraState::default())
        .add_plugins({
            let window_config = Window {
                title: "Bezy".into(),
                resolution: (1024., 768.).into(),
                ..default()
            };

            let window_plugin = WindowPlugin {
                primary_window: Some(window_config),
                ..default()
            };

            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin)
        })
        .add_systems(
            Startup,
            (setup, spawn_path_points, spawn_animated_sprite, draw_grid),
        )
        .add_systems(
            Update,
            (
                button_system,
                animate_sprite,
                update_sprite_position,
                camera_zoom,
            ),
        )
        .run();
}
