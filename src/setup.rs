use crate::cameras::{spawn_design_camera, spawn_grid_camera, spawn_ui_camera};
use crate::hud::spawn_hud;
use crate::ufo::load_ufo;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load UFO font
    load_ufo();

    // Spawn the main camera: order 0, layer 0
    //spawn_main_camera(&mut commands, 0, 0);

    // Spawn all HUD elements
    spawn_hud(&mut commands, &asset_server);

    // Spawn grid camera
    spawn_grid_camera(&mut commands);

    // Spawn design camera
    spawn_design_camera(&mut commands);

    // Spawn UI camera
    spawn_ui_camera(&mut commands);
}
