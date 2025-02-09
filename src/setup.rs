use crate::hud::spawn_hud;
use crate::ufo::load_ufo;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load UFO font
    load_ufo();

    // Spawn UI camera
    commands.spawn(Camera2d);

    // Spawn all HUD elements
    spawn_hud(&mut commands, &asset_server);
}
