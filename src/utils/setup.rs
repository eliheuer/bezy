use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Note: Checkerboard is now dynamically managed by the CheckerboardPlugin
    // and doesn't need manual spawning at startup

    // Spawn design camera
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 0.0, 100.0),
        Name::new("DesignCamera"),
    ));

    // Spawn UI camera  
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 0.0, 200.0),
        Name::new("UICamera"),
    ));
} 