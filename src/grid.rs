use bevy::prelude::*;
use rand::prelude::random;

pub fn spawn_grid_of_squares(commands: &mut Commands) {
    // Configuration for the entire background grid
    let config = GridConfig {
        grid_size: 64,          // Total number of squares in each dimension
        grid_unit_size: 8.,       // Width and height of each grid unit in pixels
        color: GridColor {
            hue: 140., // Green hue in HSL color space
            saturation_range: (0.3, 0.5),
            lightness_range: (0.3, 0.6),
        },
    };

    let offset = config.grid_unit_size * config.grid_size as f32 / 2.;
    let square_size = Vec2::new(config.grid_unit_size, config.grid_unit_size);

    // Generate the grid of squares centered at (0,0)
    for x in 0..config.grid_size {
        for y in 0..config.grid_size {
            let position = Vec2::new(x as f32, y as f32) * config.grid_unit_size
                - Vec2::splat(offset);
            let color = config.color.random();

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

// Holds all configuration values for the grid
struct GridConfig {
    grid_size: u32,
    grid_unit_size: f32,
    color: GridColor,
}

// Controls the color variation of the grid squares
struct GridColor {
    hue: f32,
    saturation_range: (f32, f32),
    lightness_range: (f32, f32),
}

impl GridColor {
    fn random(&self) -> Color {
        let saturation = random_range(self.saturation_range);
        let lightness = random_range(self.lightness_range);
        Color::hsl(self.hue, saturation, lightness)
    }
}

// Helper function to generate a random value within a range
fn random_range((min, max): (f32, f32)) -> f32 {
    min + random::<f32>() * (max - min)
}
