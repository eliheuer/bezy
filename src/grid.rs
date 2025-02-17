use bevy::prelude::*;

// Gray with an "a" the american way
fn rgb_linear_from_gray(gray: f32) -> Color {
    Color::linear_rgb(gray, gray, gray)
}

pub fn spawn_grid_of_squares(commands: &mut Commands) {
    let config = GridConfig {
        grid_size: 128,      // Total number of squares in each dimension
        grid_unit_size: 16., // Width and height of each grid unit in pixels
        colors: GridColors {
            light: rgb_linear_from_gray(0.03),
            dark: rgb_linear_from_gray(0.015),
        },
    };

    let offset = config.grid_unit_size * config.grid_size as f32 / 2.;
    let square_size = Vec2::new(config.grid_unit_size, config.grid_unit_size);

    for x in 0..config.grid_size {
        for y in 0..config.grid_size {
            let position = Vec2::new(x as f32, y as f32)
                * config.grid_unit_size
                - Vec2::splat(offset);

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
