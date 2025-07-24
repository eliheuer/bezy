//! FontIR-based glyph outline rendering
//!
//! Renders glyph outlines using FontIR data structures and kurbo::BezPath
//! instead of the old custom data structures.

use crate::core::state::FontIRAppState;
use crate::geometry::bezpath_editing::{
    extract_editable_points, PathPointType,
};
use crate::ui::theme::{
    HANDLE_LINE_COLOR, OFF_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_PRIMARY_COLOR,
    OFF_CURVE_POINT_RADIUS, ON_CURVE_INNER_CIRCLE_RATIO, ON_CURVE_PRIMARY_COLOR,
    ON_CURVE_POINT_RADIUS, ON_CURVE_SQUARE_ADJUSTMENT, PATH_STROKE_COLOR,
    USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;
use kurbo::{BezPath, PathEl, Point};

/// Draw a complete glyph outline from FontIR at a specific position
pub fn draw_fontir_glyph_outline_at_position(
    gizmos: &mut Gizmos,
    paths: &[BezPath],
    offset: Vec2,
) {
    for path in paths {
        if path.elements().is_empty() {
            continue;
        }
        draw_bezpath_outline_at_position(gizmos, path, offset);
    }
}

/// Draw a single BezPath as an outline
pub fn draw_bezpath_outline_at_position(
    gizmos: &mut Gizmos,
    path: &BezPath,
    offset: Vec2,
) {
    let elements: Vec<_> = path.elements().iter().collect();

    if elements.is_empty() {
        return;
    }

    let mut current_pos = Point::ZERO;

    for element in elements {
        match element {
            PathEl::MoveTo(pt) => {
                current_pos = *pt;
            }
            PathEl::LineTo(pt) => {
                let start =
                    Vec2::new(current_pos.x as f32, current_pos.y as f32)
                        + offset;
                let end = Vec2::new(pt.x as f32, pt.y as f32) + offset;
                gizmos.line_2d(start, end, PATH_STROKE_COLOR);
                current_pos = *pt;
            }
            PathEl::CurveTo(c1, c2, pt) => {
                // Draw cubic Bezier curve using multiple line segments
                let start = current_pos;
                let end = *pt;
                draw_cubic_curve(gizmos, start, *c1, *c2, end, offset);
                current_pos = *pt;
            }
            PathEl::QuadTo(c, pt) => {
                // Draw quadratic Bezier curve
                let start = current_pos;
                let end = *pt;
                draw_quad_curve(gizmos, start, *c, end, offset);
                current_pos = *pt;
            }
            PathEl::ClosePath => {
                // Close the path - this would be handled automatically in a real renderer
            }
        }
    }
}

/// Draw FontIR glyph points and control handles
pub fn draw_fontir_glyph_points_and_handles_at_position(
    gizmos: &mut Gizmos,
    paths: &[BezPath],
    offset: Vec2,
) {
    for path in paths {
        draw_bezpath_points_and_handles_at_position(gizmos, path, offset);
    }
}

/// Draw editable points and handles for a single BezPath
pub fn draw_bezpath_points_and_handles_at_position(
    gizmos: &mut Gizmos,
    path: &BezPath,
    offset: Vec2,
) {
    let points = extract_editable_points(path);

    // First draw control handles
    draw_bezpath_control_handles_at_position(gizmos, path, offset);

    // Then draw points
    for point in points {
        let point_pos =
            Vec2::new(point.position.x as f32, point.position.y as f32)
                + offset;
        let is_on_curve = matches!(point.point_type, PathPointType::OnCurve);

        let (size, color) = if is_on_curve {
            (ON_CURVE_POINT_RADIUS, ON_CURVE_PRIMARY_COLOR)
        } else {
            (OFF_CURVE_POINT_RADIUS, OFF_CURVE_PRIMARY_COLOR)
        };

        // Draw transparent background shape first
        let transparent_color = Color::srgba(
            color.to_srgba().red,
            color.to_srgba().green,
            color.to_srgba().blue,
            0.5,
        );

        if is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            let half_size = size / ON_CURVE_SQUARE_ADJUSTMENT;
            let square_size = Vec2::new(size * 2.0, size * 2.0);
            let bg_square_size = Vec2::new(size * 2.4, size * 2.4); // Slightly larger background

            // Background transparent shape
            gizmos.rect_2d(point_pos, bg_square_size, transparent_color);

            // Original solid shapes
            gizmos.rect_2d(point_pos, square_size, color);
            // Don't draw center dot for squares - they should be hollow
        } else {
            let bg_radius = size * 1.2; // Slightly larger background

            // Background transparent shape
            gizmos.circle_2d(point_pos, bg_radius, transparent_color);

            // Original solid shapes
            gizmos.circle_2d(point_pos, size, color);
            let inner_radius = size * OFF_CURVE_INNER_CIRCLE_RATIO;
            gizmos.circle_2d(point_pos, inner_radius, color);
        }
    }
}

/// Draw control handles for a BezPath
fn draw_bezpath_control_handles_at_position(
    gizmos: &mut Gizmos,
    path: &BezPath,
    offset: Vec2,
) {
    let elements: Vec<_> = path.elements().iter().collect();
    let mut current_pos = Point::ZERO;

    for element in elements {
        match element {
            PathEl::MoveTo(pt) => {
                current_pos = *pt;
            }
            PathEl::LineTo(pt) => {
                current_pos = *pt;
            }
            PathEl::CurveTo(c1, c2, pt) => {
                // Draw handle lines
                let start =
                    Vec2::new(current_pos.x as f32, current_pos.y as f32)
                        + offset;
                let c1_pos = Vec2::new(c1.x as f32, c1.y as f32) + offset;
                let c2_pos = Vec2::new(c2.x as f32, c2.y as f32) + offset;
                let end = Vec2::new(pt.x as f32, pt.y as f32) + offset;

                gizmos.line_2d(start, c1_pos, HANDLE_LINE_COLOR);
                gizmos.line_2d(end, c2_pos, HANDLE_LINE_COLOR);

                current_pos = *pt;
            }
            PathEl::QuadTo(c, pt) => {
                // Draw handle line
                let start =
                    Vec2::new(current_pos.x as f32, current_pos.y as f32)
                        + offset;
                let c_pos = Vec2::new(c.x as f32, c.y as f32) + offset;
                let end = Vec2::new(pt.x as f32, pt.y as f32) + offset;

                gizmos.line_2d(start, c_pos, HANDLE_LINE_COLOR);
                gizmos.line_2d(end, c_pos, HANDLE_LINE_COLOR);

                current_pos = *pt;
            }
            PathEl::ClosePath => {}
        }
    }
}

/// Draw a cubic Bezier curve using line segments
fn draw_cubic_curve(
    gizmos: &mut Gizmos,
    start: Point,
    c1: Point,
    c2: Point,
    end: Point,
    offset: Vec2,
) {
    const SEGMENTS: usize = 20;
    let mut last_point = start;

    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let point = cubic_bezier_point(start, c1, c2, end, t);

        let last_pos =
            Vec2::new(last_point.x as f32, last_point.y as f32) + offset;
        let current_pos = Vec2::new(point.x as f32, point.y as f32) + offset;

        gizmos.line_2d(last_pos, current_pos, PATH_STROKE_COLOR);
        last_point = point;
    }
}

/// Draw a quadratic Bezier curve using line segments
fn draw_quad_curve(
    gizmos: &mut Gizmos,
    start: Point,
    control: Point,
    end: Point,
    offset: Vec2,
) {
    const SEGMENTS: usize = 15;
    let mut last_point = start;

    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let point = quad_bezier_point(start, control, end, t);

        let last_pos =
            Vec2::new(last_point.x as f32, last_point.y as f32) + offset;
        let current_pos = Vec2::new(point.x as f32, point.y as f32) + offset;

        gizmos.line_2d(last_pos, current_pos, PATH_STROKE_COLOR);
        last_point = point;
    }
}

/// Calculate a point on a cubic Bezier curve
fn cubic_bezier_point(
    start: Point,
    c1: Point,
    c2: Point,
    end: Point,
    t: f64,
) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Point::new(
        mt3 * start.x
            + 3.0 * mt2 * t * c1.x
            + 3.0 * mt * t2 * c2.x
            + t3 * end.x,
        mt3 * start.y
            + 3.0 * mt2 * t * c1.y
            + 3.0 * mt * t2 * c2.y
            + t3 * end.y,
    )
}

/// Calculate a point on a quadratic Bezier curve
fn quad_bezier_point(
    start: Point,
    control: Point,
    end: Point,
    t: f64,
) -> Point {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    Point::new(
        mt2 * start.x + 2.0 * mt * t * control.x + t2 * end.x,
        mt2 * start.y + 2.0 * mt * t * control.y + t2 * end.y,
    )
}

/// Get glyph paths from FontIR app state
pub fn get_fontir_glyph_paths(
    fontir_app_state: &FontIRAppState,
    glyph_name: &str,
) -> Option<Vec<BezPath>> {
    use bevy::log::{info, warn};

    info!("get_fontir_glyph_paths: Looking for glyph '{}'", glyph_name);
    info!(
        "get_fontir_glyph_paths: Cache has {} glyphs",
        fontir_app_state.glyph_cache.len()
    );

    // Debug: List first few glyph names in cache
    let cache_names: Vec<String> = fontir_app_state
        .glyph_cache
        .keys()
        .take(10)
        .cloned()
        .collect();
    info!(
        "get_fontir_glyph_paths: First 10 cache keys: {:?}",
        cache_names
    );

    // Try to get from FontIR cache first
    if let Some(glyph) = fontir_app_state.get_glyph(glyph_name) {
        info!(
            "get_fontir_glyph_paths: Found glyph '{}' in FontIR cache",
            glyph_name
        );

        // Always use the first available instance since location matching is complex
        if let Some((_location, instance)) = glyph.sources().iter().next() {
            info!("get_fontir_glyph_paths: Using first available instance for glyph '{}' with {} contours", glyph_name, instance.contours.len());
            return Some(instance.contours.clone());
        } else {
            warn!(
                "get_fontir_glyph_paths: No instances found for glyph '{}'",
                glyph_name
            );
        }
    } else {
        warn!(
            "get_fontir_glyph_paths: Glyph '{}' not found in FontIR cache",
            glyph_name
        );
    }

    warn!(
        "get_fontir_glyph_paths: Falling back to placeholder shapes for '{}'",
        glyph_name
    );

    // Use the comprehensive fallback system from FontIRAppState
    fontir_app_state.get_glyph_paths(glyph_name)
}

/// Fallback function - try to get any glyph for testing
pub fn get_first_available_glyph_paths(
    fontir_app_state: &FontIRAppState,
) -> Option<(String, Vec<BezPath>)> {
    // For now, return None since we haven't implemented glyph enumeration
    // TODO: Implement proper glyph iteration from FontIR source
    None
}

// REMOVED: draw_fontir_glyph_outline_from_live_transforms and related functions
// This approach caused severe outline distortion and flashing during nudging.
// Using stable FontIR working copy rendering instead.
