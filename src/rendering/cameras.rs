//! Camera system for the Bezy font editor

use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use bevy_pancam::{PanCam, PanCamPlugin};

// Constants
const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.01;
const MAX_ALLOWED_ZOOM_SCALE: f32 = 32.0;
const INITIAL_ZOOM_SCALE: f32 = 1.0;
const KEYBOARD_ZOOM_STEP: f32 = 0.9; // Smaller number = faster zoom (0.9 = 10% change per step)

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

fn setup_camera(mut commands: Commands) {
    // Create a 2D camera with proper configuration
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0),
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
    mut query: Query<&mut PanCam>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Toggle zoom-to-cursor behavior with T key
    if keys.just_pressed(KeyCode::KeyT) {
        for mut pancam in query.iter_mut() {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
            let status = if pancam.zoom_to_cursor {
                "enabled"
            } else {
                "disabled"
            };
            info!("Camera zoom to cursor {}", status);
        }
    }
}
