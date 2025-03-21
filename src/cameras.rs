//! Camera system for the Bezy font editor
//!
//! This module contains camera components, camera creation, camera controls,
//! and utility functions for managing the camera system.
//!
//! The application uses two cameras:
//! - The `DesignCamera` for the main editing area where font glyphs are displayed and edited
//! - The `UiCamera` for user interface elements like toolbars and overlays
//!
//! Camera controls include panning, zooming, and centering on glyphs.

use crate::theme::{CAMERA_MIN_SCALE, CAMERA_ZOOM_FACTOR};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::*;

/// Component that marks the main design camera
///
/// This camera renders the design space where glyphs, points, and
/// other editing elements are displayed.
#[derive(Component)]
pub struct DesignCamera;

/// Component that marks the UI camera
///
/// This camera renders UI elements like toolbars and overlays
/// on a separate layer from the design elements.
#[derive(Component)]
pub struct UiCamera;

/// Spawns the main camera for the design space
///
/// This camera is used to view and interact with bezier curves, points, and other design elements.
/// It is configured with PanCam for panning and zooming functionality.
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0, // Main camera renders first (lower value = earlier render)
            ..default()
        },
        DesignCamera,
        RenderLayers::layer(0), // Main design elements are on layer 0
        PanCam {
            enabled: false, // Disabled by default, will be enabled based on edit mode
            ..default()
        },
    ));
}

/// Spawns the UI camera for toolbars and overlays
///
/// This camera is used for UI elements that should always be visible
/// regardless of panning/zooming of the design view.
pub fn spawn_ui_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // UI camera renders after design camera (on top)
            ..default()
        },
        RenderLayers::layer(1), // UI elements are on layer 1
        UiCamera,
    ));
}

/// Handles camera controls based on keyboard input
///
/// Keyboard shortcuts:
/// - Space: Hold to temporarily enable camera panning (works in any edit mode)
/// - T: Toggle zoom-to-cursor behavior
pub fn toggle_camera_controls(
    mut query: Query<&mut PanCam>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Spacebar handling for temporary panning in any edit mode
    // When pressed, enable panning
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = true;
            info!("Camera panning enabled (spacebar held)");

            // Future implementation:
            // This would be where we'd add code to:
            // 1. Draw outline with solid fill
            // 2. Hide emsquare, points, and handles
        }
    }

    // When released, disable panning
    if keys.just_released(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = false;
            info!("Camera panning disabled (spacebar released)");

            // Future implementation:
            // This would be where we'd restore the normal view
        }
    }

    // T = Toggle Zoom to Cursor
    if keys.just_pressed(KeyCode::KeyT) {
        for mut pancam in &mut query {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
            info!(
                "Camera zoom to cursor {}",
                if pancam.zoom_to_cursor {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }
}

/// Centers the camera on a given glyph
///
/// This function calculates the appropriate position and zoom level
/// to center the camera on a glyph. It takes into account the glyph's
/// bounding box and the window size.
pub fn center_camera_on_glyph(
    glyph: &norad::Glyph,
    metrics: &crate::data::FontMetrics,
    camera_query: &mut Query<
        (&mut Transform, &mut OrthographicProjection),
        With<DesignCamera>,
    >,
    window_query: &Query<&Window>,
) {
    // Only proceed if the glyph has an outline
    if glyph.outline.is_none() {
        info!("Cannot center camera: glyph has no outline");
        return;
    }

    let outline = glyph.outline.as_ref().unwrap();

    // ---------- Step 1: Calculate the bounding box of the glyph ----------

    // Initialize with impossible values to ensure they get updated
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    // Include advance width in bounding box calculation
    // If advance is not defined, use a reasonable default based on units_per_em
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f32)
        .unwrap_or_else(|| (metrics.units_per_em as f32 * 0.5));

    // Get font metrics values with defaults if they're None
    // These are important for showing the proper em square gizmo
    let descender = metrics
        .descender
        .unwrap_or_else(|| -(metrics.units_per_em * 0.2))
        as f32;
    let ascender = metrics
        .ascender
        .unwrap_or_else(|| metrics.units_per_em * 0.8)
        as f32;

    // Include font metrics in bounding box calculation
    // This ensures the full em square is included in view
    min_x = min_x.min(0.0); // Left edge of em square
    max_x = max_x.max(width); // Right edge (advance width)
    min_y = min_y.min(descender); // Bottom of em square (descender)
    max_y = max_y.max(ascender); // Top of em square (ascender)

    // Iterate through all points in all contours to find the total bounding box
    for contour in &outline.contours {
        for point in &contour.points {
            let x = point.x as f32;
            let y = point.y as f32;

            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    // Add padding to the bounding box to avoid elements touching the edge of the view
    // More padding on top/bottom to account for toolbars and ensure metrics are visible
    let horizontal_padding = 50.0;
    let vertical_padding = 100.0;

    min_x -= horizontal_padding;
    max_x += horizontal_padding;
    min_y -= vertical_padding;
    max_y += vertical_padding;

    // ---------- Step 2: Calculate camera position ----------

    // Calculate the center of the bounding box (mathematical center)
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Apply an optical adjustment to move the camera up, which makes the glyph and
    // em square appear lower in the viewport. This provides better visual balance
    // by ensuring the glyph isn't positioned too close to the toolbar.
    let optical_adjustment_factor = 0.06; // 6% of the glyph height
    let optical_adjustment = (max_y - min_y) * optical_adjustment_factor;
    let adjusted_center_y = center_y + optical_adjustment;

    // ---------- Step 3: Calculate and apply zoom level ----------

    // Get the window dimensions for zoom calculation
    let window = if let Ok(window) = window_query.get_single() {
        window
    } else {
        warn!("Cannot center camera: window not available");
        return;
    };

    // Calculate dimensions for zoom
    let glyph_width = max_x - min_x;
    let glyph_height = max_y - min_y;

    // Check if camera exists and apply position and zoom
    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut() {
        // Set the camera position to center on the glyph
        transform.translation.x = center_x;
        transform.translation.y = adjusted_center_y;

        // Calculate zoom level to fit the glyph in view while respecting aspect ratio
        let window_aspect = window.width() / window.height();
        let glyph_aspect = glyph_width / glyph_height;

        // Determine whether width or height is the limiting factor
        let scale = if glyph_aspect > window_aspect {
            // Width limited: glyph is wider relative to the window
            (window.width() / glyph_width) * 0.9 // 90% of max possible zoom
        } else {
            // Height limited: glyph is taller relative to the window
            (window.height() / glyph_height) * 0.9
        };

        // Apply zoom level, with a minimum to prevent excessive zooming for small glyphs
        projection.scale = (CAMERA_ZOOM_FACTOR / scale).max(CAMERA_MIN_SCALE);

        info!(
            "Centered camera on glyph at ({:.2}, {:.2}) with zoom {:.3}",
            center_x, adjusted_center_y, projection.scale
        );
    } else {
        warn!("Cannot center camera: design camera not found");
    }
}
