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

use crate::core::settings::{
    KEYBOARD_ZOOM_STEP, MAX_ALLOWED_ZOOM_SCALE, MIN_ALLOWED_ZOOM_SCALE,
};
use crate::ui::theme::{CAMERA_MIN_SCALE, CAMERA_ZOOM_FACTOR};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_pancam::*;

// Constants for camera positioning and rendering
const DESIGN_CAMERA_ORDER: isize = 0;
const UI_CAMERA_ORDER: isize = 1;

// RenderLayers are typically usize, not u8
const DESIGN_CAMERA_LAYER: usize = 0;
const UI_CAMERA_LAYER: usize = 1;

// Constants for glyph centering
const HORIZONTAL_PADDING: f32 = 50.0;
const VERTICAL_PADDING: f32 = 100.0;
const OPTICAL_ADJUSTMENT_FACTOR: f32 = 0.03;
const MAX_ZOOM_PERCENTAGE: f32 = 0.9; // 90% of maximum possible zoom

//------------------------------------------------------------------------------
// Camera Components
//------------------------------------------------------------------------------

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

//------------------------------------------------------------------------------
// Camera Setup Functions
//------------------------------------------------------------------------------

/// Spawns the main camera for the design space
///
/// This camera is used to view and interact with bezier curves, points, and other design elements.
/// It is configured with PanCam for panning and zooming functionality.
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // Camera order determines rendering sequence in a multi-camera setup
            // Lower values render first, higher values render later (on top)
            // Design camera (0) renders before UI camera (1)
            order: DESIGN_CAMERA_ORDER,
            ..default()
        },
        DesignCamera,
        // Render layers control which entities this camera can see
        // This camera only renders entities on layer 0 (design elements)
        // UI elements are on layer 1 and won't be visible to this camera
        RenderLayers::layer(DESIGN_CAMERA_LAYER),
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
            // Camera order determines rendering sequence in a multi-camera setup
            // Higher values render later (on top) of cameras with lower values
            // UI camera (1) renders after design camera (0), appearing on top
            order: UI_CAMERA_ORDER,
            ..default()
        },
        // Render layers control which entities this camera can see
        // This camera only renders entities on layer 1 (UI elements)
        // Design elements are on layer 0 and won't be visible to this camera
        RenderLayers::layer(UI_CAMERA_LAYER),
        UiCamera,
    ));
}

//------------------------------------------------------------------------------
// Camera Control System
//------------------------------------------------------------------------------

/// Handles camera controls based on keyboard input
///
/// Keyboard shortcuts:
/// - Space: Hold to temporarily enable camera panning (works in any edit mode)
/// - T: Toggle zoom-to-cursor behavior
/// - Cmd + Plus (+): Zoom in
/// - Cmd + Minus (-): Zoom out
pub fn toggle_camera_controls(
    mut query: Query<&mut PanCam>,
    mut camera_query: Query<&mut OrthographicProjection, With<DesignCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Note: Spacebar handling for temporary pan mode is now handled by
    // the edit_mode_toolbar::temporary_mode system for better UX
    handle_zoom_to_cursor_toggle(&mut query, &keys);
    handle_zoom_hotkeys(&mut camera_query, &keys);
}

/// Handles T key for toggling zoom-to-cursor behavior
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

/// Handles Cmd+Plus and Cmd+Minus for zooming in and out
fn handle_zoom_hotkeys(
    camera_query: &mut Query<&mut OrthographicProjection, With<DesignCamera>>,
    keys: &Res<ButtonInput<KeyCode>>,
) {
    // Check for Command/Control modifier key (works on both macOS and Windows/Linux)
    let cmd_ctrl_pressed = keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight);

    if !cmd_ctrl_pressed {
        return;
    }

    // Handle Cmd++ (zoom in)
    if keys.just_pressed(KeyCode::Equal) {
        zoom_camera(camera_query, true);
    }

    // Handle Cmd+- (zoom out)
    if keys.just_pressed(KeyCode::Minus) {
        zoom_camera(camera_query, false);
    }
}

/// Zooms the camera in or out based on the given direction
///
/// - `zoom_in`: true to zoom in, false to zoom out
fn zoom_camera(
    camera_query: &mut Query<&mut OrthographicProjection, With<DesignCamera>>,
    zoom_in: bool,
) {
    if let Ok(mut projection) = camera_query.get_single_mut() {
        let old_scale = projection.scale;

        if zoom_in {
            // Zoom in by multiplying scale by the zoom step factor
            // Smaller scale value = more zoomed in
            projection.scale *= KEYBOARD_ZOOM_STEP;
            // Ensure we don't zoom in too far
            projection.scale = projection.scale.max(MIN_ALLOWED_ZOOM_SCALE);
            info!("Zoomed in to scale {:.3}", projection.scale);
        } else {
            // Zoom out by dividing scale by the zoom step factor
            // Larger scale value = more zoomed out
            projection.scale /= KEYBOARD_ZOOM_STEP;
            // Ensure we don't zoom out too far
            projection.scale = projection.scale.min(MAX_ALLOWED_ZOOM_SCALE);
            info!("Zoomed out to scale {:.3}", projection.scale);
        }

        if projection.scale == old_scale {
            // If scale didn't change, we've hit the min or max limit
            let limit_type = if zoom_in { "minimum" } else { "maximum" };
            info!("Camera already at {} zoom limit", limit_type);
        }
    }
}

//------------------------------------------------------------------------------
// Camera Positioning Functions
//------------------------------------------------------------------------------

/// Centers the camera on a given glyph
///
/// This function calculates the appropriate position and zoom level
/// to center the camera on a glyph. It takes into account the glyph's
/// bounding box and the window size.
pub fn center_camera_on_glyph(
    glyph: &norad::Glyph,
    metrics: &crate::core::data::FontMetrics,
    camera_query: &mut Query<
        (&mut Transform, &mut OrthographicProjection),
        With<DesignCamera>,
    >,
    window_query: &Query<&Window>,
) {
    // Validate glyph has an outline
    if glyph.outline.is_none() {
        info!("Cannot center camera: glyph has no outline");
        return;
    }

    let outline = glyph.outline.as_ref().unwrap();

    // Calculate glyph bounding box with padding
    let bbox = calculate_glyph_bounding_box(glyph, metrics, outline);

    // Get window for zoom calculation
    let window = match window_query.get_single() {
        Ok(window) => window,
        Err(_) => {
            warn!("Cannot center camera: window not available");
            return;
        }
    };

    // Position and zoom the camera
    position_camera(camera_query, bbox, window);
}

/// Represents a bounding box with minimum and maximum coordinates
struct BoundingBox {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl BoundingBox {
    /// Creates a new bounding box with extreme initial values
    fn new() -> Self {
        Self {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }

    /// Returns the width of the bounding box
    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Returns the height of the bounding box
    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// Returns the center x-coordinate of the bounding box
    fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) / 2.0
    }

    /// Returns the center y-coordinate of the bounding box
    fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) / 2.0
    }

    /// Calculates an optically adjusted center y-coordinate
    /// This makes the glyph appear lower in the viewport for better visual balance
    fn adjusted_center_y(&self) -> f32 {
        let adjustment = self.height() * OPTICAL_ADJUSTMENT_FACTOR;
        self.center_y() + adjustment
    }

    /// Expands the bounding box to include the given point
    fn include_point(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    /// Adds padding to the bounding box
    fn add_padding(&mut self, horizontal: f32, vertical: f32) {
        self.min_x -= horizontal;
        self.max_x += horizontal;
        self.min_y -= vertical;
        self.max_y += vertical;
    }
}

/// Calculates the bounding box of a glyph, including metrics and padding
fn calculate_glyph_bounding_box(
    glyph: &norad::Glyph,
    metrics: &crate::core::data::FontMetrics,
    outline: &norad::Outline,
) -> BoundingBox {
    let mut bbox = BoundingBox::new();

    // Include advance width
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f32)
        .unwrap_or_else(|| (metrics.units_per_em as f32 * 0.5));

    // Get metrics values with defaults
    let descender = metrics
        .descender
        .unwrap_or_else(|| -(metrics.units_per_em * 0.2))
        as f32;
    let ascender = metrics
        .ascender
        .unwrap_or_else(|| metrics.units_per_em * 0.8)
        as f32;

    // Include em square in bounding box
    bbox.include_point(0.0, descender); // Bottom-left
    bbox.include_point(width, ascender); // Top-right

    // Include all contour points
    for contour in &outline.contours {
        for point in &contour.points {
            let x = point.x as f32;
            let y = point.y as f32;
            bbox.include_point(x, y);
        }
    }

    // Add padding to avoid elements touching the edge
    bbox.add_padding(HORIZONTAL_PADDING, VERTICAL_PADDING);

    bbox
}

/// Positions and zooms the camera to fit the glyph's bounding box
fn position_camera(
    camera_query: &mut Query<
        (&mut Transform, &mut OrthographicProjection),
        With<DesignCamera>,
    >,
    bbox: BoundingBox,
    window: &Window,
) {
    // Check if camera exists
    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut() {
        // Set the camera position to center on the glyph
        transform.translation.x = bbox.center_x();
        transform.translation.y = bbox.adjusted_center_y();

        // Calculate zoom level based on both dimensions
        let zoom_scale = calculate_zoom_scale(&bbox, window);

        // Apply zoom level
        projection.scale =
            (CAMERA_ZOOM_FACTOR / zoom_scale).max(CAMERA_MIN_SCALE);

        info!(
            "Centered camera on glyph at ({:.2}, {:.2}) with zoom {:.3}",
            transform.translation.x, transform.translation.y, projection.scale
        );
    } else {
        warn!("Cannot center camera: design camera not found");
    }
}

/// Calculates an appropriate zoom scale to fit the glyph in the window
fn calculate_zoom_scale(bbox: &BoundingBox, window: &Window) -> f32 {
    let window_aspect = window.width() / window.height();
    let glyph_aspect = bbox.width() / bbox.height();

    if glyph_aspect > window_aspect {
        // Width limited: glyph is wider relative to the window
        (window.width() / bbox.width()) * MAX_ZOOM_PERCENTAGE
    } else {
        // Height limited: glyph is taller relative to the window
        (window.height() / bbox.height()) * MAX_ZOOM_PERCENTAGE
    }
}
