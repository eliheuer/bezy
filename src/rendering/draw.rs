//! Drawing algorithms and helpers

use crate::core::state::{AppState, FontMetrics, GlyphNavigation};
use crate::editing::selection::Selectable;
use crate::ui::panes::design_space::{DPoint, ViewPort};
use crate::ui::theme::{
    ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS, OFF_CURVE_POINT_COLOR,
    OFF_CURVE_POINT_RADIUS, PATH_LINE_COLOR, HANDLE_LINE_COLOR,
    ON_CURVE_SQUARE_ADJUSTMENT, USE_SQUARE_FOR_ON_CURVE,
    DEBUG_SHOW_ORIGIN_CROSS, OFF_CURVE_INNER_CIRCLE_RATIO,
    ON_CURVE_INNER_CIRCLE_RATIO,
};
use bevy::prelude::*;
use norad::Glyph;

/// System that draws the debug origin cross and square
pub fn draw_origin_cross(mut gizmos: Gizmos) {
    // Only draw the debug cross if enabled in theme settings
    if DEBUG_SHOW_ORIGIN_CROSS {
        let red = Color::srgb(1.0, 0.0, 0.0);
        
        // Draw a simple test cross at the origin using 2D gizmos to render on top of sorts
        gizmos.line_2d(
            Vec2::new(-64.0, 0.0),
            Vec2::new(64.0, 0.0),
            red,
        );
        gizmos.line_2d(
            Vec2::new(0.0, -64.0),
            Vec2::new(0.0, 64.0),
            red,
        );
        
        // Draw a 32x32 red square centered at origin
        gizmos.rect_2d(
            Vec2::ZERO, // position (center)
            Vec2::new(32.0, 32.0), // size
            red,
        );
    }
}

/// System that draws font metrics lines (baseline, x-height, cap-height, etc.)
pub fn draw_metrics_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    glyph_navigation: Res<GlyphNavigation>,
) {
    // Early exit if no font is loaded
    if app_state.workspace.font.ufo.font_info.is_none() {
        return;
    }

    // Get viewport for coordinate transformations
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };

    // Skip drawing if we're looking for a specific codepoint that wasn't found
    let codepoint_string = glyph_navigation.get_codepoint_string();
    if !codepoint_string.is_empty() && !glyph_navigation.codepoint_found {
        return;
    }

    // Get the default layer to search for glyphs
    let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() else {
        println!("WARNING: No default layer found in the font");
        return;
    };

    // Find a glyph to use for metrics display
    let glyph = find_glyph_for_metrics(&glyph_navigation, &app_state, default_layer);
    
    // Draw the metrics using the found or placeholder glyph
    draw_metrics(
        &mut gizmos,
        &viewport,
        &glyph,
        &app_state.workspace.info.metrics,
    );
}

/// Selects a glyph to determine the width for drawing font metrics lines.
///
/// The metrics lines (baseline, x-height, etc.) need a horizontal extent.
/// This function prioritizes the glyph currently active via `glyph_navigation`.
/// If no specific glyph is active or found, it falls back to a standard
/// placeholder glyph whose width is based on the font's units_per_em value.
fn find_glyph_for_metrics(
    glyph_navigation: &GlyphNavigation,
    app_state: &AppState,
    default_layer: &norad::Layer,
) -> Glyph {
    let ufo = &app_state.workspace.font.ufo;

    // Try to get the specifically requested glyph first.
    // If found, clone it and return immediately.
    if let Some(glyph_name) = glyph_navigation.find_glyph(ufo) {
        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
            return (**glyph).clone();
        }
    }

    // If no specific glyph is found (or the layer doesn't contain it),
    // fall back to creating a standard placeholder glyph.
    create_placeholder_glyph(&app_state.workspace.info.metrics)
}

/// Create a placeholder glyph when no real glyphs are available
fn create_placeholder_glyph(
    metrics: &crate::core::state::FontMetrics
) -> Glyph {
    let mut placeholder = Glyph::new_named("placeholder");
    placeholder.advance = Some(norad::Advance {
        width: metrics.units_per_em as f32,
        height: 0.0,
    });
    placeholder
}

/// Draw font metrics lines (baseline, x-height, cap-height, ascender, descender, and bounding box)
fn draw_metrics(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
) {
    // Use the shared metrics rendering function
    crate::rendering::metrics::draw_metrics_at_origin(gizmos, viewport, glyph, metrics);
}

/// Draw a line in design space
fn draw_line(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    // DPoint = Design space Point, converted to screen coordinates
    let start_screen = viewport.to_screen(DPoint::from(start));
    let end_screen = viewport.to_screen(DPoint::from(end));
    gizmos.line_2d(start_screen, end_screen, color);
}

/// Draw a rectangle in design space
fn draw_rect(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
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

/// Event that will be triggered when the AppState changes
#[derive(Event)]
pub struct AppStateChanged;

/// System that draws points for a specific Unicode character
pub fn draw_glyph_points_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    mut glyph_navigation: ResMut<GlyphNavigation>,
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    window_query: Query<&Window>,
    selected_points_query: Query<
        (
            Entity,
            &crate::editing::selection::components::GlyphPointReference,
        ),
        With<crate::editing::selection::components::Selected>,
    >,
) {
    // Get the primary viewport or create a default one if none exists
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };

    // Get the codepoint string
    let codepoint_string = glyph_navigation.get_codepoint_string();

    // If no font is loaded, return early
    if app_state.workspace.font.ufo.font_info.is_none() {
        return;
    }

    // Collect selected points for efficient checking
    let selected_points: Vec<_> = selected_points_query.iter().collect();

    match app_state.workspace.font.ufo.get_default_layer() {
        Some(default_layer) => {
            // Flag to track if we found the glyph
            let mut glyph_found = false;
            let mut found_glyph = None;

            // Try to find the glyph using the new helper method
            if let Some(glyph_name) =
                glyph_navigation.find_glyph(&app_state.workspace.font.ufo)
            {
                if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                    draw_glyph_points(
                        &mut gizmos,
                        &viewport,
                        glyph,
                        &selected_points,
                    );
                    glyph_navigation.codepoint_found = true;
                    glyph_found = true;
                    found_glyph = Some(glyph);
                }
            }

            // If not found and no specific codepoint requested, try common glyphs
            if !glyph_found && codepoint_string.is_empty() {
                let common_glyphs =
                    ["H", "h", "A", "a", "O", "o", "space", ".notdef"];

                for glyph_name_str in common_glyphs.iter() {
                    let name = norad::GlyphName::from(*glyph_name_str);
                    if let Some(glyph) = default_layer.get_glyph(&name) {
                        draw_glyph_points(
                            &mut gizmos,
                            &viewport,
                            glyph,
                            &selected_points,
                        );
                        glyph_navigation.codepoint_found = true;
                        glyph_found = true;
                        found_glyph = Some(glyph);
                        break;
                    }
                }
            }

            // Center the camera on the glyph if one was found
            // Only do this once when the application starts
            static mut CAMERA_CENTERED: bool = false;
            if let Some(glyph) = found_glyph {
                unsafe {
                    if !CAMERA_CENTERED {
                        crate::rendering::cameras::center_camera_on_glyph(
                            glyph,
                            &app_state.workspace.info.metrics,
                            &mut camera_query,
                            &window_query,
                        );
                        CAMERA_CENTERED = true;
                    }
                }
            }

            // If we still couldn't find the glyph, show the not found message
            if !glyph_found && !codepoint_string.is_empty() {
                // Draw "Codepoint not found" message at (0,0)
                draw_codepoint_not_found_message(
                    &mut gizmos,
                    &viewport,
                    &codepoint_string,
                );
            } else if !glyph_found {
                println!("WARNING: Could not find any common test glyphs");
            }
        }
        None => {
            warn!("No default layer found in the UFO font");
        }
    }
}

/// Draw a message indicating that the codepoint was not found in the UFO source
fn draw_codepoint_not_found_message(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    codepoint: &str,
) {
    // Convert design space coordinates (0,0) to screen space
    let origin = viewport.to_screen(DPoint::from((0.0, 0.0)));

    // Draw a cross at the origin to mark the position
    let cross_size = 10.0;
    gizmos.line_2d(
        Vec2::new(origin.x - cross_size, origin.y - cross_size),
        Vec2::new(origin.x + cross_size, origin.y + cross_size),
        Color::srgb(1.0, 0.0, 0.0),
    );
    gizmos.line_2d(
        Vec2::new(origin.x - cross_size, origin.y + cross_size),
        Vec2::new(origin.x + cross_size, origin.y - cross_size),
        Color::srgb(1.0, 0.0, 0.0),
    );

    // Draw a circle at the origin
    gizmos.circle_2d(origin, 15.0, Color::srgba(1.0, 0.0, 0.0, 0.3));

    // We can't directly render text with gizmos, but we'll print to the console
    println!("Codepoint U+{} not found in UFO source", codepoint);
}

/// Draw points from a glyph
fn draw_glyph_points(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &norad::Glyph,
    selected_points: &[(
        Entity,
        &crate::editing::selection::components::GlyphPointReference,
    )],
) {
    // Only proceed if the glyph has an outline
    if let Some(outline) = &glyph.outline {
        // Iterate through all contours
        for (contour_idx, contour) in outline.contours.iter().enumerate() {
            if contour.points.is_empty() {
                continue;
            }

            // First, draw the actual path with proper cubic curves
            draw_contour_path(gizmos, viewport, contour);

            // Then draw the control handles for off-curve points
            draw_control_handles(gizmos, viewport, contour);

            // Finally, draw the points themselves, but only if they're not selected
            for (point_idx, point) in contour.points.iter().enumerate() {
                // Check if this point is currently selected
                let is_selected =
                    selected_points.iter().any(|(_, ref_point)| {
                        ref_point.glyph_name == glyph.name.to_string()
                            && ref_point.contour_index == contour_idx
                            && ref_point.point_index == point_idx
                    });

                // Skip drawing this point if it's selected (the selection indicator will be shown instead)
                if is_selected {
                    continue;
                }

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
                    let half_size = size / ON_CURVE_SQUARE_ADJUSTMENT; // Adjusting size for visual balance

                    // First draw a filled circle inside the square
                    gizmos.circle_2d(
                        screen_pos,
                        half_size * ON_CURVE_INNER_CIRCLE_RATIO,
                        color,
                    );

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
                    gizmos.circle_2d(
                        screen_pos,
                        size * OFF_CURVE_INNER_CIRCLE_RATIO,
                        color,
                    );
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
    viewport: &ViewPort,
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
    viewport: &ViewPort,
    contour: &norad::Contour,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Use the color defined in the theme file
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
    viewport: &ViewPort,
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

/// Information about a selected point, used for restoring selection
#[derive(Debug, Clone)]
struct SelectedPointInfo {
    position: (f32, f32),
    glyph_name: Option<String>,
    contour_index: Option<usize>,
    point_index: Option<usize>,
}

/// System that spawns glyph point entities for selection
pub fn spawn_glyph_point_entities(
    mut commands: Commands,
    app_state: Res<AppState>,
    point_entities: Query<
        (
            Entity,
            Option<&crate::editing::selection::components::Selected>,
            &Transform,
            Option<&crate::editing::selection::components::GlyphPointReference>,
        ),
        With<Selectable>,
    >,
    glyph_navigation: Res<GlyphNavigation>,
    mut selection_state: ResMut<
        crate::editing::selection::components::SelectionState,
    >,
) {
    // Get the codepoint string
    let codepoint_string = glyph_navigation.get_codepoint_string();
    let glyph_name = norad::GlyphName::from(&*codepoint_string);

    // If no font is loaded, return early
    if app_state.workspace.font.path.is_none() {
        return;
    }

    // Check if we need to respawn entities
    let needs_respawn = point_entities.is_empty()
        || app_state
            .workspace
            .font
            .ufo
            .get_default_layer()
            .and_then(|layer| layer.get_glyph(&glyph_name))
            .is_none();

    if !needs_respawn {
        return;
    }

    // Store selection state before despawning
    let mut selected_points = Vec::new();
    for (entity, selected, transform, point_ref) in point_entities.iter() {
        if selected.is_some() {
            // Remember position and detailed reference information
            let position = (transform.translation.x, transform.translation.y);
            let (glyph_name, contour_index, point_index) =
                if let Some(ref_info) = point_ref {
                    (
                        Some(ref_info.glyph_name.clone()),
                        Some(ref_info.contour_index),
                        Some(ref_info.point_index),
                    )
                } else {
                    (None, None, None)
                };

            selected_points.push(SelectedPointInfo {
                position,
                glyph_name,
                contour_index,
                point_index,
            });

            info!(
                "Stored selection state for point at {:?} with ref {:?}",
                position,
                point_ref.map(|r| format!(
                    "{}/{}/{}",
                    r.glyph_name, r.contour_index, r.point_index
                ))
            );
        }
        commands.entity(entity).despawn();
    }

    // If a specific codepoint was requested and not found, don't spawn any entities
    if !codepoint_string.is_empty() && !glyph_navigation.codepoint_found {
        info!(
            "Not spawning point entities for missing codepoint: U+{}",
            codepoint_string
        );
        return;
    }

    // Get the Ufo reference
    let ufo = &app_state.workspace.font.ufo;

    match ufo.get_default_layer() {
        Some(default_layer) => {
            let mut found_glyph = None;

            // Try to find the glyph using the new helper method
            if let Some(glyph_name) = glyph_navigation.find_glyph(ufo) {
                if let Some(g) = default_layer.get_glyph(&glyph_name) {
                    found_glyph = Some(g);
                }
            }

            // If still not found, try common glyphs
            if found_glyph.is_none() && codepoint_string.is_empty() {
                // Try with some common glyphs
                let common_glyphs =
                    ["H", "h", "A", "a", "O", "o", "space", ".notdef"];

                for glyph_name_str in common_glyphs.iter() {
                    let name = norad::GlyphName::from(*glyph_name_str);
                    if let Some(g) = default_layer.get_glyph(&name) {
                        found_glyph = Some(g);
                        break;
                    }
                }
            }

            match found_glyph {
                Some(glyph) => {
                    // Spawn point entities for the glyph
                    info!(
                        "Spawned selectable point entities for glyph {}",
                        glyph.name
                    );
                    spawn_entities_for_glyph(
                        &mut commands,
                        &glyph,
                        &selected_points,
                        &mut selection_state,
                    );
                }
                None => {
                    info!("No glyph found for selection system");
                }
            }
        }
        None => {
            warn!("No default layer found in the UFO file");
        }
    }
}

/// Spawn selectable entities for a glyph
fn spawn_entities_for_glyph(
    commands: &mut Commands,
    glyph: &norad::Glyph,
    selected_points: &[SelectedPointInfo],
    selection_state: &mut crate::editing::selection::components::SelectionState,
) {
    // Only proceed if the glyph has an outline
    if let Some(outline) = &glyph.outline {
        // Iterate through all contours
        for (contour_idx, contour) in outline.contours.iter().enumerate() {
            if contour.points.is_empty() {
                continue;
            }

            // Spawn entities for each point
            for (point_idx, point) in contour.points.iter().enumerate() {
                let point_pos = (point.x as f32, point.y as f32);

                // Determine if point is on-curve or off-curve
                let is_on_curve = match point.typ {
                    norad::PointType::Move
                    | norad::PointType::Line
                    | norad::PointType::Curve => true,
                    _ => false,
                };

                // Use a unique name for the point entity based on its position
                let entity_name =
                    format!("Point_{}_{}", point_pos.0, point_pos.1);

                // Check if this point was previously selected
                let was_selected = selected_points.iter().any(|info| {
                    // First try to match by exact reference
                    (info.glyph_name.as_ref().map(|name| name == &glyph.name.to_string()).unwrap_or(false) &&
                     info.contour_index.map(|idx| idx == contour_idx).unwrap_or(false) &&
                     info.point_index.map(|idx| idx == point_idx).unwrap_or(false))
                    ||
                    // Fallback to position matching with tolerance
                    ((info.position.0 - point_pos.0).abs() < 0.001 &&
                     (info.position.1 - point_pos.1).abs() < 0.001)
                });

                // Determine if we should add the Selected component
                let mut entity_cmds = commands.spawn((
                    Transform::from_translation(Vec3::new(
                        point_pos.0,
                        point_pos.1,
                        0.0,
                    )),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Selectable,
                    crate::editing::selection::components::PointType { is_on_curve },
                    crate::editing::selection::nudge::PointCoordinates {
                        position: Vec2::new(point_pos.0, point_pos.1),
                    },
                    crate::editing::selection::components::GlyphPointReference {
                        glyph_name: glyph.name.to_string(),
                        contour_index: contour_idx,
                        point_index: point_idx,
                    },
                    Name::new(entity_name),
                ));

                // If the point was selected before, restore selection state
                if was_selected {
                    let entity = entity_cmds.id();
                    entity_cmds.insert(
                        crate::editing::selection::components::Selected,
                    );
                    selection_state.selected.insert(entity);
                    info!(
                        "Restored selection for point at ({}, {})",
                        point_pos.0, point_pos.1
                    );
                }
            }
        }
    }
}

/// System to detect when AppState changes and send an event
pub fn detect_app_state_changes(
    app_state: Res<AppState>,
    mut event_writer: EventWriter<AppStateChanged>,
) {
    if app_state.is_changed() {
        event_writer.send(AppStateChanged);
    }
}
