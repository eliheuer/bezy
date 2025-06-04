//! Shared glyph outline rendering functionality
//!
//! This module contains functions for rendering glyph outlines that can be used
//! by both the main glyph rendering system and individual sorts.

use crate::ui::panes::design_space::{DPoint, ViewPort};
use crate::ui::theme::{
    PATH_LINE_COLOR, HANDLE_LINE_COLOR, ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS,
    OFF_CURVE_POINT_COLOR, OFF_CURVE_POINT_RADIUS, USE_SQUARE_FOR_ON_CURVE,
    ON_CURVE_SQUARE_ADJUSTMENT, ON_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_INNER_CIRCLE_RATIO,
};
use bevy::prelude::*;
use norad::{Contour, ContourPoint};

/// Draw a complete glyph outline at a specific position
pub fn draw_glyph_outline_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    outline: &norad::glyph::Outline,
    offset: Vec2,
) {
    // Render each contour in the outline
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }

        // Draw the actual path with proper cubic curves
        draw_contour_path_at_position(gizmos, viewport, contour, offset);

        // Draw the control handles for off-curve points
        draw_control_handles_at_position(gizmos, viewport, contour, offset);
    }
}

/// Draw glyph points (on-curve and off-curve) at a specific position
pub fn draw_glyph_points_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    outline: &norad::glyph::Outline,
    offset: Vec2,
) {
    // Render each contour's points
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }

        // Draw each point in the contour
        for point in &contour.points {
            let point_pos = (point.x as f32 + offset.x, point.y as f32 + offset.y);
            let screen_pos = viewport.to_screen(DPoint::from(point_pos));

            // Determine if point is on-curve or off-curve
            let is_on_curve = match point.typ {
                norad::PointType::Move
                | norad::PointType::Line
                | norad::PointType::Curve => true,
                _ => false,
            };

            // Use different sizes and colors based on point type
            let (size, color) = if is_on_curve {
                (ON_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR)
            } else {
                (OFF_CURVE_POINT_RADIUS, OFF_CURVE_POINT_COLOR)
            };

            // Draw the appropriate shape based on point type
            if is_on_curve && USE_SQUARE_FOR_ON_CURVE {
                // For on-curve points, draw a square with a circle inside
                let half_size = size / ON_CURVE_SQUARE_ADJUSTMENT;
                
                // Draw the outer square
                gizmos.rect_2d(
                    screen_pos,
                    Vec2::new(size * 2.0, size * 2.0),
                    color,
                );

                // Draw the inner circle
                gizmos.circle_2d(
                    screen_pos,
                    half_size * ON_CURVE_INNER_CIRCLE_RATIO,
                    color,
                );
            } else {
                // For off-curve points, draw a filled circle with a smaller circle inside
                // First draw the outer circle
                gizmos.circle_2d(screen_pos, size, color);

                // Then draw a smaller inner circle with the same color
                gizmos.circle_2d(
                    screen_pos,
                    size * OFF_CURVE_INNER_CIRCLE_RATIO,
                    color,
                );
            }
        }
    }
}

/// Draw a contour path with proper cubic curves at a specific position
pub fn draw_contour_path_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &Contour,
    offset: Vec2,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Find segments between on-curve points
    let mut segment_start_idx = 0;
    let mut last_was_on_curve = false;

    // Handle the special case where the first point might be off-curve
    if !is_on_curve(&points[0]) {
        // Find the last on-curve point to start with
        let mut last_on_curve_idx = points.len() - 1;
        while last_on_curve_idx > 0 && !is_on_curve(&points[last_on_curve_idx])
        {
            last_on_curve_idx -= 1;
        }

        if is_on_curve(&points[last_on_curve_idx]) {
            segment_start_idx = last_on_curve_idx;
            last_was_on_curve = true;
        }
    } else {
        last_was_on_curve = true;
    }

    let path_color = PATH_LINE_COLOR;

    // Iterate through all points to identify and draw segments
    for i in 0..points.len() + 1 {
        let point_idx = i % points.len();
        let is_on = is_on_curve(&points[point_idx]);

        if is_on && last_was_on_curve {
            // If we have two consecutive on-curve points, draw a straight line
            let start_point = &points[segment_start_idx];
            let end_point = &points[point_idx];

            let start_pos = viewport.to_screen(DPoint::from((
                start_point.x as f32 + offset.x,
                start_point.y as f32 + offset.y,
            )));
            let end_pos = viewport.to_screen(DPoint::from((
                end_point.x as f32 + offset.x,
                end_point.y as f32 + offset.y,
            )));

            gizmos.line_2d(start_pos, end_pos, path_color);

            segment_start_idx = point_idx;
        } else if is_on {
            // Found the end of a curve segment (on-curve point after off-curve points)
            // Collect all points in this segment to draw a cubic bezier
            let mut segment_points = Vec::new();
            let mut idx = segment_start_idx;

            // Collect all points from segment_start_idx to point_idx
            loop {
                segment_points.push(&points[idx]);
                idx = (idx + 1) % points.len();
                if idx == (point_idx + 1) % points.len() {
                    break;
                }
            }

            // Draw the appropriate curve based on number of points
            draw_curve_segment_at_position(gizmos, viewport, &segment_points, path_color, offset);

            // Update for next segment
            segment_start_idx = point_idx;
        }

        last_was_on_curve = is_on;
    }
}

/// Draw control handles for off-curve points at a specific position
pub fn draw_control_handles_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &Contour,
    offset: Vec2,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    let handle_color = HANDLE_LINE_COLOR;

    // Process the contour looking at each segment
    let mut current_on_curve_idx = None;

    // First, find the first on-curve point
    for i in 0..points.len() {
        if is_on_curve(&points[i]) {
            current_on_curve_idx = Some(i);
            break;
        }
    }

    // If we couldn't find an on-curve point, we can't draw handles
    if current_on_curve_idx.is_none() {
        return;
    }

    let mut current_idx = current_on_curve_idx.unwrap();

    // Iterate through the contour segments
    for _ in 0..points.len() {
        // We're only processing segments that start with an on-curve point
        if is_on_curve(&points[current_idx]) {
            let current_on_curve_pos = viewport.to_screen(DPoint::from((
                points[current_idx].x as f32 + offset.x,
                points[current_idx].y as f32 + offset.y,
            )));

            // Look for the next on-curve point and collect off-curve points between them
            let mut off_curve_points = Vec::new();
            let mut next_idx = (current_idx + 1) % points.len();

            // Collect off-curve points until we find the next on-curve point
            while !is_on_curve(&points[next_idx]) {
                off_curve_points.push(next_idx);
                next_idx = (next_idx + 1) % points.len();

                // Safety check to avoid infinite loop
                if next_idx == current_idx {
                    break;
                }
            }

            // Only proceed if we found another on-curve point and have off-curve points
            if next_idx != current_idx && !off_curve_points.is_empty() {
                let next_on_curve_pos = viewport.to_screen(DPoint::from((
                    points[next_idx].x as f32 + offset.x,
                    points[next_idx].y as f32 + offset.y,
                )));

                // For cubic Bézier with 2 control points (most common case)
                if off_curve_points.len() == 2 {
                    // First control point connects back to the current on-curve point
                    let p1_idx = off_curve_points[0];
                    let p1_pos = viewport.to_screen(DPoint::from((
                        points[p1_idx].x as f32 + offset.x,
                        points[p1_idx].y as f32 + offset.y,
                    )));
                    gizmos.line_2d(current_on_curve_pos, p1_pos, handle_color);

                    // Second control point connects forward to the next on-curve point
                    let p2_idx = off_curve_points[1];
                    let p2_pos = viewport.to_screen(DPoint::from((
                        points[p2_idx].x as f32 + offset.x,
                        points[p2_idx].y as f32 + offset.y,
                    )));
                    gizmos.line_2d(next_on_curve_pos, p2_pos, handle_color);
                }
                // For quadratic Bézier or other cases with just one control point
                else if off_curve_points.len() == 1 {
                    // The single control point gets a handle from the current on-curve point
                    let control_idx = off_curve_points[0];
                    let control_pos = viewport.to_screen(DPoint::from((
                        points[control_idx].x as f32 + offset.x,
                        points[control_idx].y as f32 + offset.y,
                    )));
                    gizmos.line_2d(
                        current_on_curve_pos,
                        control_pos,
                        handle_color,
                    );
                }
                // For cases with more than 2 control points (less common)
                else {
                    // Connect first control point to the current on-curve point
                    let first_idx = off_curve_points[0];
                    let first_pos = viewport.to_screen(DPoint::from((
                        points[first_idx].x as f32 + offset.x,
                        points[first_idx].y as f32 + offset.y,
                    )));
                    gizmos.line_2d(
                        current_on_curve_pos,
                        first_pos,
                        handle_color,
                    );

                    // Connect last control point to the next on-curve point
                    let last_idx = off_curve_points[off_curve_points.len() - 1];
                    let last_pos = viewport.to_screen(DPoint::from((
                        points[last_idx].x as f32 + offset.x,
                        points[last_idx].y as f32 + offset.y,
                    )));
                    gizmos.line_2d(next_on_curve_pos, last_pos, handle_color);
                }

                // Move to the next segment
                current_idx = next_idx;
            } else {
                // Just move to the next point if we didn't find a valid segment
                current_idx = (current_idx + 1) % points.len();
            }
        } else {
            // Skip off-curve points when searching for segment starts
            current_idx = (current_idx + 1) % points.len();
        }
    }
}

/// Draw a curve segment based on the number of points at a specific position
fn draw_curve_segment_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    points: &[&ContourPoint],
    color: Color,
    offset: Vec2,
) {
    if points.len() < 2 {
        return;
    }

    if points.len() == 2 {
        // Simple line segment between two on-curve points
        let start_pos = viewport
            .to_screen(DPoint::from((points[0].x as f32 + offset.x, points[0].y as f32 + offset.y)));
        let end_pos = viewport
            .to_screen(DPoint::from((points[1].x as f32 + offset.x, points[1].y as f32 + offset.y)));
        gizmos.line_2d(start_pos, end_pos, color);
        return;
    }

    // Check if we have a cubic Bezier curve pattern: on-curve, off-curve, off-curve, on-curve
    if points.len() == 4
        && is_on_curve(points[0])
        && !is_on_curve(points[1])
        && !is_on_curve(points[2])
        && is_on_curve(points[3])
    {
        draw_cubic_bezier(
            gizmos,
            viewport.to_screen(DPoint::from((
                points[0].x as f32 + offset.x,
                points[0].y as f32 + offset.y,
            ))),
            viewport.to_screen(DPoint::from((
                points[1].x as f32 + offset.x,
                points[1].y as f32 + offset.y,
            ))),
            viewport.to_screen(DPoint::from((
                points[2].x as f32 + offset.x,
                points[2].y as f32 + offset.y,
            ))),
            viewport.to_screen(DPoint::from((
                points[3].x as f32 + offset.x,
                points[3].y as f32 + offset.y,
            ))),
            color,
        );
        return;
    }

    // For other cases (e.g. multiple off-curve points), approximate with line segments
    // This is a fallback and should be improved for proper curve rendering
    for i in 0..points.len() - 1 {
        let start_pos = viewport
            .to_screen(DPoint::from((points[i].x as f32 + offset.x, points[i].y as f32 + offset.y)));
        let end_pos = viewport.to_screen(DPoint::from((
            points[i + 1].x as f32 + offset.x,
            points[i + 1].y as f32 + offset.y,
        )));
        gizmos.line_2d(start_pos, end_pos, color);
    }
}

/// Draw a cubic Bezier curve using line segments for approximation
fn draw_cubic_bezier(
    gizmos: &mut Gizmos,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    color: Color,
) {
    // Number of segments to use for approximation
    let segments = 16;

    // Calculate points along the curve using the cubic Bezier formula
    let mut last_point = p0;

    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        // Cubic Bezier formula: B(t) = (1-t)^3*P0 + 3*(1-t)^2*t*P1 + 3*(1-t)*t^2*P2 + t^3*P3
        let point = Vec2::new(
            mt3 * p0.x
                + 3.0 * mt2 * t * p1.x
                + 3.0 * mt * t2 * p2.x
                + t3 * p3.x,
            mt3 * p0.y
                + 3.0 * mt2 * t * p1.y
                + 3.0 * mt * t2 * p2.y
                + t3 * p3.y,
        );

        // Draw line segment from last point to current point
        gizmos.line_2d(last_point, point, color);
        last_point = point;
    }
}

/// Helper function to check if a point is on-curve
fn is_on_curve(point: &ContourPoint) -> bool {
    matches!(
        point.typ,
        norad::PointType::Move
            | norad::PointType::Line
            | norad::PointType::Curve
            | norad::PointType::QCurve
    )
} 