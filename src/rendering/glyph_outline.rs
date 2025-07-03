//! Glyph outline rendering functions
//!
//! Renders glyph outlines with proper cubic Bézier curves, control points, and handles.
//! This uses our thread-safe FontData structures for performance.

use crate::core::state::{OutlineData, ContourData, PointData, PointTypeData};
use crate::ui::theme::{
    PATH_LINE_COLOR, HANDLE_LINE_COLOR, ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS,
    OFF_CURVE_POINT_COLOR, OFF_CURVE_POINT_RADIUS, USE_SQUARE_FOR_ON_CURVE,
    ON_CURVE_SQUARE_ADJUSTMENT, ON_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_INNER_CIRCLE_RATIO,
};
use bevy::prelude::*;
use bevy::gizmos::gizmos::Gizmos;

/// Draw a complete glyph outline at a specific design-space position
pub fn draw_glyph_outline_at_position(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    offset: Vec2,
) {
    let Some(outline) = outline else { return };
    for contour in &outline.contours {
        draw_contour_path_at_position(gizmos, contour, offset);
    }
}

/// Draw glyph points (on-curve and off-curve) at a specific design-space position
pub fn draw_glyph_points_at_position(
    gizmos: &mut Gizmos,
    outline: &OutlineData,
    offset: Vec2,
) {
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }
        for point in &contour.points {
            let point_pos = Vec2::new(point.x as f32, point.y as f32) + offset;
            let is_on_curve = is_on_curve(point);
            let (size, color) = if is_on_curve {
                (ON_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR)
            } else {
                (OFF_CURVE_POINT_RADIUS, OFF_CURVE_POINT_COLOR)
            };
            if is_on_curve && USE_SQUARE_FOR_ON_CURVE {
                let half_size = size / ON_CURVE_SQUARE_ADJUSTMENT;
                gizmos.rect_2d(point_pos, Vec2::new(size * 2.0, size * 2.0), color);
                gizmos.circle_2d(point_pos, half_size * ON_CURVE_INNER_CIRCLE_RATIO, color);
            } else {
                gizmos.circle_2d(point_pos, size, color);
                gizmos.circle_2d(point_pos, size * OFF_CURVE_INNER_CIRCLE_RATIO, color);
            }
        }
    }
}

/// Draw a contour path with proper cubic curves at a specific design-space position
pub fn draw_contour_path_at_position(
    gizmos: &mut Gizmos,
    contour: &ContourData,
    offset: Vec2,
) {
    let points = &contour.points;
    if points.is_empty() { return; }

    let mut segment_start_idx = 0;
    if !is_on_curve(&points[0]) {
        if let Some(last_on_curve_idx) = points.iter().rposition(|p| is_on_curve(p)) {
            segment_start_idx = last_on_curve_idx;
        }
    }
    
    let mut last_was_on_curve = true;
    for i in 0..=points.len() {
        let point_idx = (segment_start_idx + i) % points.len();
        let next_point_idx = (segment_start_idx + i + 1) % points.len();
        if i == points.len() { break; } // Completed full loop

        let is_on = is_on_curve(&points[next_point_idx]);
        if is_on && last_was_on_curve {
            let start_point = &points[point_idx];
            let end_point = &points[next_point_idx];
            let start_pos = Vec2::new(start_point.x as f32, start_point.y as f32) + offset;
            let end_pos = Vec2::new(end_point.x as f32, end_point.y as f32) + offset;
            gizmos.line_2d(start_pos, end_pos, PATH_LINE_COLOR);
        } else if is_on {
            let mut segment_points = Vec::new();
            let mut idx = point_idx;
            loop {
                segment_points.push(&points[idx]);
                idx = (idx + 1) % points.len();
                if idx == (next_point_idx + 1) % points.len() { break; }
            }
            draw_curve_segment_at_position(gizmos, &segment_points, PATH_LINE_COLOR, offset);
        }
        last_was_on_curve = is_on;
    }
}

/// Draw a curve segment (line, quadratic, or cubic) at a specific design-space position
fn draw_curve_segment_at_position(
    gizmos: &mut Gizmos,
    points: &[&PointData],
    color: Color,
    offset: Vec2,
) {
    if points.len() < 2 { return; }
    let p0 = Vec2::new(points[0].x as f32, points[0].y as f32) + offset;
    match points.len() {
        2 => {
            let p1 = Vec2::new(points[1].x as f32, points[1].y as f32) + offset;
            gizmos.line_2d(p0, p1, color);
        }
        3 => {
            let p1 = Vec2::new(points[1].x as f32, points[1].y as f32) + offset;
            let p2 = Vec2::new(points[2].x as f32, points[2].y as f32) + offset;
            let c1 = p0 + (2.0/3.0) * (p1 - p0);
            let c2 = p2 + (2.0/3.0) * (p1 - p2);
            draw_cubic_bezier(gizmos, p0, c1, c2, p2, color);
        }
        4 => {
            let p1 = Vec2::new(points[1].x as f32, points[1].y as f32) + offset;
            let p2 = Vec2::new(points[2].x as f32, points[2].y as f32) + offset;
            let p3 = Vec2::new(points[3].x as f32, points[3].y as f32) + offset;
            draw_cubic_bezier(gizmos, p0, p1, p2, p3, color);
        }
        _ => {}
    }
}

/// Draw a cubic bezier curve using line segments
fn draw_cubic_bezier(
    gizmos: &mut Gizmos,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    color: Color,
) {
    let num_segments = 32;
    let mut prev_point = p0;
    for i in 1..=num_segments {
        let t = i as f32 / num_segments as f32;
        let p = (1.0 - t).powi(3) * p0
            + 3.0 * (1.0 - t).powi(2) * t * p1
            + 3.0 * (1.0 - t) * t.powi(2) * p2
            + t.powi(3) * p3;
        gizmos.line_2d(prev_point, p, color);
        prev_point = p;
    }
}

/// Check if a point is on the curve
fn is_on_curve(point: &PointData) -> bool {
    matches!(point.point_type, PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve)
} 