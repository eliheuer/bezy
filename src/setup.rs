use crate::cameras::{spawn_design_camera, spawn_grid_camera, spawn_ui_camera};
use crate::hud::spawn_hud;
use crate::ufo::load_ufo;
use bevy::prelude::*;
use rand::prelude::random;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load UFO font
    load_ufo();

    // Spawn background grid of squares
    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color =
                Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }

    // Spawn all HUD elements
    spawn_hud(&mut commands, &asset_server);

    // Spawn grid camera
    spawn_grid_camera(&mut commands);

    // Spawn design camera
    spawn_design_camera(&mut commands);

    // Spawn UI camera
    spawn_ui_camera(&mut commands);
}
