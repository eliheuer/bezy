use crate::cameras::DesignCamera;
use crate::theme::{CHECKERBOARD_COLOR, CHECKERBOARD_UNIT_SIZE};
use bevy::prelude::*;

// Component to identify checkerboard sprites
#[derive(Component)]
pub struct CheckerboardSquare;

// Main function to spawn the checkerboard
pub fn spawn_checkerboard(commands: &mut Commands) {
    // Define a very large grid size to ensure coverage
    let grid_size = 300; // Large number of squares in each dimension

    // Calculate the starting position to ensure (0,0) is in the center
    let half_size = (grid_size as f32 * CHECKERBOARD_UNIT_SIZE) / 2.0;
    let start_x = -half_size;
    let start_y = -half_size;

    let square_size = Vec2::splat(CHECKERBOARD_UNIT_SIZE);

    // Create a fixed grid of checkerboard squares
    // Using a simpler, static approach to avoid conflicts with other systems
    for x in 0..grid_size {
        for y in 0..grid_size {
            // Create the alternating pattern
            if (x % 2 == 0) != (y % 2 == 0) {
                let pos_x = start_x
                    + (x as f32 * CHECKERBOARD_UNIT_SIZE)
                    + (CHECKERBOARD_UNIT_SIZE / 2.0);
                let pos_y = start_y
                    + (y as f32 * CHECKERBOARD_UNIT_SIZE)
                    + (CHECKERBOARD_UNIT_SIZE / 2.0);

                commands.spawn((
                    Sprite {
                        color: CHECKERBOARD_COLOR,
                        custom_size: Some(square_size),
                        ..default()
                    },
                    Transform::from_xyz(pos_x, pos_y, -0.1),
                    CheckerboardSquare,
                ));
            }
        }
    }
}

// System to toggle checkerboard visibility based on camera zoom level
pub fn toggle_checkerboard_visibility(
    camera_query: Query<
        &bevy::render::camera::OrthographicProjection,
        With<DesignCamera>,
    >,
    mut checkerboard_query: Query<&mut Visibility, With<CheckerboardSquare>>,
) {
    // Default zoom level for Bevy OrthographicProjection is 1.0
    // Higher values mean zoomed out, lower values mean zoomed in
    const DEFAULT_ZOOM: f32 = 1.0;

    if let Ok(projection) = camera_query.get_single() {
        // Check if we've zoomed out past the default level
        let visible = projection.scale <= DEFAULT_ZOOM;

        // Update visibility for all checkerboard squares
        for mut visibility in checkerboard_query.iter_mut() {
            *visibility = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

// Plugin to manage the checkerboard
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        // Add system to toggle checkerboard visibility based on zoom level
        app.add_systems(Update, toggle_checkerboard_visibility);
    }
}
