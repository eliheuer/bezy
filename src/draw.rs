//! Drawing algorithms and helpers

use crate::data::{AppState, FontMetrics};
use crate::design_space::{DPoint, ViewPort};
use crate::theme::{
    DEBUG_SHOW_ORIGIN_CROSS, METRICS_GUIDE_COLOR, OFF_CURVE_POINT_COLOR,
    OFF_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS, PATH_LINE_COLOR,
    USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;
use norad::Glyph;

/// System that draws basic test elements for development
pub fn draw_test_elements(mut gizmos: Gizmos) {
    // Only draw the debug cross if enabled in theme settings
    if DEBUG_SHOW_ORIGIN_CROSS {
        // Draw a simple test cross at the origin
        gizmos.line_2d(
            Vec2::new(-64.0, 0.0),
            Vec2::new(64.0, 0.0),
            Color::srgb(1.0, 0.0, 0.0),
        );
        gizmos.line_2d(
            Vec2::new(0.0, -64.0),
            Vec2::new(0.0, 64.0),
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

/// System that draws font metrics
pub fn draw_metrics_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    cli_args: Res<crate::cli::CliArgs>,
) {
    // Debug metrics info
    debug!("=== Font Metrics Debug ===");
    debug!(
        "Has font_info: {}",
        app_state.workspace.font.ufo.font_info.is_some()
    );

    // Get the primary viewport or create a default one if none exists
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => {
            debug!("No viewport found, using default viewport");
            ViewPort::default()
        }
    };

    // We need a glyph to draw metrics for
    if app_state.workspace.font.ufo.font_info.is_some() {
        // Debug metrics values
        let metrics = &app_state.workspace.info.metrics;
        debug!("Units per em: {}", metrics.units_per_em);
        debug!("X-height: {:?}", metrics.x_height);
        debug!("Cap-height: {:?}", metrics.cap_height);
        debug!("Ascender: {:?}", metrics.ascender);
        debug!("Descender: {:?}", metrics.descender);

        // Get the test glyph name from CLI args
        let test_glyph = cli_args.get_test_glyph();

        match app_state.workspace.font.ufo.get_default_layer() {
            Some(default_layer) => {
                // Try to get the glyph directly by name
                let glyph_name = norad::GlyphName::from(test_glyph.clone());

                match default_layer.get_glyph(&glyph_name) {
                    Some(glyph) => {
                        // Draw the metrics using the actual glyph
                        draw_metrics(
                            &mut gizmos,
                            viewport,
                            glyph,
                            &app_state.workspace.info.metrics,
                        );
                        debug!("Metrics drawn for glyph '{}' with advance width: {:?}", 
                              glyph.name, glyph.advance.as_ref().map(|a| a.width));
                    }
                    None => {
                        // Try with some common glyphs
                        let common_glyphs =
                            ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
                        let mut found = false;

                        for glyph_name_str in common_glyphs.iter() {
                            let name = norad::GlyphName::from(*glyph_name_str);
                            if let Some(glyph) = default_layer.get_glyph(&name)
                            {
                                draw_metrics(
                                    &mut gizmos,
                                    viewport,
                                    glyph,
                                    &app_state.workspace.info.metrics,
                                );
                                debug!("Metrics drawn for glyph '{}' with advance width: {:?}", 
                                      glyph.name, glyph.advance.as_ref().map(|a| a.width));
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // If no glyph found, use a placeholder with a warning
                            let mut placeholder =
                                Glyph::new_named("placeholder");
                            placeholder.advance = Some(norad::Advance {
                                width: metrics.units_per_em as f32,
                                height: 0.0,
                            });

                            draw_metrics(
                                &mut gizmos,
                                viewport,
                                &placeholder,
                                &app_state.workspace.info.metrics,
                            );

                            println!("WARNING: Could not find any glyphs for metrics. Using units_per_em as placeholder width.");
                        }
                    }
                }
            }
            None => {
                println!("WARNING: No default layer found in the font");
            }
        }

        debug!(
            "Metrics drawn for viewport at zoom: {}, flipped_y: {}",
            viewport.zoom, viewport.flipped_y
        );
    } else {
        debug!("No font info available, metrics not drawn");
    }
}

/// Draw font metrics lines (baseline, x-height, cap-height, ascender, descender, and bounding box)
fn draw_metrics(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
) {
    let upm = metrics.units_per_em;
    let x_height = metrics.x_height.unwrap_or_else(|| (upm * 0.5).round());
    let cap_height = metrics.cap_height.unwrap_or_else(|| (upm * 0.7).round());
    let ascender = metrics.ascender.unwrap_or_else(|| (upm * 0.8).round());
    let descender = metrics.descender.unwrap_or_else(|| -(upm * 0.2).round());
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f64)
        .unwrap_or_else(|| (upm * 0.5).round());

    // Use the constant from theme.rs instead of hardcoding the color
    let metrics_color = METRICS_GUIDE_COLOR;

    // Draw the bounding box that represents the glyph metrics
    draw_rect(
        gizmos,
        viewport,
        (0.0, descender as f32),
        (width as f32, ascender as f32),
        metrics_color,
    );

    // Draw baseline
    draw_line(
        gizmos,
        viewport,
        (0.0, 0.0),
        (width as f32, 0.0),
        metrics_color,
    );

    // Draw x-height line
    draw_line(
        gizmos,
        viewport,
        (0.0, x_height as f32),
        (width as f32, x_height as f32),
        metrics_color,
    );

    // Draw cap-height line
    draw_line(
        gizmos,
        viewport,
        (0.0, cap_height as f32),
        (width as f32, cap_height as f32),
        metrics_color,
    );

    // Draw ascender line
    draw_line(
        gizmos,
        viewport,
        (0.0, ascender as f32),
        (width as f32, ascender as f32),
        metrics_color,
    );

    // Draw descender line
    draw_line(
        gizmos,
        viewport,
        (0.0, descender as f32),
        (width as f32, descender as f32),
        metrics_color,
    );
}

/// Draw a line in design space
fn draw_line(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    let start_screen = viewport.to_screen(DPoint::from(start));
    let end_screen = viewport.to_screen(DPoint::from(end));
    gizmos.line_2d(start_screen, end_screen, color);
}

/// Draw a rectangle in design space
fn draw_rect(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    top_left: (f32, f32),
    bottom_right: (f32, f32),
    color: Color,
) {
    let tl_screen = viewport.to_screen(DPoint::from(top_left));
    let br_screen = viewport.to_screen(DPoint::from(bottom_right));

    // Draw the rectangle outline (four lines)
    gizmos.line_2d(
        Vec2::new(tl_screen.x, tl_screen.y),
        Vec2::new(br_screen.x, tl_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(br_screen.x, tl_screen.y),
        Vec2::new(br_screen.x, br_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(br_screen.x, br_screen.y),
        Vec2::new(tl_screen.x, br_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(tl_screen.x, br_screen.y),
        Vec2::new(tl_screen.x, tl_screen.y),
        color,
    );
}

/// Plugin to add drawing systems
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_test_elements,
                draw_metrics_system,
                draw_glyph_points_system,
            ),
        );
    }
}

/// System that draws points for a specific Unicode character
pub fn draw_glyph_points_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    cli_args: Res<crate::cli::CliArgs>,
) {
    // Get the primary viewport or create a default one if none exists
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };

    // Get the test glyph name from CLI args
    let test_glyph = cli_args.get_test_glyph();

    match app_state.workspace.font.ufo.get_default_layer() {
        Some(default_layer) => {
            // Try to get the glyph directly by name
            let glyph_name = norad::GlyphName::from(test_glyph.clone());

            match default_layer.get_glyph(&glyph_name) {
                Some(glyph) => {
                    // Draw the points
                    draw_glyph_points(&mut gizmos, viewport, glyph);
                }
                None => {
                    // Try with some common glyphs
                    let common_glyphs =
                        ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
                    let mut found = false;

                    for glyph_name_str in common_glyphs.iter() {
                        let name = norad::GlyphName::from(*glyph_name_str);
                        if let Some(glyph) = default_layer.get_glyph(&name) {
                            draw_glyph_points(&mut gizmos, viewport, glyph);
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        println!(
                            "WARNING: Could not find any common test glyphs"
                        );
                    }
                }
            }
        }
        None => {
            println!("WARNING: No default layer found in the font");
        }
    }
}

/// Draw points from a glyph
fn draw_glyph_points(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    glyph: &norad::Glyph,
) {
    // Only proceed if the glyph has an outline
    if let Some(outline) = &glyph.outline {
        // Iterate through all contours
        for (_contour_idx, contour) in outline.contours.iter().enumerate() {
            if contour.points.is_empty() {
                continue;
            }

            // First, draw the actual path with proper cubic curves
            draw_contour_path(gizmos, viewport, contour);

            // Then draw the control handles for off-curve points
            draw_control_handles(gizmos, viewport, contour);

            // Finally, draw the points themselves
            for point in contour.points.iter() {
                let point_pos = (point.x as f32, point.y as f32);
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
                    // For on-curve points, draw a square outline
                    let half_size = size / 1.4; // Adjusting size for visual balance

                    // Draw a filled square by first drawing a circle fill then the square outline
                    // First draw a filled circle inside the square
                    gizmos.circle_2d(screen_pos, half_size * 0.5, color);

                    // Then draw the square outline
                    let top_left = Vec2::new(
                        screen_pos.x - half_size,
                        screen_pos.y + half_size,
                    );
                    let top_right = Vec2::new(
                        screen_pos.x + half_size,
                        screen_pos.y + half_size,
                    );
                    let bottom_right = Vec2::new(
                        screen_pos.x + half_size,
                        screen_pos.y - half_size,
                    );
                    let bottom_left = Vec2::new(
                        screen_pos.x - half_size,
                        screen_pos.y - half_size,
                    );

                    // Draw the square sides
                    gizmos.line_2d(top_left, top_right, color);
                    gizmos.line_2d(top_right, bottom_right, color);
                    gizmos.line_2d(bottom_right, bottom_left, color);
                    gizmos.line_2d(bottom_left, top_left, color);
                } else {
                    // For off-curve points, draw a filled circle with a smaller circle inside
                    // First draw the outer circle
                    gizmos.circle_2d(screen_pos, size, color);
                    
                    // Then draw a smaller inner circle with the same color
                    // Draw the inner circle at 40% of the original size
                    gizmos.circle_2d(screen_pos, size * 0.4, color);
                }
            }
        }
    } else {
        println!("Glyph '{}' has no outline", glyph.name);
    }
}

/// Draw the contour path with proper cubic curves
fn draw_contour_path(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    contour: &norad::Contour,
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
                start_point.x as f32,
                start_point.y as f32,
            )));
            let end_pos = viewport.to_screen(DPoint::from((
                end_point.x as f32,
                end_point.y as f32,
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
            draw_curve_segment(gizmos, viewport, &segment_points, path_color);

            // Update for next segment
            segment_start_idx = point_idx;
        }

        last_was_on_curve = is_on;
    }
}

/// Draw control handles for off-curve points
fn draw_control_handles(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    contour: &norad::Contour,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    let handle_color = Color::srgba(0.7, 0.7, 0.7, 0.6); // Light gray, semi-transparent

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
                points[current_idx].x as f32,
                points[current_idx].y as f32,
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
                    points[next_idx].x as f32,
                    points[next_idx].y as f32,
                )));

                // For cubic Bézier with 2 control points (most common case)
                if off_curve_points.len() == 2 {
                    // First control point connects back to the current on-curve point
                    let p1_idx = off_curve_points[0];
                    let p1_pos = viewport.to_screen(DPoint::from((
                        points[p1_idx].x as f32,
                        points[p1_idx].y as f32,
                    )));
                    gizmos.line_2d(current_on_curve_pos, p1_pos, handle_color);

                    // Second control point connects forward to the next on-curve point
                    let p2_idx = off_curve_points[1];
                    let p2_pos = viewport.to_screen(DPoint::from((
                        points[p2_idx].x as f32,
                        points[p2_idx].y as f32,
                    )));
                    gizmos.line_2d(next_on_curve_pos, p2_pos, handle_color);
                }
                // For quadratic Bézier or other cases with just one control point
                else if off_curve_points.len() == 1 {
                    // The single control point gets a handle from the current on-curve point
                    let control_idx = off_curve_points[0];
                    let control_pos = viewport.to_screen(DPoint::from((
                        points[control_idx].x as f32,
                        points[control_idx].y as f32,
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
                        points[first_idx].x as f32,
                        points[first_idx].y as f32,
                    )));
                    gizmos.line_2d(
                        current_on_curve_pos,
                        first_pos,
                        handle_color,
                    );

                    // Connect last control point to the next on-curve point
                    let last_idx = off_curve_points[off_curve_points.len() - 1];
                    let last_pos = viewport.to_screen(DPoint::from((
                        points[last_idx].x as f32,
                        points[last_idx].y as f32,
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

/// Draw a curve segment based on the number of points
fn draw_curve_segment(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    points: &[&norad::ContourPoint],
    color: Color,
) {
    if points.len() < 2 {
        return;
    }

    if points.len() == 2 {
        // Simple line segment between two on-curve points
        let start_pos = viewport
            .to_screen(DPoint::from((points[0].x as f32, points[0].y as f32)));
        let end_pos = viewport
            .to_screen(DPoint::from((points[1].x as f32, points[1].y as f32)));
        gizmos.line_2d(start_pos, end_pos, color);
        return;
    }

    // For cubic curve (4 points: on-curve, off-curve, off-curve, on-curve)
    if points.len() == 4
        && is_on_curve(points[0])
        && !is_on_curve(points[1])
        && !is_on_curve(points[2])
        && is_on_curve(points[3])
    {
        draw_cubic_bezier(
            gizmos,
            viewport.to_screen(DPoint::from((
                points[0].x as f32,
                points[0].y as f32,
            ))),
            viewport.to_screen(DPoint::from((
                points[1].x as f32,
                points[1].y as f32,
            ))),
            viewport.to_screen(DPoint::from((
                points[2].x as f32,
                points[2].y as f32,
            ))),
            viewport.to_screen(DPoint::from((
                points[3].x as f32,
                points[3].y as f32,
            ))),
            color,
        );
        return;
    }

    // For other cases (e.g. multiple off-curve points), approximate with line segments
    // This is a fallback and should be improved for proper curve rendering
    for i in 0..points.len() - 1 {
        let start_pos = viewport
            .to_screen(DPoint::from((points[i].x as f32, points[i].y as f32)));
        let end_pos = viewport.to_screen(DPoint::from((
            points[i + 1].x as f32,
            points[i + 1].y as f32,
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
fn is_on_curve(point: &norad::ContourPoint) -> bool {
    matches!(
        point.typ,
        norad::PointType::Move
            | norad::PointType::Line
            | norad::PointType::Curve
            | norad::PointType::QCurve
    )
}
