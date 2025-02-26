//! Drawing algorithms and helpers

use bevy::prelude::*;
use crate::design_space::{ViewPort, DPoint};
use crate::data::{FontMetrics, AppState};
use crate::theme::{ON_CURVE_POINT_RADIUS, OFF_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR, 
                  OFF_CURVE_POINT_COLOR, PATH_LINE_COLOR, USE_SQUARE_FOR_ON_CURVE};
use norad::Glyph;

/// System that draws basic test elements for development
pub fn draw_test_elements(mut gizmos: Gizmos) {
    // Draw a simple test cross at the origin
    gizmos.line_2d(
        Vec2::new(-64.0, 0.0),
        Vec2::new(64.0, 0.0),
        Color::srgb(1.0, 0.0, 0.0)
    );
    gizmos.line_2d(
        Vec2::new(0.0, -64.0),
        Vec2::new(0.0, 64.0),
        Color::srgb(1.0, 0.0, 0.0)
    );
}

/// System that draws font metrics
pub fn draw_metrics_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
) {
    // Get the primary viewport
    if let Ok(viewport) = viewports.get_single() {
        // We need a glyph to draw metrics for
        if app_state.workspace.font.ufo.font_info.is_some() {
            // Use a placeholder glyph with standard advance width
            let mut placeholder = Glyph::new_named("placeholder");
            placeholder.advance = Some(norad::Advance { 
                width: 1000.0, 
                height: 0.0 
            });
            
            // Draw the metrics directly without using DrawCtx
            draw_metrics(&mut gizmos, *viewport, &placeholder, &app_state.workspace.info.metrics);
        }
    }
}

/// Draw font metrics lines (baseline, x-height, cap-height)
fn draw_metrics(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics
) {
    let upm = metrics.units_per_em;
    let x_height = metrics.x_height.unwrap_or_else(|| (upm * 0.5).round());
    let cap_height = metrics.cap_height.unwrap_or_else(|| (upm * 0.7).round());
    let width = glyph.advance
        .as_ref()
        .map(|a| a.width as f64)
        .unwrap_or_else(|| (upm * 0.5).round());
    
    // Metrics color - light gray
    let metrics_color = Color::srgba(0.7, 0.7, 0.7, 0.8);
    
    // Draw baseline
    draw_line(gizmos, viewport, 
        (0.0, 0.0),
        (width as f32, 0.0),
        metrics_color
    );
    
    // Draw x-height line
    draw_line(gizmos, viewport,
        (0.0, x_height as f32),
        (width as f32, x_height as f32),
        metrics_color
    );
    
    // Draw cap-height line
    draw_line(gizmos, viewport,
        (0.0, cap_height as f32),
        (width as f32, cap_height as f32),
        metrics_color
    );
}

/// Draw a line in design space
fn draw_line(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color
) {
    let start_screen = viewport.to_screen(DPoint::from(start));
    let end_screen = viewport.to_screen(DPoint::from(end));
    gizmos.line_2d(start_screen, end_screen, color);
}

/// Plugin to add drawing systems
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            draw_test_elements,
            draw_metrics_system,
            draw_glyph_points_system,
        ));
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
                    let common_glyphs = ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
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
                        println!("WARNING: Could not find any common test glyphs");
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
            // Draw each point in the contour
            for (point_idx, point) in contour.points.iter().enumerate() {
                let point_pos = (point.x as f32, point.y as f32);
                let screen_pos = viewport.to_screen(DPoint::from(point_pos));
                
                // Determine if point is on-curve or off-curve
                let is_on_curve = match point.typ {
                    norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve => true,
                    _ => false
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
                    let top_left = Vec2::new(screen_pos.x - half_size, screen_pos.y + half_size);
                    let top_right = Vec2::new(screen_pos.x + half_size, screen_pos.y + half_size);
                    let bottom_right = Vec2::new(screen_pos.x + half_size, screen_pos.y - half_size);
                    let bottom_left = Vec2::new(screen_pos.x - half_size, screen_pos.y - half_size);
                    
                    // Draw the square sides
                    gizmos.line_2d(top_left, top_right, color);
                    gizmos.line_2d(top_right, bottom_right, color);
                    gizmos.line_2d(bottom_right, bottom_left, color);
                    gizmos.line_2d(bottom_left, top_left, color);
                } else {
                    // For off-curve points or if squares are disabled, draw a filled circle
                    gizmos.circle_2d(screen_pos, size, color);
                }
                
                // Draw a line connecting on-curve and off-curve points
                if point_idx > 0 {
                    let prev_point = &contour.points[point_idx - 1];
                    let prev_pos = (prev_point.x as f32, prev_point.y as f32);
                    let prev_screen_pos = viewport.to_screen(DPoint::from(prev_pos));
                    
                    // Draw line between points
                    gizmos.line_2d(prev_screen_pos, screen_pos, PATH_LINE_COLOR);
                }
            }
            
            // Connect the last point to the first to close the contour
            if !contour.points.is_empty() {
                let first_point = &contour.points[0];
                let last_point = &contour.points[contour.points.len() - 1];
                
                let first_pos = (first_point.x as f32, first_point.y as f32);
                let last_pos = (last_point.x as f32, last_point.y as f32);
                
                let first_screen_pos = viewport.to_screen(DPoint::from(first_pos));
                let last_screen_pos = viewport.to_screen(DPoint::from(last_pos));
                
                // Draw closing line
                gizmos.line_2d(last_screen_pos, first_screen_pos, PATH_LINE_COLOR);
            }
        }
    } else {
        println!("Glyph '{}' has no outline", glyph.name);
    }
}
