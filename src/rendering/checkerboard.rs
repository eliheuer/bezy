//! Checkerboard background system for the Bezy font editor
//!
//! This module handles the creation and visibility management of the
//! checkerboard background that provides visual context in the design space.
//! Uses dynamic rendering to only show squares visible to the camera.

use bevy::prelude::*;
use bevy_pancam::PanCam;
use std::collections::HashSet;
use crate::rendering::cameras::DesignCamera;
use crate::ui::theme::{
    CHECKERBOARD_COLOR, CHECKERBOARD_UNIT_SIZE, CHECKERBOARD_SCALE_FACTOR,
    CHECKERBOARD_DARK_COLOR,
};

// Constants ------------------------------------------------------------------

/// Z-coordinate for checkerboard squares (slightly behind other elements)
const CHECKERBOARD_Z_LEVEL: f32 = -0.1;

/// Padding around visible area to prevent squares popping in/out
const VISIBILITY_PADDING: f32 = 128.0;

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
    /// Last grid size used to detect when we need to respawn all squares
    last_grid_size: Option<f32>,
}

// Systems --------------------------------------------------------------------

/// Calculates the appropriate grid size based on zoom level
///
/// This function automatically scales the grid size based on zoom level.
/// As you zoom out (higher scale values), the grid size increases to maintain performance.
fn calculate_dynamic_grid_size(zoom_scale: f32) -> f32 {
    // Calculate how much to scale the grid based on zoom level
    // Use a more gradual scaling approach to reduce frequent changes
    let scale_multiplier = if zoom_scale <= 1.0 {
        1.0
    } else {
        // For zoom > 1.0, double the grid size every time zoom doubles
        let log_scale = zoom_scale.log2().floor();
        2_f32.powf(log_scale)
    };
    
    CHECKERBOARD_UNIT_SIZE * scale_multiplier * CHECKERBOARD_SCALE_FACTOR
}

/// Updates the checkerboard based on camera position and zoom
///
/// This system dynamically spawns and despawns checkerboard squares
/// based on what's visible to the camera, providing better performance.
pub fn update_checkerboard(
    mut commands: Commands,
    mut state: ResMut<CheckerboardState>,
    camera_query: Query<(&Camera, &GlobalTransform, &PanCam), With<DesignCamera>>,
    square_query: Query<(Entity, &CheckerboardSquare)>,
) {
    // Get camera position and PanCam component
    let Ok((camera, camera_transform, pan_cam)) = camera_query.single() else {
        return;
    };
    
    // Get current zoom scale from PanCam
    let camera_scale = pan_cam.min_scale; // This is the current scale in PanCam

    // Calculate dynamic grid size based on zoom
    let current_grid_size = calculate_dynamic_grid_size(camera_scale);

    // Hide checkerboard if zoom is outside visible range
    if !is_checkerboard_visible() {
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    // Check if grid size changed significantly - if so, respawn all squares
    let grid_size_changed = state.last_grid_size.map_or(true, |last_size| {
        // Only trigger change if grid size doubled or halved to reduce flicker
        let ratio = current_grid_size / last_size;
        ratio >= 2.0 || ratio <= 0.5
    });

    if grid_size_changed {
        // Clear all existing squares and state
        despawn_all_squares(&mut commands, &mut state, &square_query);
        state.last_grid_size = Some(current_grid_size);
        state.last_camera_state = None; // Force camera update
        // Don't update visible squares this frame - let the next frame handle spawning
        return;
    }

    update_visible_squares(
        &mut commands,
        &mut state,
        camera,
        camera_transform,
        camera_scale,
        &square_query,
        current_grid_size,
    );
}

/// Checks if checkerboard should be visible at the current zoom level
///
/// Only hides checkerboard when zoomed out too far to avoid visual noise
/// and performance issues. Always visible when zoomed in for alignment.
fn is_checkerboard_visible() -> bool {
    // Always return true to make checkerboard always visible
    true
}

/// Updates which squares are visible based on camera position
fn update_visible_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    camera_scale: f32,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
    current_grid_size: f32,
) {
    let camera_pos = camera_transform.translation().truncate();
    
    // Skip update if camera hasn't moved significantly
    if let Some((last_pos, last_scale)) = state.last_camera_state {
        let pos_diff = (camera_pos - last_pos).length();
        let scale_diff = (camera_scale - last_scale).abs();
        
        // Only update if moved more than one grid unit or zoom changed >20%
        // Increased thresholds to reduce update frequency
        if pos_diff < current_grid_size 
            && scale_diff < last_scale * 0.2 
        {
            return;
        }
    }
    
    // Update camera state
    state.last_camera_state = Some((camera_pos, camera_scale));

    // Calculate visible area
    let visible_area = calculate_visible_area(camera, camera_transform, camera_scale, current_grid_size);
    let needed_squares = get_needed_squares(&visible_area, current_grid_size);

    // Despawn squares that are no longer needed
    despawn_unneeded_squares(commands, state, square_query, &needed_squares);

    // Spawn new squares that are needed
    spawn_needed_squares(commands, state, &needed_squares, current_grid_size);
}

/// Calculates the area visible to the camera
fn calculate_visible_area(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    camera_scale: f32,
    current_grid_size: f32,
) -> Rect {
    let camera_pos = camera_transform.translation().truncate();
    
    // Calculate half dimensions of the visible area using camera's viewport size
    let window_size = camera.logical_viewport_size().unwrap_or(Vec2::new(800.0, 600.0));
    let half_width = (window_size.x / 2.0) * camera_scale + VISIBILITY_PADDING;
    let half_height = (window_size.y / 2.0) * camera_scale + VISIBILITY_PADDING;
    
    // Ensure minimum visible area even when extremely zoomed in
    let min_half_size = current_grid_size * 2.0; // Show at least 4x4 grid
    let half_width = half_width.max(min_half_size);
    let half_height = half_height.max(min_half_size);

    Rect::from_center_half_size(camera_pos, Vec2::new(half_width, half_height))
}

/// Gets the set of grid positions that need checkerboard squares
fn get_needed_squares(visible_area: &Rect, current_grid_size: f32) -> HashSet<IVec2> {
    let mut needed = HashSet::new();

    // Calculate grid bounds for visible area
    let min_x = (visible_area.min.x / current_grid_size).floor() as i32;
    let max_x = (visible_area.max.x / current_grid_size).ceil() as i32;
    let min_y = (visible_area.min.y / current_grid_size).floor() as i32;
    let max_y = (visible_area.max.y / current_grid_size).ceil() as i32;

    // Add squares in checkerboard pattern (only add alternating squares)
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            needed.insert(IVec2::new(x, y));
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
    let mut entities_to_remove = Vec::new();
    
    for (entity, square) in square_query.iter() {
        if !needed_squares.contains(&square.grid_pos) {
            entities_to_remove.push((entity, square.grid_pos));
        }
    }
    
    // Despawn entities and update state
    for (entity, grid_pos) in entities_to_remove {
        // Just despawn the entity directly
        commands.entity(entity).despawn();
        state.spawned_squares.remove(&grid_pos);
    }
}

/// Spawns new squares that are needed
fn spawn_needed_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    needed_squares: &HashSet<IVec2>,
    current_grid_size: f32,
) {
    for &grid_pos in needed_squares {
        if !state.spawned_squares.contains(&grid_pos) {
            spawn_square(commands, grid_pos, current_grid_size);
            state.spawned_squares.insert(grid_pos);
        }
    }
}

/// Spawns a single checkerboard square at the given grid position
fn spawn_square(commands: &mut Commands, grid_pos: IVec2, current_grid_size: f32) {
    let world_pos = grid_to_world_position(grid_pos, current_grid_size);
    
    // Use alternating colors for checkerboard pattern
    let is_dark = (grid_pos.x + grid_pos.y) % 2 == 0;
    let color = if is_dark { CHECKERBOARD_DARK_COLOR } else { CHECKERBOARD_COLOR };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(current_grid_size)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, CHECKERBOARD_Z_LEVEL),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        CheckerboardSquare { grid_pos },
    ));
}

/// Converts grid coordinates to world position
fn grid_to_world_position(grid_pos: IVec2, current_grid_size: f32) -> Vec2 {
    Vec2::new(
        grid_pos.x as f32 * current_grid_size + current_grid_size / 2.0,
        grid_pos.y as f32 * current_grid_size + current_grid_size / 2.0,
    )
}

/// Despawns all checkerboard squares
fn despawn_all_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
) {
    // Collect all entities first to avoid iterator invalidation
    let entities_to_despawn: Vec<Entity> = square_query.iter().map(|(entity, _)| entity).collect();
    
    // Despawn all entities
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
    }
    
    // Clear the state
    state.spawned_squares.clear();
}

// Plugin ---------------------------------------------------------------------

/// Plugin to manage the checkerboard background
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CheckerboardState>()
            .add_systems(Startup, setup_initial_checkerboard)
            .add_systems(Update, update_checkerboard);
    }
}

/// Setup initial checkerboard squares on startup
fn setup_initial_checkerboard(mut commands: Commands) {
    // Create a simple 10x10 grid of squares at startup for immediate visibility
    let grid_size = CHECKERBOARD_UNIT_SIZE * CHECKERBOARD_SCALE_FACTOR;
    
    for x in -5..=5 {
        for y in -5..=5 {
            let grid_pos = IVec2::new(x, y);
            let world_pos = Vec2::new(
                x as f32 * grid_size + grid_size / 2.0,
                y as f32 * grid_size + grid_size / 2.0,
            );
            
            let is_dark = (x + y) % 2 == 0;
            let color = if is_dark { CHECKERBOARD_DARK_COLOR } else { CHECKERBOARD_COLOR };
            
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(grid_size)),
                    ..default()
                },
                Transform::from_xyz(world_pos.x, world_pos.y, CHECKERBOARD_Z_LEVEL),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                CheckerboardSquare { grid_pos },
            ));
        }
    }
} 