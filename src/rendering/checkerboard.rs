//! Checkerboard background system for the Bezy font editor
//!
//! This module handles the creation and visibility management of the
//! checkerboard background that provides visual context in the design space.
//! Uses dynamic rendering to only show squares visible to the camera.

use crate::rendering::cameras::DesignCamera;
use crate::ui::theme::{CHECKERBOARD_COLOR, CHECKERBOARD_UNIT_SIZE};
use bevy::prelude::*;
use std::collections::HashSet;

// Constants ------------------------------------------------------------------

/// Z-coordinate for checkerboard squares (slightly behind other elements)
const CHECKERBOARD_Z_LEVEL: f32 = -0.1;

/// Padding around visible area to prevent squares popping in/out
const VISIBILITY_PADDING: f32 = 128.0;

/// Minimum zoom level where checkerboard is visible
const MIN_VISIBILITY_ZOOM: f32 = 0.1;

/// Maximum zoom level where checkerboard is visible  
const MAX_VISIBILITY_ZOOM: f32 = 8.0;

// Components -----------------------------------------------------------------

/// Component to identify checkerboard squares
///
/// This is used to query and modify only checkerboard sprites
/// when toggling visibility or performing other operations.
#[derive(Component)]
pub struct CheckerboardSquare {
    /// Grid coordinates of this square
    pub grid_pos: IVec2,
}

/// Resource to track currently spawned checkerboard squares
#[derive(Resource, Default)]
pub struct CheckerboardState {
    /// Set of grid positions that currently have spawned squares
    spawned_squares: HashSet<IVec2>,
    /// Last camera position and scale to detect significant changes
    last_camera_state: Option<(Vec2, f32)>,
}

// Systems --------------------------------------------------------------------

/// Updates the checkerboard based on camera position and zoom
///
/// This system dynamically spawns and despawns checkerboard squares
/// based on what's visible to the camera, providing better performance.
pub fn update_checkerboard(
    mut commands: Commands,
    mut state: ResMut<CheckerboardState>,
    camera_query: Query<
        (&Transform, &OrthographicProjection),
        With<DesignCamera>,
    >,
    square_query: Query<(Entity, &CheckerboardSquare)>,
) {
    let Ok((camera_transform, projection)) = camera_query.get_single() else {
        return;
    };

    // Hide checkerboard if zoom is outside visible range
    if !is_checkerboard_visible(projection.scale) {
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    update_visible_squares(
        &mut commands,
        &mut state,
        camera_transform,
        projection,
        &square_query,
    );
}

/// Checks if checkerboard should be visible at the current zoom level
fn is_checkerboard_visible(zoom_scale: f32) -> bool {
    zoom_scale >= MIN_VISIBILITY_ZOOM && zoom_scale <= MAX_VISIBILITY_ZOOM
}

/// Updates which squares are visible based on camera position
fn update_visible_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    camera_transform: &Transform,
    projection: &OrthographicProjection,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
) {
    let camera_pos = camera_transform.translation.truncate();
    let camera_scale = projection.scale;
    
    // Skip update if camera hasn't moved significantly
    if let Some((last_pos, last_scale)) = state.last_camera_state {
        let pos_diff = (camera_pos - last_pos).length();
        let scale_diff = (camera_scale - last_scale).abs();
        
        // Only update if moved more than half a grid unit or zoom changed >10%
        if pos_diff < CHECKERBOARD_UNIT_SIZE * 0.5 
            && scale_diff < last_scale * 0.1 
        {
            return;
        }
    }
    
    // Update camera state
    state.last_camera_state = Some((camera_pos, camera_scale));

    // Calculate visible area
    let visible_area = calculate_visible_area(camera_transform, projection);
    let needed_squares = get_needed_squares(&visible_area);

    // Despawn squares that are no longer needed
    despawn_unneeded_squares(commands, state, square_query, &needed_squares);

    // Spawn new squares that are needed
    spawn_needed_squares(commands, state, &needed_squares);
}

/// Calculates the area visible to the camera
fn calculate_visible_area(
    camera_transform: &Transform,
    projection: &OrthographicProjection,
) -> Rect {
    let camera_pos = camera_transform.translation.truncate();
    let half_width =
        projection.area.width() * projection.scale / 2.0 + VISIBILITY_PADDING;
    let half_height =
        projection.area.height() * projection.scale / 2.0 + VISIBILITY_PADDING;

    Rect::from_center_half_size(camera_pos, Vec2::new(half_width, half_height))
}

/// Gets the set of grid positions that need checkerboard squares
fn get_needed_squares(visible_area: &Rect) -> HashSet<IVec2> {
    let mut needed = HashSet::new();

    // Calculate grid bounds for visible area
    let min_x = (visible_area.min.x / CHECKERBOARD_UNIT_SIZE).floor() as i32;
    let max_x = (visible_area.max.x / CHECKERBOARD_UNIT_SIZE).ceil() as i32;
    let min_y = (visible_area.min.y / CHECKERBOARD_UNIT_SIZE).floor() as i32;
    let max_y = (visible_area.max.y / CHECKERBOARD_UNIT_SIZE).ceil() as i32;

    // Add squares in checkerboard pattern
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            // Only create squares in checkerboard pattern (skip white squares)
            if (x % 2 == 0) != (y % 2 == 0) {
                needed.insert(IVec2::new(x, y));
            }
        }
    }

    needed
}

/// Despawns squares that are no longer needed
fn despawn_unneeded_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
    needed_squares: &HashSet<IVec2>,
) {
    for (entity, square) in square_query.iter() {
        if !needed_squares.contains(&square.grid_pos) {
            commands.entity(entity).despawn();
            state.spawned_squares.remove(&square.grid_pos);
        }
    }
}

/// Spawns new squares that are needed
fn spawn_needed_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    needed_squares: &HashSet<IVec2>,
) {
    for &grid_pos in needed_squares {
        if !state.spawned_squares.contains(&grid_pos) {
            spawn_square(commands, grid_pos);
            state.spawned_squares.insert(grid_pos);
        }
    }
}

/// Spawns a single checkerboard square at the given grid position
fn spawn_square(commands: &mut Commands, grid_pos: IVec2) {
    let world_pos = grid_to_world_position(grid_pos);

    commands.spawn((
        Sprite {
            color: CHECKERBOARD_COLOR,
            custom_size: Some(Vec2::splat(CHECKERBOARD_UNIT_SIZE)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, CHECKERBOARD_Z_LEVEL),
        CheckerboardSquare { grid_pos },
    ));
}

/// Converts grid coordinates to world position
fn grid_to_world_position(grid_pos: IVec2) -> Vec2 {
    Vec2::new(
        grid_pos.x as f32 * CHECKERBOARD_UNIT_SIZE
            + CHECKERBOARD_UNIT_SIZE / 2.0,
        grid_pos.y as f32 * CHECKERBOARD_UNIT_SIZE
            + CHECKERBOARD_UNIT_SIZE / 2.0,
    )
}

/// Despawns all checkerboard squares
fn despawn_all_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
) {
    for (entity, _) in square_query.iter() {
        commands.entity(entity).despawn();
    }
    state.spawned_squares.clear();
}

// Plugin ---------------------------------------------------------------------

/// Plugin to manage the checkerboard background
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CheckerboardState>()
            .add_systems(Update, update_checkerboard);
    }
}
