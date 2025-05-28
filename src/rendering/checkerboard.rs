//! Checkerboard background system for the Bezy font editor
//!
//! This module handles the creation and visibility management of the
//! checkerboard background that provides visual context in the design space.

use crate::rendering::cameras::DesignCamera;
use crate::ui::theme::{CHECKERBOARD_COLOR, CHECKERBOARD_UNIT_SIZE};
use bevy::prelude::*;

//------------------------------------------------------------------------------
// Constants
//------------------------------------------------------------------------------

/// Number of squares in each dimension (width and height)
/// Needs to be large enough to cover the entire viewable area even when zoomed out
const GRID_SIZE: usize = 256;

/// Z-coordinate for checkerboard squares (slightly behind other elements)
const CHECKERBOARD_Z_LEVEL: f32 = -0.1;

/// Default zoom level where checkerboard visibility changes
const VISIBILITY_THRESHOLD_ZOOM: f32 = 1.0;

//------------------------------------------------------------------------------
// Components
//------------------------------------------------------------------------------

/// Component to identify checkerboard squares
///
/// This is used to query and modify only checkerboard sprites
/// when toggling visibility or performing other operations.
#[derive(Component)]
pub struct CheckerboardSquare;

//------------------------------------------------------------------------------
// Systems
//------------------------------------------------------------------------------

/// Spawns the checkerboard background
///
/// Creates a grid of alternating colored squares centered at (0,0).
/// Only the colored squares are created (not the "white" spaces between them).
pub fn spawn_checkerboard(commands: &mut Commands) {
    // Calculate the total size and starting position
    let half_size = (GRID_SIZE as f32 * CHECKERBOARD_UNIT_SIZE) / 2.0;
    let start_x = -half_size;
    let start_y = -half_size;
    let square_size = Vec2::splat(CHECKERBOARD_UNIT_SIZE);

    info!(
        "Spawning checkerboard background with grid size {}",
        GRID_SIZE
    );

    // Create checkerboard pattern
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            // Skip squares in even/even or odd/odd positions to create checkerboard pattern
            if (x % 2 == 0) == (y % 2 == 0) {
                continue;
            }

            // Calculate square position
            let position = calculate_square_position(x, y, start_x, start_y);

            // Spawn the square
            commands.spawn((
                Sprite {
                    color: CHECKERBOARD_COLOR,
                    custom_size: Some(square_size),
                    ..default()
                },
                Transform::from_xyz(
                    position.x,
                    position.y,
                    CHECKERBOARD_Z_LEVEL,
                ),
                CheckerboardSquare,
            ));
        }
    }
}

/// Calculates the position of a checkerboard square
///
/// Returns the center position of a square at the given grid coordinates.
fn calculate_square_position(
    x: usize,
    y: usize,
    start_x: f32,
    start_y: f32,
) -> Vec2 {
    let pos_x = start_x
        + (x as f32 * CHECKERBOARD_UNIT_SIZE)
        + (CHECKERBOARD_UNIT_SIZE / 2.0);
    let pos_y = start_y
        + (y as f32 * CHECKERBOARD_UNIT_SIZE)
        + (CHECKERBOARD_UNIT_SIZE / 2.0);

    Vec2::new(pos_x, pos_y)
}

/// Toggles checkerboard visibility based on camera zoom level
///
/// Shows the checkerboard when zoomed in (scale ≤ 1.0)
/// Hides the checkerboard when zoomed out (scale > 1.0)
///
/// This improves performance when zoomed out and reduces visual noise.
pub fn toggle_checkerboard_visibility(
    camera_query: Query<&OrthographicProjection, With<DesignCamera>>,
    mut checkerboard_query: Query<&mut Visibility, With<CheckerboardSquare>>,
) {
    // Get the design camera's projection
    if let Ok(projection) = camera_query.get_single() {
        // Determine if checkerboard should be visible based on zoom level
        let is_zoomed_in = projection.scale <= VISIBILITY_THRESHOLD_ZOOM;
        let visibility_state = if is_zoomed_in {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Skip visibility updates if there are no checkerboard squares
        if checkerboard_query.is_empty() {
            return;
        }

        // Update visibility for all checkerboard squares
        for mut visibility in checkerboard_query.iter_mut() {
            *visibility = visibility_state;
        }
    }
}

//------------------------------------------------------------------------------
// Plugin
//------------------------------------------------------------------------------

/// Plugin to manage the checkerboard background
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_checkerboard_visibility);
    }
}
