//! Glyph outline rendering functions
//!
//! Renders glyph outlines with proper cubic Bézier curves, control points, and handles.
//! This uses our thread-safe FontData structures for performance.
//! 
//! IMPORTANT: Includes live rendering mode that reads from Transform components
//! during nudging to ensure perfect synchronization between points and outline.

use crate::core::state::{ContourData, OutlineData, PointData, PointTypeData};
use crate::editing::selection::components::GlyphPointReference;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::{
    HANDLE_LINE_COLOR, OFF_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_POINT_COLOR,
    OFF_CURVE_POINT_RADIUS, ON_CURVE_INNER_CIRCLE_RATIO, ON_CURVE_POINT_COLOR,
    ON_CURVE_POINT_RADIUS, ON_CURVE_SQUARE_ADJUSTMENT, PATH_STROKE_COLOR,
    USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Draw a complete glyph outline at a specific design-space position
pub fn draw_glyph_outline_at_position(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    offset: Vec2,
) {
    let Some(outline) = outline else { return };
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }
        // Draw only the actual path with proper cubic curves (no control handles)
        draw_contour_path_at_position(gizmos, contour, offset);
    }
}

/// Draw glyph points AND control handles at a specific design-space position
/// This should be called when you want to show the editing interface (active sorts)
pub fn draw_glyph_points_and_handles_at_position(
    gizmos: &mut Gizmos,
    outline: &OutlineData,
    offset: Vec2,
) {
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }
        // Draw control handles
        draw_control_handles_at_position(gizmos, contour, offset);
        // Draw points
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

/// Draw glyph points (on-curve and off-curve) at a specific design-space position
/// This draws only the points, without control handles
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
    if points.is_empty() {
        return;
    }

    // Find segments between on-curve points using the sophisticated logic from backup
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

    let path_color = PATH_STROKE_COLOR;

    // Iterate through all points to identify and draw segments
    for i in 0..points.len() + 1 {
        let point_idx = i % points.len();
        let is_on = is_on_curve(&points[point_idx]);

        if is_on && last_was_on_curve {
            // If we have two consecutive on-curve points, draw a straight line
            let start_point = &points[segment_start_idx];
            let end_point = &points[point_idx];

            let start_pos =
                Vec2::new(start_point.x as f32, start_point.y as f32) + offset;
            let end_pos =
                Vec2::new(end_point.x as f32, end_point.y as f32) + offset;

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
            draw_curve_segment_at_position(
                gizmos,
                &segment_points,
                path_color,
                offset,
            );

            // Update for next segment
            segment_start_idx = point_idx;
        }

        last_was_on_curve = is_on;
    }
}

/// Draw control handles for off-curve points at a specific design-space position
pub fn draw_control_handles_at_position(
    gizmos: &mut Gizmos,
    contour: &ContourData,
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
            let current_on_curve_pos = Vec2::new(
                points[current_idx].x as f32,
                points[current_idx].y as f32,
            ) + offset;

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
                let next_on_curve_pos = Vec2::new(
                    points[next_idx].x as f32,
                    points[next_idx].y as f32,
                ) + offset;

                // For cubic Bézier with 2 control points (most common case)
                if off_curve_points.len() == 2 {
                    // First control point connects back to the current on-curve point
                    let p1_idx = off_curve_points[0];
                    let p1_pos = Vec2::new(
                        points[p1_idx].x as f32,
                        points[p1_idx].y as f32,
                    ) + offset;
                    gizmos.line_2d(current_on_curve_pos, p1_pos, handle_color);

                    // Second control point connects forward to the next on-curve point
                    let p2_idx = off_curve_points[1];
                    let p2_pos = Vec2::new(
                        points[p2_idx].x as f32,
                        points[p2_idx].y as f32,
                    ) + offset;
                    gizmos.line_2d(next_on_curve_pos, p2_pos, handle_color);
                }
                // For quadratic Bézier or other cases with just one control point
                else if off_curve_points.len() == 1 {
                    // The single control point gets a handle from the current on-curve point
                    let control_idx = off_curve_points[0];
                    let control_pos = Vec2::new(
                        points[control_idx].x as f32,
                        points[control_idx].y as f32,
                    ) + offset;
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
                    let first_pos = Vec2::new(
                        points[first_idx].x as f32,
                        points[first_idx].y as f32,
                    ) + offset;
                    gizmos.line_2d(
                        current_on_curve_pos,
                        first_pos,
                        handle_color,
                    );

                    // Connect last control point to the next on-curve point
                    let last_idx = off_curve_points[off_curve_points.len() - 1];
                    let last_pos = Vec2::new(
                        points[last_idx].x as f32,
                        points[last_idx].y as f32,
                    ) + offset;
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

/// Draw a curve segment based on the number of points at a specific design-space position
fn draw_curve_segment_at_position(
    gizmos: &mut Gizmos,
    points: &[&PointData],
    color: Color,
    offset: Vec2,
) {
    if points.len() < 2 {
        return;
    }

    if points.len() == 2 {
        // Simple line segment between two on-curve points
        let start_pos =
            Vec2::new(points[0].x as f32, points[0].y as f32) + offset;
        let end_pos =
            Vec2::new(points[1].x as f32, points[1].y as f32) + offset;
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
            Vec2::new(points[0].x as f32, points[0].y as f32) + offset,
            Vec2::new(points[1].x as f32, points[1].y as f32) + offset,
            Vec2::new(points[2].x as f32, points[2].y as f32) + offset,
            Vec2::new(points[3].x as f32, points[3].y as f32) + offset,
            color,
        );
        return;
    }

    // For other cases (e.g. multiple off-curve points), approximate with line segments
    // This is a fallback and should be improved for proper curve rendering
    for i in 0..points.len() - 1 {
        let start_pos =
            Vec2::new(points[i].x as f32, points[i].y as f32) + offset;
        let end_pos =
            Vec2::new(points[i + 1].x as f32, points[i + 1].y as f32) + offset;
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
    let segments = 32; // Increased from 16 for smoother curves

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

/// Check if a point is on the curve
fn is_on_curve(point: &PointData) -> bool {
    matches!(
        point.point_type,
        PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve
    )
}

/// LIVE RENDERING: Draw glyph outline using current Transform positions
/// This ensures perfect sync between points and outline during nudging
pub fn draw_glyph_outline_from_live_transforms(
    gizmos: &mut Gizmos,
    _sort_entity: Entity,
    sort_transform: &Transform,
    glyph_name: &str,
    point_query: &Query<(
        Entity,
        &Transform,
        &GlyphPointReference,
        &crate::editing::selection::components::PointType,
    ), With<SortPointEntity>>,
    app_state: &crate::core::state::AppState,
    selected_query: &Query<Entity, With<crate::editing::selection::components::Selected>>,
) {
    // Get the original glyph structure to understand point organization
    let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) else {
        return;
    };
    let Some(outline) = &glyph_data.outline else {
        return;
    };

    // Build a map of live positions from Transform components
    let mut live_positions = HashMap::new();
    for (_entity, transform, point_ref, point_type) in point_query.iter() {
        if point_ref.glyph_name == glyph_name {
            let world_pos = transform.translation.truncate();
            let sort_pos = sort_transform.translation.truncate();
            let relative_pos = world_pos - sort_pos;
            
            live_positions.insert(
                (point_ref.contour_index, point_ref.point_index),
                (relative_pos, point_type.is_on_curve),
            );
        }
    }

    // Render each contour using live positions
    for (contour_idx, contour) in outline.contours.iter().enumerate() {
        if contour.points.is_empty() {
            continue;
        }

        // Build live contour data
        let mut live_points = Vec::new();
        for (point_idx, original_point) in contour.points.iter().enumerate() {
            if let Some((live_pos, is_on_curve)) = live_positions.get(&(contour_idx, point_idx)) {
                // Use live position
                live_points.push(PointData {
                    x: live_pos.x as f64,
                    y: live_pos.y as f64,
                    point_type: if *is_on_curve {
                        PointTypeData::Curve // Use curve for on-curve points
                    } else {
                        PointTypeData::OffCurve // Use off-curve for control points
                    },
                });
            } else {
                // Fallback to original position
                live_points.push(original_point.clone());
            }
        }

        // Create live contour and render it
        let live_contour = ContourData {
            points: live_points,
        };
        
        draw_contour_path_at_position(gizmos, &live_contour, sort_transform.translation.truncate());
    }

    // CRITICAL: Also draw the individual points when in live mode
    // This replaces the separate point rendering systems during nudging
    for (entity, transform, point_ref, point_type) in point_query.iter() {
        if point_ref.glyph_name == glyph_name {
            let position = transform.translation.truncate();
            
            // Check if this point is selected
            let is_selected = selected_query.get(entity).is_ok();
            
            // Use selected colors if selected, normal colors if not
            let (size, color) = if is_selected {
                // Selected points use yellow color but normal size
                if point_type.is_on_curve {
                    (crate::ui::theme::ON_CURVE_POINT_RADIUS, crate::ui::theme::SELECTED_POINT_COLOR)
                } else {
                    (crate::ui::theme::OFF_CURVE_POINT_RADIUS, crate::ui::theme::SELECTED_POINT_COLOR)
                }
            } else {
                // Normal point colors
                if point_type.is_on_curve {
                    (crate::ui::theme::ON_CURVE_POINT_RADIUS, crate::ui::theme::ON_CURVE_POINT_COLOR)
                } else {
                    (crate::ui::theme::OFF_CURVE_POINT_RADIUS, crate::ui::theme::OFF_CURVE_POINT_COLOR)
                }
            };

            // Draw point as circle or square based on type, preserving crosshairs
            if point_type.is_on_curve && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE {
                let half_size = size / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT;
                let square_size = Vec2::new(size * 2.0, size * 2.0);
                gizmos.rect_2d(position, square_size, color);
                let inner_radius = half_size * crate::ui::theme::ON_CURVE_INNER_CIRCLE_RATIO;
                gizmos.circle_2d(position, inner_radius, color);
            } else {
                gizmos.circle_2d(position, size, color);
                let inner_radius = size * crate::ui::theme::OFF_CURVE_INNER_CIRCLE_RATIO;
                gizmos.circle_2d(position, inner_radius, color);
            }
        }
    }

    // CRITICAL: Also render control handles when in live mode
    // This replaces the separate control handle rendering system during nudging
    render_live_control_handles(gizmos, glyph_name, point_query, app_state);
}

/// Render control handles using live Transform positions during nudging
fn render_live_control_handles(
    gizmos: &mut Gizmos,
    glyph_name: &str,
    point_query: &Query<(
        Entity,
        &Transform,
        &GlyphPointReference,
        &crate::editing::selection::components::PointType,
    ), With<SortPointEntity>>,
    app_state: &crate::core::state::AppState,
) {
    let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) else {
        return;
    };
    let Some(outline) = &glyph_data.outline else {
        return;
    };

    // Group points by contour to process them together
    let mut contour_points: Vec<Vec<(Vec2, bool, usize)>> =
        vec![Vec::new(); outline.contours.len()];

    // Collect all point entities and their positions using live Transform data
    for (_entity, transform, glyph_ref, point_type) in point_query.iter() {
        if glyph_ref.glyph_name == glyph_name {
            let position = transform.translation.truncate();
            let is_on_curve = point_type.is_on_curve;
            let contour_index = glyph_ref.contour_index;
            let point_index = glyph_ref.point_index;

            if contour_index < contour_points.len() {
                contour_points[contour_index].push((
                    position,
                    is_on_curve,
                    point_index,
                ));
            }
        }
    }

    // Sort points within each contour by their original index
    for contour_points in &mut contour_points {
        contour_points.sort_by_key(|(_, _, index)| *index);
    }

    // Render handles for each contour using the same logic as the original system
    for contour_points in contour_points {
        if contour_points.len() < 2 {
            continue;
        }

        render_live_contour_handles(gizmos, &contour_points);
    }
}

/// Render handles for a single contour using live data
fn render_live_contour_handles(
    gizmos: &mut Gizmos,
    contour_points: &[(Vec2, bool, usize)],
) {
    let len = contour_points.len();
    
    for i in 0..len {
        let (current_pos, current_on_curve, _) = contour_points[i];
        
        // Check next point (wrapping around)
        let next_i = (i + 1) % len;
        let (next_pos, next_on_curve, _) = contour_points[next_i];
        
        // Draw handle line if one point is on-curve and the other is off-curve
        if current_on_curve != next_on_curve {
            gizmos.line_2d(
                current_pos,
                next_pos,
                crate::ui::theme::HANDLE_LINE_COLOR,
            );
        }
    }
}
