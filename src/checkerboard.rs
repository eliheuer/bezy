use crate::theme::{
    CHECKERBOARD_COLOR, CHECKERBOARD_SIZE, CHECKERBOARD_UNIT_SIZE,
};
use bevy::prelude::*;

pub fn spawn_checkerboard(commands: &mut Commands) {
    let config = CheckerboardConfig {
        size: CHECKERBOARD_SIZE, // Use constant from theme
        square_size: CHECKERBOARD_UNIT_SIZE, // Use constant from theme
        color: CHECKERBOARD_COLOR, // Use color from theme
    };

    // Calculate where the checkerboard center should be
    // We want a square's corner to be at (0,0), not its center
    let half_total_size = config.square_size * config.size as f32 / 2.0;

    // Shift the checkerboard by half a square so the corner is at (0,0) instead of the center
    let offset_x = half_total_size - config.square_size / 2.0;
    let offset_y = half_total_size - config.square_size / 2.0;

    let square_size = Vec2::new(config.square_size, config.square_size);

    for x in 0..config.size {
        for y in 0..config.size {
            // Only spawn sprites for alternating squares (checkerboard pattern)
            // Skip squares where (x+y) is even, or you could use odd - whatever looks better
            if (x + y) % 2 == 1 {
                // Position calculation:
                // 1. Calculate position based on checkerboard index (x,y)
                // 2. Shift by the offset to center the checkerboard
                // 3. Position is the center of each square (for the sprite)
                let position = Vec2::new(
                    x as f32 * config.square_size - offset_x,
                    y as f32 * config.square_size - offset_y,
                );

                commands.spawn((
                    Sprite {
                        color: config.color,
                        custom_size: Some(square_size),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.),
                ));
            }
            // The other squares are not drawn, letting the background color show through
        }
    }
}

struct CheckerboardConfig {
    size: u32,
    square_size: f32,
    color: Color,
}
