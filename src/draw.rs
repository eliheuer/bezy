//! Drawing algorithms and helpers

use crate::design_space::DesignPoint;
use bevy::prelude::*;

#[derive(Component)]
pub struct GridLine;

/// Calculates the appropriate grid resolution based on zoom level
fn calculate_grid_resolution(zoom: f32) -> f32 {
    // Base resolution is 32 units at default zoom (zoom = 1.0)
    let base_resolution = 32.0;

    // Calculate the zoom factor
    // We want to double the grid spacing as we zoom out
    let zoom_factor = (1.0 / zoom).log2().floor();

    // At high zoom levels (>= 2.0), divide the base resolution
    if zoom >= 2.0 {
        base_resolution / 8.0
    } else {
        base_resolution * 2.0f32.powf(zoom_factor)
    }
}

/// Initial grid setup that runs on startup
pub fn draw_grid(mut commands: Commands) {
    // Initial grid will be updated immediately by update_grid system
    // This is just to ensure we have a system for startup
}

/// Spawns or updates the grid based on current camera transform
pub fn update_grid(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    existing_grid: Query<Entity, With<GridLine>>,
) {
    // Remove existing grid lines
    for entity in existing_grid.iter() {
        commands.entity(entity).despawn();
    }

    let camera_transform = camera_query.single();
    let zoom = camera_transform.scale.x;

    // Calculate grid resolution based on zoom level
    let grid_spacing = calculate_grid_resolution(zoom);

    // Calculate visible area in design space
    // Using a larger value to ensure coverage of the visible area
    let visible_width = 2048.0 / zoom;
    let visible_height = 2048.0 / zoom;
    let line_thickness = 1.0 / zoom;

    // Calculate grid line positions
    let min_x = (-visible_width / 2.0).floor() as i32;
    let max_x = (visible_width / 2.0).ceil() as i32;
    let min_y = (-visible_height / 2.0).floor() as i32;
    let max_y = (visible_height / 2.0).ceil() as i32;

    // Create vertical lines
    for i in min_x..=max_x {
        let x = i as f32 * grid_spacing;
        let design_point = DesignPoint::new(x, 0.0);
        let screen_pos = design_point.to_screen_space(camera_transform);

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(line_thickness, visible_height)),
                ..default()
            },
            Transform::from_xyz(screen_pos.x, 0.0, 0.0),
        ));
    }

    // Create horizontal lines
    for i in min_y..=max_y {
        let y = i as f32 * grid_spacing;
        let design_point = DesignPoint::new(0.0, y);
        let screen_pos = design_point.to_screen_space(camera_transform);

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(visible_width, line_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, screen_pos.y, 0.0),
        ));
    }
}
