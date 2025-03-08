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

/// Center the camera on a glyph, ensuring all glyph points are visible
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
        return;
    }

    let outline = glyph.outline.as_ref().unwrap();

    // Calculate the bounding box of the glyph
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    // Include advance width in bounding box calculation
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f32)
        .unwrap_or_else(|| (metrics.units_per_em as f32 * 0.5));

    // Get font metrics values with defaults if they're None
    let descender = metrics
        .descender
        .unwrap_or_else(|| -(metrics.units_per_em * 0.2))
        as f32;
    let ascender = metrics
        .ascender
        .unwrap_or_else(|| metrics.units_per_em * 0.8)
        as f32;

    // Include font metrics in bounding box calculation - this ensures the full metrics rectangle is included
    min_x = min_x.min(0.0); // Left edge of metrics rectangle
    max_x = max_x.max(width); // Right edge of metrics rectangle
    min_y = min_y.min(descender); // Bottom of metrics rectangle (descender)
    max_y = max_y.max(ascender); // Top of metrics rectangle (ascender)

    // Iterate through all points in all contours to find the bounding box of the glyph outline
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

    // Add padding to the bounding box
    // Use more generous padding for the top and bottom to ensure metrics rectangle is fully visible
    let horizontal_padding = 50.0;
    let vertical_padding = 100.0; // More padding on top and bottom

    min_x -= horizontal_padding;
    max_x += horizontal_padding;
    min_y -= vertical_padding;
    max_y += vertical_padding;

    // Calculate the center of the bounding box
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Apply an optical adjustment to move the camera up, which will make the glyph and
    // metrics rectangle appear lower in the viewport. This ensures the glyph isn't
    // positioned too close to the toolbar and improves the visual balance of the UI.
    // The value is a percentage of the glyph height to shift upward.
    let optical_adjustment_factor = 0.06; // 6% of the glyph height (reduced from 12%)
    let optical_adjustment = (max_y - min_y) * optical_adjustment_factor;
    let adjusted_center_y = center_y + optical_adjustment;

    // Get the window dimensions for zoom calculation
    let window = if let Ok(window) = window_query.get_single() {
        window
    } else {
        return;
    };

    // Calculate the required zoom level to fit the glyph and metrics rectangle
    let glyph_width = max_x - min_x;
    let glyph_height = max_y - min_y;

    // Check if we need to adjust zoom
    if let Ok((mut transform, mut projection)) = camera_query.get_single_mut() {
        // Set the camera position to center on the glyph
        transform.translation.x = center_x;
        transform.translation.y = adjusted_center_y;

        // Calculate zoom level to fit the glyph and metrics rectangle in the view
        let window_aspect = window.width() / window.height();
        let glyph_aspect = glyph_width / glyph_height;

        // Adjust the scale/zoom based on whether width or height is the limiting factor
        let scale = if glyph_aspect > window_aspect {
            // Width limited
            (window.width() / glyph_width) * 0.9
        } else {
            // Height limited
            (window.height() / glyph_height) * 0.9
        };

        // Apply zoom - make sure it's not too zoomed in for small glyphs
        // For a consistent view, use a minimum zoom level to prevent excessive zooming
        let min_scale = 0.8; // Slightly zoomed out view
        projection.scale = (1.0 / scale).max(min_scale);

        info!(
            "Centered camera on glyph at ({}, {}) with zoom {}",
            center_x, adjusted_center_y, projection.scale
        );
    }
}
