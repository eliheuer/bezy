//! Camera system for the Bezy font editor
//!
//! This module manages the camera system that lets users view and navigate
//! the font editing workspace. It provides two main cameras:
//!
//! - **Design Camera**: Shows the actual font glyphs and editing tools
//! - **UI Camera**: Shows menus, toolbars, and interface elements
//!
//! Features include panning, zooming, and automatic centering on glyphs.

use crate::core::settings::{
    KEYBOARD_ZOOM_STEP, MAX_ALLOWED_ZOOM_SCALE, MIN_ALLOWED_ZOOM_SCALE,
};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::*;

// CONSTANTS ------------------------------------------------------------------

// Camera rendering order (lower numbers render first)
const DESIGN_CAMERA_ORDER: isize = 0;
const UI_CAMERA_ORDER: isize = 1;

// Which layer each camera renders
const DESIGN_CAMERA_LAYER: usize = 0;
const UI_CAMERA_LAYER: usize = 1;

// Settings for glyph centering
#[allow(dead_code)]
const HORIZONTAL_PADDING: f32 = 50.0;
#[allow(dead_code)]
const VERTICAL_PADDING: f32 = 100.0;
#[allow(dead_code)]
const OPTICAL_ADJUSTMENT_FACTOR: f32 = 0.03;
#[allow(dead_code)]
const MAX_ZOOM_PERCENTAGE: f32 = 0.9; // Use 90% of max zoom

// CAMERA COMPONENTS ----------------------------------------------------------

/// Marks the main design camera
///
/// This camera shows the font glyphs, bezier curves, and editing tools.
/// Users can pan and zoom this camera to navigate the design space.
#[derive(Component)]
pub struct DesignCamera;

/// Marks the UI camera
///
/// This camera shows interface elements like toolbars and menus.
/// These elements stay in place when the design camera moves.
#[derive(Component)]
pub struct UiCamera;

// CAMERA SETUP ----------------------------------------------------------------

/// Creates the main design camera
///
/// This camera is used to view and edit font glyphs. It supports
/// panning and zooming to navigate around the design space.
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // Design camera renders first (order 0)
            order: DESIGN_CAMERA_ORDER,
            ..default()
        },
        DesignCamera,
        // Only shows entities on the design layer
        RenderLayers::layer(DESIGN_CAMERA_LAYER),
        PanCam {
            // Disabled by default, enabled based on edit mode
            enabled: false,
            ..default()
        },
    ));
}

/// Creates the UI camera
///
/// This camera shows interface elements that should always stay
/// visible, regardless of how the user pans or zooms the design view.
pub fn spawn_ui_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // UI camera renders second (order 1), appearing on top
            order: UI_CAMERA_ORDER,
            ..default()
        },
        // Only shows entities on the UI layer
        RenderLayers::layer(UI_CAMERA_LAYER),
        UiCamera,
    ));
}

// CAMERA CONTROLS -------------------------------------------------------------

/// Handles keyboard shortcuts for camera control
///
/// Supported shortcuts:
/// - T: Toggle zoom-to-cursor behavior
/// - Cmd/Ctrl + Plus: Zoom in
/// - Cmd/Ctrl + Minus: Zoom out
pub fn toggle_camera_controls(
    mut query: Query<&mut PanCam>,
    mut camera_query: Query<
        &mut OrthographicProjection, 
        With<DesignCamera>
    >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    handle_zoom_to_cursor_toggle(&mut query, &keys);
    handle_zoom_hotkeys(&mut camera_query, &keys);
}

/// Handles the T key to toggle zoom-to-cursor behavior
fn handle_zoom_to_cursor_toggle(
    query: &mut Query<&mut PanCam>,
    keys: &Res<ButtonInput<KeyCode>>,
) {
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

/// Handles Cmd/Ctrl + Plus/Minus for zooming
fn handle_zoom_hotkeys(
    camera_query: &mut Query<
        &mut OrthographicProjection, 
        With<DesignCamera>
    >,
    keys: &Res<ButtonInput<KeyCode>>,
) {
    // Check if Cmd (macOS) or Ctrl (Windows/Linux) is pressed
    let modifier_pressed = keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight);

    if !modifier_pressed {
        return;
    }

    // Handle zoom in (Cmd/Ctrl + Plus)
    if keys.just_pressed(KeyCode::Equal) {
        zoom_camera(camera_query, true);
    }

    // Handle zoom out (Cmd/Ctrl + Minus)
    if keys.just_pressed(KeyCode::Minus) {
        zoom_camera(camera_query, false);
    }
}

/// Zooms the camera in or out
///
/// * `zoom_in` - true to zoom in, false to zoom out
fn zoom_camera(
    camera_query: &mut Query<
        &mut OrthographicProjection, 
        With<DesignCamera>
    >,
    zoom_in: bool,
) {
    if let Ok(mut projection) = camera_query.get_single_mut() {
        let old_scale = projection.scale;

        if zoom_in {
            // Smaller scale = more zoomed in
            projection.scale *= KEYBOARD_ZOOM_STEP;
            projection.scale = projection.scale.max(MIN_ALLOWED_ZOOM_SCALE);
            info!("Zoomed in to scale {:.3}", projection.scale);
        } else {
            // Larger scale = more zoomed out
            projection.scale /= KEYBOARD_ZOOM_STEP;
            projection.scale = projection.scale.min(MAX_ALLOWED_ZOOM_SCALE);
            info!("Zoomed out to scale {:.3}", projection.scale);
        }

        // Check if we hit a zoom limit
        if projection.scale == old_scale {
            let limit_type = if zoom_in { "minimum" } else { "maximum" };
            info!("Camera already at {} zoom limit", limit_type);
        }
    }
}
