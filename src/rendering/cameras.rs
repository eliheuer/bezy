//! Camera system for the Bezy font editor

use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use bevy_pancam::{PanCam, PanCamPlugin};

// Constants
const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.1;   // Don't zoom in beyond 8-unit grid usefulness
const MAX_ALLOWED_ZOOM_SCALE: f32 = 64.0;  // Stop before 256-unit grid becomes too large  
const INITIAL_ZOOM_SCALE: f32 = 1.0;       // Start at 32-unit grid level
const KEYBOARD_ZOOM_STEP: f32 = 0.9;       // Smaller number = faster zoom (0.9 = 10% change per step)

// Component to mark the main design camera
#[derive(Component)]
pub struct DesignCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin::default())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (zoom_camera, toggle_camera_controls));
    }
}

pub fn setup_camera(mut commands: Commands) {
    // Create a 2D camera with proper configuration positioned to view font design space
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        // Position camera to view the font design space (negative Y where fonts are positioned)
        Transform::from_xyz(200.0, -500.0, 1000.0).with_scale(Vec3::splat(INITIAL_ZOOM_SCALE)),
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: MIN_ALLOWED_ZOOM_SCALE,
            max_scale: MAX_ALLOWED_ZOOM_SCALE,
            ..default()
        },
        DesignCamera,
    ));
}

// Handle camera zooming with mouse wheel
fn zoom_camera(
    mut scroll_events: EventReader<MouseWheel>,
    _cameras: Query<&mut Transform, With<DesignCamera>>,
) {
    let scroll = scroll_events
        .read()
        .fold(0.0, |scroll, event| {
            scroll
                + match event.unit {
                    MouseScrollUnit::Line => event.y,
                    MouseScrollUnit::Pixel => event.y / 20.0,
                }
        });

    if scroll == 0.0 {
        return;
    }

    // We don't need to manually handle zooming here,
    // as PanCam will handle it automatically
    // This system is kept in place for future custom zoom behavior if needed
}

/// Handles keyboard shortcuts for camera control
fn toggle_camera_controls(
    mut query: Query<(&mut PanCam, &mut Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Handle zoom to cursor toggle
    if keys.just_pressed(KeyCode::KeyT) {
        for (mut pancam, _) in query.iter_mut() {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
            let status = if pancam.zoom_to_cursor {
                "enabled"
            } else {
                "disabled"
            };
            info!("Camera zoom to cursor {}", status);
        }
    }

    // Handle keyboard zoom
    let modifier_pressed = keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight);

    if !modifier_pressed {
        return;
    }

    for (_, mut transform) in query.iter_mut() {
        let current_scale = transform.scale.x;
        if keys.just_pressed(KeyCode::Equal) {
            // Zoom in
            let new_scale = (current_scale * KEYBOARD_ZOOM_STEP).max(MIN_ALLOWED_ZOOM_SCALE);
            transform.scale = Vec3::splat(new_scale);
        } else if keys.just_pressed(KeyCode::Minus) {
            // Zoom out
            let new_scale = (current_scale / KEYBOARD_ZOOM_STEP).min(MAX_ALLOWED_ZOOM_SCALE);
            transform.scale = Vec3::splat(new_scale);
        }
    }
}
