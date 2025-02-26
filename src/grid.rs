use crate::theme::{
    GRID_COLOR_DARK, GRID_COLOR_LIGHT, GRID_SIZE, GRID_UNIT_SIZE,
};
use bevy::prelude::*;

pub fn spawn_grid_of_squares(commands: &mut Commands) {
    let config = GridConfig {
        grid_size: GRID_SIZE,           // Use constant from theme
        grid_unit_size: GRID_UNIT_SIZE, // Use constant from theme
        colors: GridColors {
            light: GRID_COLOR_LIGHT, // Use color from theme
            dark: GRID_COLOR_DARK,   // Use color from theme
        },
    };

    // Calculate where the grid center should be
    // We want a grid unit's corner to be at (0,0), not its center
    let half_total_size = config.grid_unit_size * config.grid_size as f32 / 2.0;

    // Shift the grid by half a grid unit so the corner is at (0,0) instead of the center
    let grid_offset_x = half_total_size - config.grid_unit_size / 2.0;
    let grid_offset_y = half_total_size - config.grid_unit_size / 2.0;

    let square_size = Vec2::new(config.grid_unit_size, config.grid_unit_size);

    for x in 0..config.grid_size {
        for y in 0..config.grid_size {
            // Position calculation:
            // 1. Calculate position based on grid index (x,y)
            // 2. Shift by the offset to center the grid
            // 3. Position is the center of each square (for the sprite)
            let position = Vec2::new(
                x as f32 * config.grid_unit_size - grid_offset_x,
                y as f32 * config.grid_unit_size - grid_offset_y,
            );

            // Determine the color of the square
            let is_dark = (x + y) % 2 == 0;
            let color = if is_dark {
                config.colors.dark
            } else {
                config.colors.light
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(square_size),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, 0.),
            ));
        }
    }
}

struct GridConfig {
    grid_size: u32,
    grid_unit_size: f32,
    colors: GridColors,
}

struct GridColors {
    light: Color,
    dark: Color,
}
