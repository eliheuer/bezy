use crate::cameras::{spawn_design_camera, spawn_ui_camera};
use crate::grid::spawn_grid_of_squares;
use crate::hud::spawn_hud;
use crate::ufo::load_ufo;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load UFO font
    load_ufo();

    // Spawn background grid of squares
    spawn_grid_of_squares(&mut commands);

    // Spawn all HUD elements
    spawn_hud(&mut commands, &asset_server);

    // Spawn design camera
    spawn_design_camera(&mut commands);

    // Spawn UI camera
    spawn_ui_camera(&mut commands);
}
