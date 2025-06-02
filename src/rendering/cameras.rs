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
use crate::ui::theme::{CAMERA_MIN_SCALE, CAMERA_ZOOM_FACTOR};
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
const HORIZONTAL_PADDING: f32 = 50.0;
const VERTICAL_PADDING: f32 = 100.0;
const OPTICAL_ADJUSTMENT_FACTOR: f32 = 0.03;
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

// GLYPH CENTERING -------------------------------------------------------------

/// Centers the camera on a specific glyph
///
/// This automatically positions and zooms the camera to show the glyph
/// clearly in the center of the screen.
pub fn center_camera_on_glyph(
    glyph: &norad::Glyph,
    metrics: &crate::core::state::FontMetrics,
    camera_query: &mut Query<
        (&mut Transform, &mut OrthographicProjection),
        With<DesignCamera>,
    >,
    window_query: &Query<&Window>,
) {
    // Make sure the glyph has something to show
    if glyph.outline.is_none() {
        info!("Cannot center camera: glyph has no outline");
        return;
    }

    let outline = glyph.outline.as_ref().unwrap();

    // Calculate where the glyph is and how big it is
    let bbox = calculate_glyph_bounding_box(glyph, metrics, outline);

    // Get the window size for zoom calculations
    let window = match window_query.get_single() {
        Ok(window) => window,
        Err(_) => {
            warn!("Cannot center camera: window not available");
            return;
        }
    };

    // Move and zoom the camera to show the glyph
    position_camera(camera_query, bbox, window);
}

// HELPER TYPES AND FUNCTIONS -------------------------------------------------

/// A rectangle that contains a glyph
///
/// This represents the area that a glyph occupies, used for centering
/// the camera and calculating zoom levels.
struct BoundingBox {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl BoundingBox {
    /// Creates a new empty bounding box
    fn new() -> Self {
        Self {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }

    /// Gets the width of the bounding box
    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Gets the height of the bounding box
    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// Gets the center X coordinate
    fn center_x(&self) -> f32 {
        (self.min_x + self.max_x) / 2.0
    }

    /// Gets the center Y coordinate
    fn center_y(&self) -> f32 {
        (self.min_y + self.max_y) / 2.0
    }

    /// Gets a visually adjusted center Y coordinate
    ///
    /// This makes the glyph appear slightly lower in the viewport
    /// for better visual balance.
    fn adjusted_center_y(&self) -> f32 {
        let adjustment = self.height() * OPTICAL_ADJUSTMENT_FACTOR;
        self.center_y() + adjustment
    }

    /// Expands the box to include a point
    fn include_point(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    /// Adds padding around the bounding box
    fn add_padding(&mut self, horizontal: f32, vertical: f32) {
        self.min_x -= horizontal;
        self.max_x += horizontal;
        self.min_y -= vertical;
        self.max_y += vertical;
    }
}

/// Calculates the area that a glyph occupies
fn calculate_glyph_bounding_box(
    glyph: &norad::Glyph,
    metrics: &crate::core::state::FontMetrics,
    outline: &norad::Outline,
) -> BoundingBox {
    let mut bbox = BoundingBox::new();

    // Get the glyph's width
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f32)
        .unwrap_or_else(|| (metrics.units_per_em as f32 * 0.5));

    // Get font metrics with sensible defaults
    let descender = metrics
        .descender
        .unwrap_or_else(|| -(metrics.units_per_em * 0.2))
        as f32;
    let ascender = metrics
        .ascender
        .unwrap_or_else(|| metrics.units_per_em * 0.8)
        as f32;

    // Start with the font's standard dimensions
    bbox.include_point(0.0, descender);
    bbox.include_point(width, ascender);

    // Include all the actual glyph points
    for contour in &outline.contours {
        for point in &contour.points {
            let x = point.x as f32;
            let y = point.y as f32;
            bbox.include_point(x, y);
        }
    }

    // Add some padding so the glyph isn't right at the edge
    bbox.add_padding(HORIZONTAL_PADDING, VERTICAL_PADDING);

    bbox
}

/// Moves and zooms the camera to show a glyph
fn position_camera(
    camera_query: &mut Query<
        (&mut Transform, &mut OrthographicProjection),
        With<DesignCamera>,
    >,
    bbox: BoundingBox,
    window: &Window,
) {
    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut() {
        // Center the camera on the glyph
        transform.translation.x = bbox.center_x();
        transform.translation.y = bbox.adjusted_center_y();

        // Calculate how much to zoom based on glyph and window size
        let zoom_scale = calculate_zoom_scale(&bbox, window);

        // Apply the zoom
        projection.scale = 
            (CAMERA_ZOOM_FACTOR / zoom_scale).max(CAMERA_MIN_SCALE);

        info!(
            "Centered camera on glyph at ({:.2}, {:.2}) with zoom {:.3}",
            transform.translation.x, 
            transform.translation.y, 
            projection.scale
        );
    } else {
        warn!("Cannot center camera: design camera not found");
    }
}

/// Calculates how much to zoom to fit a glyph in the window
fn calculate_zoom_scale(bbox: &BoundingBox, window: &Window) -> f32 {
    let window_aspect = window.width() / window.height();
    let glyph_aspect = bbox.width() / bbox.height();

    if glyph_aspect > window_aspect {
        // Glyph is wide - fit to width
        (window.width() / bbox.width()) * MAX_ZOOM_PERCENTAGE
    } else {
        // Glyph is tall - fit to height
        (window.height() / bbox.height()) * MAX_ZOOM_PERCENTAGE
    }
}
