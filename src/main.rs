// A font editor made with the Bevy game engine.

mod setup;
mod stub;
mod theme;
mod hud;
mod draw;
mod camera;

use crate::setup::setup;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use theme::*;
use crate::stub::{spawn_path_points, animate_sprite, spawn_animated_sprite, update_sprite_position};
use crate::hud::button_system;
use crate::draw::draw_grid;
use crate::camera::{CameraState, camera_zoom};

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        // .insert_resource(WinitSettings {
        //     focused_mode: bevy::winit::UpdateMode::Continuous,
        //     unfocused_mode: bevy::winit::UpdateMode::Continuous,
        //     ..default()
        // })
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
            (
                setup,
                spawn_path_points,
                spawn_animated_sprite,
                draw_grid,
            ),
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
