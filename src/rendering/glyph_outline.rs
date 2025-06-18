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
    contours: &[Contour],
    offset: Vec2,
) {
    // Render each contour in the outline
    for contour in contours {
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
    contours: &[Contour],
    offset: Vec2,
) {
    // Render each contour's points
    for contour in contours {
        for point in &contour.points {
            let world_pos = offset + Vec2::new(point.x as f32, point.y as f32);
            let screen_pos = viewport.to_screen(DPoint::new(world_pos.x, world_pos.y));
            
            let is_on_curve = matches!(point.typ, norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve);
            
            if is_on_curve {
                draw_on_curve_point(gizmos, screen_pos, ON_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR);
            } else {
                draw_off_curve_point(gizmos, screen_pos, OFF_CURVE_POINT_RADIUS, OFF_CURVE_POINT_COLOR);
            }
        }
    }
}

/// Draw a contour path at a specific position
pub fn draw_contour_path_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &Contour,
    offset: Vec2,
) {
    if contour.points.is_empty() {
        return;
    }

    let points: Vec<_> = contour.points.iter().collect();
    let mut i = 0;

    while i < points.len() {
        let current = points[i];
        let next_index = (i + 1) % points.len();
        let next = points[next_index];

        let start_world = offset + Vec2::new(current.x as f32, current.y as f32);
        let start_screen = viewport.to_screen(DPoint::new(start_world.x, start_world.y));

        match next.typ {
            norad::PointType::Line | norad::PointType::Move => {
                // Draw straight line
                let end_world = offset + Vec2::new(next.x as f32, next.y as f32);
                let end_screen = viewport.to_screen(DPoint::new(end_world.x, end_world.y));
                
                gizmos.line_2d(
                    Vec2::new(start_screen.x as f32, start_screen.y as f32),
                    Vec2::new(end_screen.x as f32, end_screen.y as f32),
                    PATH_LINE_COLOR,
                );
                i += 1;
            }
            norad::PointType::Curve | norad::PointType::QCurve => {
                // Find the curve segment - look for off-curve points before the on-curve point
                let mut curve_points = vec![current];
                let mut j = (i + 1) % points.len();
                
                // Collect off-curve points
                while j != next_index && matches!(points[j].typ, norad::PointType::OffCurve) {
                    curve_points.push(points[j]);
                    j = (j + 1) % points.len();
                }
                
                // Add the final on-curve point
                curve_points.push(next);
                
                // Draw the curve based on number of control points
                match curve_points.len() {
                    2 => {
                        // No control points - draw line
                        let end_world = offset + Vec2::new(next.x as f32, next.y as f32);
                        let end_screen = viewport.to_screen(DPoint::new(end_world.x, end_world.y));
                        
                        gizmos.line_2d(
                            Vec2::new(start_screen.x as f32, start_screen.y as f32),
                            Vec2::new(end_screen.x as f32, end_screen.y as f32),
                            PATH_LINE_COLOR,
                        );
                    }
                    3 => {
                        // One control point - quadratic curve
                        let control_world = offset + Vec2::new(curve_points[1].x as f32, curve_points[1].y as f32);
                        let control_screen = viewport.to_screen(DPoint::new(control_world.x, control_world.y));
                        let end_world = offset + Vec2::new(curve_points[2].x as f32, curve_points[2].y as f32);
                        let end_screen = viewport.to_screen(DPoint::new(end_world.x, end_world.y));
                        
                        draw_quadratic_curve(
                            gizmos,
                            Vec2::new(start_screen.x as f32, start_screen.y as f32),
                            Vec2::new(control_screen.x as f32, control_screen.y as f32),
                            Vec2::new(end_screen.x as f32, end_screen.y as f32),
                            PATH_LINE_COLOR,
                        );
                    }
                    4 => {
                        // Two control points - cubic curve
                        let control1_world = offset + Vec2::new(curve_points[1].x as f32, curve_points[1].y as f32);
                        let control1_screen = viewport.to_screen(DPoint::new(control1_world.x, control1_world.y));
                        let control2_world = offset + Vec2::new(curve_points[2].x as f32, curve_points[2].y as f32);
                        let control2_screen = viewport.to_screen(DPoint::new(control2_world.x, control2_world.y));
                        let end_world = offset + Vec2::new(curve_points[3].x as f32, curve_points[3].y as f32);
                        let end_screen = viewport.to_screen(DPoint::new(end_world.x, end_world.y));
                        
                        draw_cubic_curve(
                            gizmos,
                            Vec2::new(start_screen.x as f32, start_screen.y as f32),
                            Vec2::new(control1_screen.x as f32, control1_screen.y as f32),
                            Vec2::new(control2_screen.x as f32, control2_screen.y as f32),
                            Vec2::new(end_screen.x as f32, end_screen.y as f32),
                            PATH_LINE_COLOR,
                        );
                    }
                    _ => {
                        // More complex curves - draw as line segments for now
                        let end_world = offset + Vec2::new(next.x as f32, next.y as f32);
                        let end_screen = viewport.to_screen(DPoint::new(end_world.x, end_world.y));
                        
                        gizmos.line_2d(
                            Vec2::new(start_screen.x as f32, start_screen.y as f32),
                            Vec2::new(end_screen.x as f32, end_screen.y as f32),
                            PATH_LINE_COLOR,
                        );
                    }
                }
                
                // Skip over the points we've processed
                i = j;
            }
            norad::PointType::OffCurve => {
                // This shouldn't happen in a well-formed contour
                i += 1;
            }
        }
    }
}

/// Draw control handles for off-curve points at a specific position
pub fn draw_control_handles_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &Contour,
    offset: Vec2,
) {
    let points: Vec<_> = contour.points.iter().collect();
    
    for (i, point) in points.iter().enumerate() {
        if matches!(point.typ, norad::PointType::OffCurve) {
            let point_world = offset + Vec2::new(point.x as f32, point.y as f32);
            let point_screen = viewport.to_screen(DPoint::new(point_world.x, point_world.y));
            
            // Find the connected on-curve points
            let prev_on_curve = find_previous_on_curve(&points, i);
            let next_on_curve = find_next_on_curve(&points, i);
            
            if let Some(prev_idx) = prev_on_curve {
                let prev_world = offset + Vec2::new(points[prev_idx].x as f32, points[prev_idx].y as f32);
                let prev_screen = viewport.to_screen(DPoint::new(prev_world.x, prev_world.y));
                
                gizmos.line_2d(
                    Vec2::new(prev_screen.x as f32, prev_screen.y as f32),
                    Vec2::new(point_screen.x as f32, point_screen.y as f32),
                    HANDLE_LINE_COLOR,
                );
            }
            
            if let Some(next_idx) = next_on_curve {
                let next_world = offset + Vec2::new(points[next_idx].x as f32, points[next_idx].y as f32);
                let next_screen = viewport.to_screen(DPoint::new(next_world.x, next_world.y));
                
                gizmos.line_2d(
                    Vec2::new(point_screen.x as f32, point_screen.y as f32),
                    Vec2::new(next_screen.x as f32, next_screen.y as f32),
                    HANDLE_LINE_COLOR,
                );
            }
        }
    }
}

/// Draw an on-curve point
pub fn draw_on_curve_point(gizmos: &mut Gizmos, position: Vec2, radius: f32, color: Color) {
    let pos = position;
    
    if USE_SQUARE_FOR_ON_CURVE {
        // Draw as square
        let half_size = radius * ON_CURVE_SQUARE_ADJUSTMENT;
        let min = pos - Vec2::splat(half_size);
        let max = pos + Vec2::splat(half_size);
        
        // Draw square outline
        gizmos.line_2d(Vec2::new(min.x, min.y), Vec2::new(max.x, min.y), color);
        gizmos.line_2d(Vec2::new(max.x, min.y), Vec2::new(max.x, max.y), color);
        gizmos.line_2d(Vec2::new(max.x, max.y), Vec2::new(min.x, max.y), color);
        gizmos.line_2d(Vec2::new(min.x, max.y), Vec2::new(min.x, min.y), color);
        
        // Draw inner circle
        let inner_radius = radius * ON_CURVE_INNER_CIRCLE_RATIO;
        gizmos.circle_2d(pos, inner_radius, color);
    } else {
        // Draw as circle
        gizmos.circle_2d(pos, radius, color);
        
        // Draw inner circle
        let inner_radius = radius * ON_CURVE_INNER_CIRCLE_RATIO;
        gizmos.circle_2d(pos, inner_radius, color);
    }
}

/// Draw an off-curve point
pub fn draw_off_curve_point(gizmos: &mut Gizmos, position: Vec2, radius: f32, color: Color) {
    let pos = position;
    
    // Draw as circle
    gizmos.circle_2d(pos, radius, color);
    
    // Draw inner circle
    let inner_radius = radius * OFF_CURVE_INNER_CIRCLE_RATIO;
    gizmos.circle_2d(pos, inner_radius, color);
}

/// Find the previous on-curve point index
fn find_previous_on_curve(points: &[&ContourPoint], start_index: usize) -> Option<usize> {
    let len = points.len();
    for offset in 1..len {
        let idx = (start_index + len - offset) % len;
        if matches!(points[idx].typ, norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve) {
            return Some(idx);
        }
    }
    None
}

/// Find the next on-curve point index
fn find_next_on_curve(points: &[&ContourPoint], start_index: usize) -> Option<usize> {
    let len = points.len();
    for offset in 1..len {
        let idx = (start_index + offset) % len;
        if matches!(points[idx].typ, norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve) {
            return Some(idx);
        }
    }
    None
}

/// Draw a quadratic Bezier curve
fn draw_quadratic_curve(
    gizmos: &mut Gizmos,
    start: Vec2,
    control: Vec2,
    end: Vec2,
    color: Color,
) {
    const SEGMENTS: usize = 20;
    let mut last_point = start;
    
    for i in 1..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let point = quadratic_bezier(start, control, end, t);
        gizmos.line_2d(last_point, point, color);
        last_point = point;
    }
}

/// Draw a cubic Bezier curve
fn draw_cubic_curve(
    gizmos: &mut Gizmos,
    start: Vec2,
    control1: Vec2,
    control2: Vec2,
    end: Vec2,
    color: Color,
) {
    const SEGMENTS: usize = 20;
    let mut last_point = start;
    
    for i in 1..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let point = cubic_bezier(start, control1, control2, end, t);
        gizmos.line_2d(last_point, point, color);
        last_point = point;
    }
}

/// Calculate a point on a quadratic Bezier curve
fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

/// Calculate a point on a cubic Bezier curve
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
} 