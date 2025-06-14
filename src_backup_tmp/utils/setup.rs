use crate::rendering::cameras::{spawn_design_camera, spawn_ui_camera};
use crate::ui::hud::spawn_hud;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Note: Checkerboard is now dynamically managed by the CheckerboardPlugin
    // and doesn't need manual spawning at startup

    // Spawn all HUD elements
    spawn_hud(&mut commands, &asset_server);

    // Spawn design camera
    spawn_design_camera(&mut commands);

    // Spawn UI camera
    spawn_ui_camera(&mut commands);
}
