use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::*;

#[derive(Component)]
pub struct DesignCamera;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct GridCamera;

#[derive(Component)]
pub struct CoordinateDisplay;

// Main camera for the design space (bezier curves, points, etc.)
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        DesignCamera,
        RenderLayers::layer(0),
        PanCam {
            grab_buttons: vec![MouseButton::Right], // Use right mouse button for panning
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.01,
            max_scale: 50.0,
            ..default()
        },
    ));
}

// Grid camera that renders underneath the design space
pub fn spawn_grid_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        RenderLayers::layer(1),
        GridCamera,
    ));
}

// UI camera for toolbars and overlays
pub fn spawn_ui_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        RenderLayers::layer(2),
        UiCamera,
    ));
}

// Define a constant for the grid render layer
pub const GRID_LAYER: usize = 1;

pub fn update_coordinate_display(
    camera_query: Query<&GlobalTransform, With<DesignCamera>>,
    mut query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_pos = camera_transform.translation().truncate();
        for mut text in &mut query {
            text.0 = format!("X: {:.1}, Y: {:.1}", camera_pos.x, camera_pos.y);
        }
    }
}

// Camera controls
pub fn toggle_camera_controls(
    mut query: Query<&mut PanCam>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Space = Toggle Panning
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = !pancam.enabled;
        }
    }
    // T = Toggle Zoom to Cursor
    if keys.just_pressed(KeyCode::KeyT) {
        for mut pancam in &mut query {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
        }
    }
}
