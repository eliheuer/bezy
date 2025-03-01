//! Drawing algorithms and helpers

use crate::data::{AppState, FontMetrics};
use crate::design_space::{DPoint, ViewPort};
use crate::theme::{
    OFF_CURVE_POINT_COLOR, OFF_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR,
    ON_CURVE_POINT_RADIUS, PATH_LINE_COLOR, USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;
use norad::Glyph;

/// System that draws basic test elements for development
pub fn draw_test_elements(mut gizmos: Gizmos) {
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

/// System that draws font metrics
pub fn draw_metrics_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    cli_args: Res<crate::cli::CliArgs>,
) {
    // Debug metrics info
    info!("=== Font Metrics Debug ===");
    info!("Has font_info: {}", app_state.workspace.font.ufo.font_info.is_some());
    
    // Get the primary viewport or create a default one if none exists
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => {
            info!("No viewport found, using default viewport");
            ViewPort::default()
        },
    };
    
    // We need a glyph to draw metrics for
    if app_state.workspace.font.ufo.font_info.is_some() {
        // Debug metrics values
        let metrics = &app_state.workspace.info.metrics;
        info!("Units per em: {}", metrics.units_per_em);
        info!("X-height: {:?}", metrics.x_height);
        info!("Cap-height: {:?}", metrics.cap_height);
        info!("Ascender: {:?}", metrics.ascender);
        info!("Descender: {:?}", metrics.descender);
        
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
                        info!("Metrics drawn for glyph '{}' with advance width: {:?}", 
                              glyph.name, glyph.advance.as_ref().map(|a| a.width));
                    }
                    None => {
                        // Try with some common glyphs
                        let common_glyphs =
                            ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
                        let mut found = false;

                        for glyph_name_str in common_glyphs.iter() {
                            let name = norad::GlyphName::from(*glyph_name_str);
                            if let Some(glyph) = default_layer.get_glyph(&name) {
                                draw_metrics(
                                    &mut gizmos,
                                    viewport,
                                    glyph,
                                    &app_state.workspace.info.metrics,
                                );
                                info!("Metrics drawn for glyph '{}' with advance width: {:?}", 
                                      glyph.name, glyph.advance.as_ref().map(|a| a.width));
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // If no glyph found, use a placeholder with a warning
                            let mut placeholder = Glyph::new_named("placeholder");
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
        
        info!("Metrics drawn for viewport at zoom: {}, flipped_y: {}", viewport.zoom, viewport.flipped_y);
    } else {
        info!("No font info available, metrics not drawn");
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

    // Metrics color - bright forest green
    let metrics_color = Color::srgba(0.0, 0.9, 0.3, 0.9);

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
                    // For off-curve points or if squares are disabled, draw a filled circle
                    gizmos.circle_2d(screen_pos, size, color);
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
        while last_on_curve_idx > 0 && !is_on_curve(&points[last_on_curve_idx]) {
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
            
            let start_pos = viewport.to_screen(DPoint::from((start_point.x as f32, start_point.y as f32)));
            let end_pos = viewport.to_screen(DPoint::from((end_point.x as f32, end_point.y as f32)));
            
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
    
    let handle_color = Color::rgba(0.7, 0.7, 0.7, 0.6); // Light gray, semi-transparent
    
    // Find previous and next on-curve points for each off-curve point
    for (i, point) in points.iter().enumerate() {
        if !is_on_curve(point) {
            // This is an off-curve point, draw handles to adjacent on-curve points
            let mut prev_idx = i;
            while prev_idx > 0 && !is_on_curve(&points[prev_idx - 1]) {
                prev_idx -= 1;
            }
            
            if prev_idx == 0 && !is_on_curve(&points[prev_idx]) {
                // Wrap around to the end of the contour
                prev_idx = points.len() - 1;
                while prev_idx > 0 && !is_on_curve(&points[prev_idx]) {
                    prev_idx -= 1;
                }
            }
            
            let off_curve_pos = viewport.to_screen(DPoint::from((point.x as f32, point.y as f32)));
            
            // Draw handle to previous on-curve point if it exists
            if is_on_curve(&points[prev_idx]) {
                let prev_on_curve = &points[prev_idx];
                let prev_pos = viewport.to_screen(DPoint::from((prev_on_curve.x as f32, prev_on_curve.y as f32)));
                gizmos.line_2d(off_curve_pos, prev_pos, handle_color);
            }
            
            // Draw handle to next on-curve point
            let mut next_idx = i;
            while next_idx < points.len() - 1 && !is_on_curve(&points[next_idx + 1]) {
                next_idx += 1;
            }
            
            if next_idx == points.len() - 1 && !is_on_curve(&points[next_idx]) {
                // Wrap around to the beginning of the contour
                next_idx = 0;
                while next_idx < points.len() - 1 && !is_on_curve(&points[next_idx]) {
                    next_idx += 1;
                }
            }
            
            if is_on_curve(&points[next_idx]) {
                let next_on_curve = &points[next_idx];
                let next_pos = viewport.to_screen(DPoint::from((next_on_curve.x as f32, next_on_curve.y as f32)));
                gizmos.line_2d(off_curve_pos, next_pos, handle_color);
            }
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
        let start_pos = viewport.to_screen(DPoint::from((points[0].x as f32, points[0].y as f32)));
        let end_pos = viewport.to_screen(DPoint::from((points[1].x as f32, points[1].y as f32)));
        gizmos.line_2d(start_pos, end_pos, color);
        return;
    }
    
    // For cubic curve (4 points: on-curve, off-curve, off-curve, on-curve)
    if points.len() == 4 &&
       is_on_curve(points[0]) && !is_on_curve(points[1]) && 
       !is_on_curve(points[2]) && is_on_curve(points[3]) {
        draw_cubic_bezier(
            gizmos,
            viewport.to_screen(DPoint::from((points[0].x as f32, points[0].y as f32))),
            viewport.to_screen(DPoint::from((points[1].x as f32, points[1].y as f32))),
            viewport.to_screen(DPoint::from((points[2].x as f32, points[2].y as f32))),
            viewport.to_screen(DPoint::from((points[3].x as f32, points[3].y as f32))),
            color,
        );
        return;
    }
    
    // For other cases (e.g. multiple off-curve points), approximate with line segments
    // This is a fallback and should be improved for proper curve rendering
    for i in 0..points.len() - 1 {
        let start_pos = viewport.to_screen(DPoint::from((points[i].x as f32, points[i].y as f32)));
        let end_pos = viewport.to_screen(DPoint::from((points[i+1].x as f32, points[i+1].y as f32)));
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
            mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
            mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
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
        norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve | norad::PointType::QCurve
    )
}
