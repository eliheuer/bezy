//! Glyph outline rendering functions
//!
//! Renders glyph outlines with proper cubic Bézier curves, control points, and handles.
//! This uses our thread-safe FontData structures for performance.

use crate::core::state::{ContourData, OutlineData, PointData, PointTypeData};
use crate::ui::theme::{
    HANDLE_LINE_COLOR, OFF_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_POINT_COLOR,
    OFF_CURVE_POINT_RADIUS, ON_CURVE_INNER_CIRCLE_RATIO, ON_CURVE_POINT_COLOR,
    ON_CURVE_POINT_RADIUS, ON_CURVE_SQUARE_ADJUSTMENT, PATH_STROKE_COLOR, USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;

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
                let square_size = Vec2::new(size * 2.0, size * 2.0);
                gizmos.rect_2d(point_pos, square_size, color);
                let inner_radius = half_size * ON_CURVE_INNER_CIRCLE_RATIO;
                gizmos.circle_2d(point_pos, inner_radius, color);
            } else {
                gizmos.circle_2d(point_pos, size, color);
                let inner_radius = size * OFF_CURVE_INNER_CIRCLE_RATIO;
                gizmos.circle_2d(point_pos, inner_radius, color);
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
    if points.len() < 2 {
        return;
    }

    let mut last_on_curve: Option<Vec2> = None;
    if points[0].point_type == PointTypeData::Move {
        last_on_curve = Some(Vec2::new(points[0].x as f32, points[0].y as f32) + offset);
    }

    for i in 0..points.len() {
        let p1_idx = i;
        let p2_idx = (i + 1) % points.len();
        
        let p1 = &points[p1_idx];
        let p2 = &points[p2_idx];

        let p1_pos = Vec2::new(p1.x as f32, p1.y as f32) + offset;

        if p2.point_type == PointTypeData::Line {
            let p2_pos = Vec2::new(p2.x as f32, p2.y as f32) + offset;
            gizmos.line_2d(p1_pos, p2_pos, PATH_STROKE_COLOR);
            last_on_curve = Some(p2_pos);
        } else if p2.point_type == PointTypeData::Curve {
            let p3_idx = (i + 2) % points.len();
            let p4_idx = (i + 3) % points.len();

            if let (Some(p3), Some(p4)) = (points.get(p3_idx), points.get(p4_idx)) {
                 if let Some(p0_pos) = last_on_curve {
                    let p2_pos = Vec2::new(p2.x as f32, p2.y as f32) + offset;
                    let p3_pos = Vec2::new(p3.x as f32, p3.y as f32) + offset;
                    draw_cubic_bezier(gizmos, p0_pos, p1_pos, p2_pos, p3_pos, PATH_STROKE_COLOR);
                    last_on_curve = Some(p3_pos);
                 }
            }
        }
    }
}


/// Draw a cubic bezier curve using line segments
fn draw_cubic_bezier(
    gizmos: &mut Gizmos,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
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
    matches!(
        point.point_type,
        PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve
    )
}

fn draw_control_handles_at_position(
    gizmos: &mut Gizmos,
    contour: &ContourData,
    offset: Vec2,
) {
    for i in 0..contour.points.len() {
        let p = &contour.points[i];
        if p.point_type != PointTypeData::OffCurve {
            continue;
        }

        let p_pos = Vec2::new(p.x as f32, p.y as f32) + offset;

        // Find previous on-curve point
        let mut prev_on_curve = None;
        for j in (0..i).rev() {
            if is_on_curve(&contour.points[j]) {
                prev_on_curve = Some(Vec2::new(contour.points[j].x as f32, contour.points[j].y as f32) + offset);
                break;
            }
        }

        // Find next on-curve point
        let mut next_on_curve = None;
        for j in (i + 1)..contour.points.len() {
            if is_on_curve(&contour.points[j]) {
                next_on_curve = Some(Vec2::new(contour.points[j].x as f32, contour.points[j].y as f32) + offset);
                break;
            }
        }

        if let Some(prev) = prev_on_curve {
            gizmos.line_2d(p_pos, prev, HANDLE_LINE_COLOR);
        }
        if let Some(next) = next_on_curve {
             gizmos.line_2d(p_pos, next, HANDLE_LINE_COLOR);
        }
    }
} 