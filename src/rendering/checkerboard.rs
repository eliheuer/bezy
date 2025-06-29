//! Checkerboard background grid system for the Bezy font editor GUI.
//!
//! This module handles the creation and visibility management of the
//! checkerboard grid that provides visual context in the design space.
//! Uses dynamic rendering to only show squares visible to the camera.

use crate::rendering::cameras::DesignCamera;
use crate::ui::theme::{
    CHECKERBOARD_COLOR, CHECKERBOARD_UNIT_SIZE, CHECKERBOARD_SCALE_FACTOR,
    CHECKERBOARD_MAX_ZOOM_VISIBLE, CHECKERBOARD_ENABLED_BY_DEFAULT,
    WINDOW_WIDTH, WINDOW_HEIGHT,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_pancam::PanCam;
use std::collections::HashSet;

/// Z-coordinate for checkerboard squares (slightly behind other elements)
const CHECKERBOARD_Z_LEVEL: f32 = 0.1; // Render above background but below main

/// Padding around visible area to prevent squares popping in/out
const VISIBILITY_PADDING: f32 = 512.0;

/// Minimum zoom level where checkerboard is visible (very zoomed out)
/// Lower values = more zoomed out before hiding checkerboard
const MIN_VISIBILITY_ZOOM: f32 = 0.01;

// Resources -------------------

/// Resource to control whether the checkerboard is enabled
/// This allows complete disabling for performance when not needed
#[derive(Resource, Clone)]
pub struct CheckerboardEnabled {
    pub enabled: bool,
}

impl Default for CheckerboardEnabled {
    fn default() -> Self {
        Self {
            enabled: CHECKERBOARD_ENABLED_BY_DEFAULT,
        }
    }
}

// Components ------------------

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

// Systems -----------------

/// Calculates the appropriate grid size based on zoom level
///
/// This function provides a stepped scaling system where grid squares get 
/// larger as you zoom out, maintaining visual clarity and usefulness at all 
/// zoom levels. Grid sizes: 8 -> 16 -> 32 -> 64 -> 128 units
fn calculate_dynamic_grid_size(zoom_scale: f32) -> f32 {
    // Define grid sizes for different zoom levels - tighter thresholds for 
    // more responsiveness. These provide clear, useful grid units for font 
    // design work
    if zoom_scale <= 1.0 {
        // Very zoomed in - small, precise grid
        8.0
    } else if zoom_scale <= 2.0 {
        // Slightly zoomed in - small-medium grid
        16.0
    } else if zoom_scale <= 3.0 {
        // Normal zoom - medium grid good for typical editing
        32.0
    } else if zoom_scale <= 4.0 {
        // Moderately zoomed out - larger grid
        64.0
    } else {
        // Very zoomed out - large grid
        128.0
    }
}

/// Updates the checkerboard based on camera position and zoom
///
/// This system dynamically spawns and despawns checkerboard squares
/// based on what's visible to the camera, providing better performance.
pub fn update_checkerboard(
    mut commands: Commands,
    mut state: ResMut<CheckerboardState>,
    camera_query: Query<
        (&Transform, &Projection, Option<&PanCam>), 
        With<DesignCamera>
    >,
    square_query: Query<(Entity, &CheckerboardSquare)>,
    checkerboard_enabled: Res<CheckerboardEnabled>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // If checkerboard is disabled, despawn all squares and return early for 
    // performance
    if !checkerboard_enabled.enabled {
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    let Ok((camera_transform, projection, _pancam_opt)) = 
        camera_query.single() else {
        return;
    };

    // Get camera scale from OrthographicProjection (this is what PanCam 
    // actually modifies for zoom)
    let projection_scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0, // Default for non-orthographic projections
    };
    let transform_scale = camera_transform.scale.x;
    
    // Use projection scale for grid calculation (this is the real zoom level)
    let camera_scale = projection_scale;
    let current_grid_size = calculate_dynamic_grid_size(camera_scale);
    
    // Debug logging to help troubleshoot zoom issues (only log when scale 
    // changes significantly)
    let significant_scale_change = state.last_camera_state.map_or(true, 
        |(_, last_scale)| {     
            (camera_scale / last_scale - 1.0).abs() > 0.05 // Log if scale 
                                                           // changes by >5%
        });
    
    if significant_scale_change {
        info!("Camera debug: projection_scale={:.3}, transform_scale={:.3}, \
               using={:.3}, transform=({:.1}, {:.1}, {:.1}), grid_size={:.0}", 
              projection_scale, transform_scale, camera_scale, 
              camera_transform.translation.x, camera_transform.translation.y, 
              camera_transform.translation.z, current_grid_size);
    }

    // Hide checkerboard if zoom is outside visible range
    if !is_checkerboard_visible(camera_scale) {
        info!("Checkerboard not visible at current zoom scale: {:.3} \
               (range: {:.3} to {:.1})", 
               camera_scale, MIN_VISIBILITY_ZOOM, 
               CHECKERBOARD_MAX_ZOOM_VISIBLE);
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    // Debug logging for checkerboard (only log once per grid size change)
    
    if state.last_grid_size.is_none() || 
       state.last_grid_size.unwrap() != current_grid_size || 
       significant_scale_change {
        info!("Checkerboard: camera_scale={:.3}, grid_size={:.0} units, \
               camera_pos=({:.1}, {:.1})", 
              camera_scale, current_grid_size, 
              camera_transform.translation.x, 
              camera_transform.translation.y);
        
        // Show which grid level we're at
        let grid_level = match current_grid_size as u32 {
            8 => "Fine (8 units)",
            16 => "Small (16 units)",
            32 => "Normal (32 units)", 
            64 => "Medium (64 units)",
            128 => "Large (128 units)",
            _ => "Custom",
        };
        info!("  grid level: {}", grid_level);
        info!("  checkerboard visible: {}", 
              is_checkerboard_visible(camera_scale));
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
        // Don't update visible squares this frame - let the next frame handle 
        // spawning
        return;
    }

    // Get actual window size for visible area calculation
    let window_size = if let Ok(window) = window_query.single() {
        Vec2::new(window.resolution.width(), window.resolution.height())
    } else {
        // Fallback to theme constants if window query fails
        Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)
    };

    update_visible_squares(
        &mut commands,
        &mut state,
        camera_transform,
        camera_scale,
        &square_query,
        current_grid_size,
        window_size,
    );
}

/// Checks if checkerboard should be visible at the current zoom level
///
/// The checkerboard is always visible within our defined zoom range
/// since the dynamic scaling system ensures it's always useful.
fn is_checkerboard_visible(zoom_scale: f32) -> bool {
    // Always show checkerboard within our zoom limits
    // The dynamic grid scaling ensures it's always useful
    zoom_scale >= 0.05 && zoom_scale <= 100.0
}

/// Updates which squares are visible based on camera position
fn update_visible_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    camera_transform: &Transform,
    camera_scale: f32,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
    current_grid_size: f32,
    window_size: Vec2,
) {
    let camera_pos = camera_transform.translation.truncate();
    
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
    let visible_area = calculate_visible_area(
        camera_transform, 
        camera_scale, 
        current_grid_size, 
        window_size
    );
   let needed_squares = get_needed_squares(&visible_area, current_grid_size);

    // Debug logging for visible area (only when grid size changes)
    if state.last_grid_size.is_none() || 
       state.last_grid_size.unwrap() != current_grid_size {
        info!("Design space grid: visible=({:.0}, {:.0}) to ({:.0}, {:.0}), \
               {} squares", 
              visible_area.min.x, visible_area.min.y, 
              visible_area.max.x, visible_area.max.y, needed_squares.len());
    }

    // Despawn squares that are no longer needed
    despawn_unneeded_squares(commands, state, square_query, &needed_squares);

    // Spawn new squares that are needed
    spawn_needed_squares(commands, state, &needed_squares, current_grid_size);
}

/// Calculates the area visible to the camera
fn calculate_visible_area(
    camera_transform: &Transform,
    camera_scale: f32,
    current_grid_size: f32,
    window_size: Vec2,
) -> Rect {
    let camera_pos = camera_transform.translation.truncate();
    
    // Use a very generous approach to ensure full screen coverage
    // Make the area much larger than the window to guarantee coverage
    let screen_width = window_size.x;
    let screen_height = window_size.y;
    
    // Use a much more aggressive multiplier to ensure full coverage
    // The formula: visible_area = window_size * camera_scale * multiplier + 
    // huge_padding
    let aggressive_multiplier = 5.0; // Much more aggressive coverage
    let huge_padding = 2000.0 * camera_scale.max(1.0); // Very large padding
    
    let half_width = (screen_width * camera_scale * aggressive_multiplier) 
        / 2.0 + huge_padding;
    let half_height = (screen_height * camera_scale * aggressive_multiplier) 
        / 2.0 + huge_padding;
    
    // Ensure minimum coverage - at least 20 grid squares in each direction  
    let min_half_size = current_grid_size * 20.0;
    let final_half_width = half_width.max(min_half_size);
    let final_half_height = half_height.max(min_half_size);

    info!("Visible area calc: window=({:.0}x{:.0}), camera_scale={:.3}, \
           half_size=({:.0}, {:.0})", 
           screen_width, screen_height, camera_scale, 
           final_half_width, final_half_height);

    Rect::from_center_half_size(
        camera_pos, 
        Vec2::new(final_half_width, final_half_height)
    )
}

/// Gets the set of grid positions that need checkerboard squares
fn get_needed_squares(
    visible_area: &Rect, 
    current_grid_size: f32
) -> HashSet<IVec2> {
    let mut needed = HashSet::new();

    // Calculate grid bounds for visible area
    let min_x = (visible_area.min.x / current_grid_size).floor() as i32;
    let max_x = (visible_area.max.x / current_grid_size).ceil() as i32;
    let min_y = (visible_area.min.y / current_grid_size).floor() as i32;
    let max_y = (visible_area.max.y / current_grid_size).ceil() as i32;

    // Add squares in checkerboard pattern
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            // Only add squares that should be visible in checkerboard pattern
            if (x + y) % 2 == 0 {
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
    let mut to_remove = Vec::new();
    
    for (entity, square) in square_query.iter() {
        if !needed_squares.contains(&square.grid_pos) {
            commands.entity(entity).despawn();
            to_remove.push(square.grid_pos);
        }
    }
    
    for pos in to_remove {
        state.spawned_squares.remove(&pos);
    }
}

/// Spawns new squares that are needed
fn spawn_needed_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    needed_squares: &HashSet<IVec2>,
    current_grid_size: f32,
) {
    let new_squares: Vec<_> = needed_squares.iter()
        .filter(|pos| !state.spawned_squares.contains(pos))
        .copied()
        .collect();
        
    if !new_squares.is_empty() {
        debug!("Spawning {} new checkerboard squares (grid size: {:.1})", 
               new_squares.len(), current_grid_size);
    }
    
    for grid_pos in new_squares {
        spawn_square(commands, grid_pos, current_grid_size);
        state.spawned_squares.insert(grid_pos);
    }
}

/// Spawns a single checkerboard square at the given grid position
fn spawn_square(
    commands: &mut Commands, 
    grid_pos: IVec2, 
    current_grid_size: f32
) {
    let world_pos = grid_to_world_position(grid_pos, current_grid_size);
    
    // Debug log the first few squares spawned to verify design space alignment
    static mut SPAWN_COUNT: usize = 0;
    unsafe {
        if SPAWN_COUNT < 3 {
            info!("Design space square {} at grid=({}, {}), \
                   world=({:.0}, {:.0}), size={:.0}", 
                  SPAWN_COUNT, grid_pos.x, grid_pos.y, 
                  world_pos.x, world_pos.y, current_grid_size);
            SPAWN_COUNT += 1;
        }
    }
    
    commands.spawn((
        CheckerboardSquare { grid_pos },
        Sprite {
            color: CHECKERBOARD_COLOR,
            custom_size: Some(Vec2::splat(current_grid_size)),
            ..default()
        },
        Transform::from_translation(world_pos.extend(CHECKERBOARD_Z_LEVEL)),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

/// Converts grid position to world position aligned to design space
/// The grid is positioned so that (0,0) - the font baseline/left sidebearing 
/// intersection - falls exactly at the intersection of four grid squares, 
/// making unit counting accurate
fn grid_to_world_position(grid_pos: IVec2, current_grid_size: f32) -> Vec2 {
    Vec2::new(
        (grid_pos.x as f32 + 0.5) * current_grid_size,
        (grid_pos.y as f32 + 0.5) * current_grid_size,
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
    
    // Also clear grid size to force recalculation
    state.last_grid_size = None;
}
 
#[derive(Default)]
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CheckerboardState>()
            .init_resource::<CheckerboardEnabled>()
            .add_systems(Update, update_checkerboard);
    }
} 
