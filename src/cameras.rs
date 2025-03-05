use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::*;

#[derive(Component)]
pub struct DesignCamera;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct CoordinateDisplay;

// Main camera for the design space (bezier curves, points, etc.)
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0, // Main camera renders in the middle
            ..default()
        },
        DesignCamera,
        RenderLayers::layer(0), // Main design layer
        PanCam {
            enabled: false, // Disabled by default, will be controlled by the edit mode
            ..default()
        },
    ));
}

// UI camera for toolbars and overlays
pub fn spawn_ui_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // UI camera renders on top
            ..default()
        },
        RenderLayers::layer(1), // UI layer
        UiCamera,
    ));
}

pub fn update_coordinate_display(
    camera_query: Query<&GlobalTransform, With<DesignCamera>>,
    mut query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_pos = camera_transform.translation().truncate();
        for mut text in &mut query {
            text.0 = format!(
                "Camera Location: {} {}",
                camera_pos.x.round(),
                camera_pos.y.round()
            );
        }
    }
}

// Camera controls
pub fn toggle_camera_controls(
    mut query: Query<&mut PanCam>,
    keys: Res<ButtonInput<KeyCode>>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
) {
    // Space = Toggle Panning, but only if we're in Pan mode
    if keys.just_pressed(KeyCode::Space)
        && matches!(current_mode.0, crate::edit_mode_toolbar::EditMode::Pan)
    {
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
