//! Drawing algorithms and helpers

use bevy::prelude::*;

/// Spawns a grid centered in the window.
/// Creates both vertical and horizontal lines with semi-transparent gray color.
pub fn draw_grid(mut commands: Commands) {
    // Get window dimensions (using a larger value to ensure coverage)
    let window_width = 2048.0;
    let window_height = 2048.0;
    let grid_position = Vec2::new(0.0, 0.0); // Center of the window

    // Create vertical lines
    for i in -512..=512 {
        let x = grid_position.x + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(1.0, window_height)),
                ..default()
            },
            Transform::from_xyz(x * 32.0, grid_position.y, 0.0),
        ));
    }

    // Create horizontal lines
    for i in -512..=512 {
        // Increased range
        let y = grid_position.y + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(window_width, 1.0)),
                ..default()
            },
            Transform::from_xyz(grid_position.x, y * 32.0, 0.0),
        ));
    }
}
