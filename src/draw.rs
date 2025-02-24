//! Drawing algorithms and helpers

use bevy::prelude::*;
use crate::design_space::{ViewPort, DPoint};
use crate::data::{FontMetrics, AppState};
use norad::Glyph;

/// System that draws basic test elements for development
pub fn draw_test_elements(mut gizmos: Gizmos) {
    // Draw a simple test cross at the origin
    gizmos.line_2d(
        Vec2::new(-50.0, 0.0),
        Vec2::new(50.0, 0.0),
        Color::srgb(1.0, 0.0, 0.0)
    );
    gizmos.line_2d(
        Vec2::new(0.0, -50.0),
        Vec2::new(0.0, 50.0),
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
) {
    // Add debug message at the start so we know this system is being called
    info!("DRAW SYSTEM: draw_glyph_points_system is running...");
    
    // Get the primary viewport or create a default one if none exists
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };
    
    // Print font info for debugging
    println!("Font family: {}", app_state.workspace.info.family_name);
    println!("Font style: {}", app_state.workspace.info.style_name);
    
    // Check if we can get the default layer
    println!("Checking for default layer in the font...");
    match app_state.workspace.font.ufo.get_default_layer() {
        Some(default_layer) => {
            println!("Found default layer, trying to access glyph 'H'");
            
            // Try to get the 'H' glyph directly by name
            let glyph_name = norad::GlyphName::from("H");
            
            match default_layer.get_glyph(&glyph_name) {
                Some(glyph) => {
                    println!("SUCCESS: Found glyph 'H', drawing points...");
                    
                    // Draw the points
                    draw_glyph_points(&mut gizmos, viewport, glyph);
                }
                None => {
                    println!("Glyph 'H' not found by name, trying 'h'");
                    
                    // Try with lowercase 'h'
                    let lowercase_name = norad::GlyphName::from("h");
                    match default_layer.get_glyph(&lowercase_name) {
                        Some(glyph) => {
                            println!("Found glyph 'h' instead, drawing points...");
                            draw_glyph_points(&mut gizmos, viewport, glyph);
                        }
                        None => {
                            println!("Couldn't find 'h' either, trying common glyph names...");
                            
                            // Try with some other common glyphs
                            let common_glyphs = ["A", "a", "O", "o", "space", ".notdef"];
                            let mut found = false;
                            
                            for glyph_name_str in common_glyphs.iter() {
                                let name = norad::GlyphName::from(*glyph_name_str);
                                if let Some(glyph) = default_layer.get_glyph(&name) {
                                    println!("Found glyph '{}', drawing points...", glyph_name_str);
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
    println!("Drawing glyph '{}' at scale {:.2}", glyph.name, viewport.zoom);
    
    // Only proceed if the glyph has an outline
    if let Some(outline) = &glyph.outline {
        // Log information about the glyph for debugging
        println!("Glyph '{}' has {} contours", glyph.name, outline.contours.len());
        
        // Color for drawing points
        let point_color = Color::srgba(0.0, 0.5, 1.0, 1.0);
        let control_point_color = Color::srgba(1.0, 0.3, 0.3, 0.8); // Red for control points
        
        // Iterate through all contours
        for (contour_idx, contour) in outline.contours.iter().enumerate() {
            println!("Contour {} has {} points", contour_idx, contour.points.len());
            
            // Draw each point in the contour
            for (point_idx, point) in contour.points.iter().enumerate() {
                let point_pos = (point.x as f32, point.y as f32);
                let screen_pos = viewport.to_screen(DPoint::from(point_pos));
                
                // Use different sizes and colors based on point type
                let (size, color) = match point.typ {
                    norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve => 
                        (6.0, point_color),
                    norad::PointType::OffCurve => 
                        (4.0, control_point_color),
                    _ => (4.0, point_color)
                };
                
                // Draw a circle for each point
                gizmos.circle_2d(screen_pos, size, color);
                
                // Draw a line connecting on-curve and off-curve points
                if point_idx > 0 {
                    let prev_point = &contour.points[point_idx - 1];
                    let prev_pos = (prev_point.x as f32, prev_point.y as f32);
                    let prev_screen_pos = viewport.to_screen(DPoint::from(prev_pos));
                    
                    // Draw line between points
                    let line_color = Color::srgba(0.5, 0.5, 0.5, 0.7);
                    gizmos.line_2d(prev_screen_pos, screen_pos, line_color);
                }
                
                // Useful debug info
                println!("Point {}: ({}, {}), type: {:?}", 
                       point_idx, point.x, point.y, point.typ);
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
                let line_color = Color::srgba(0.5, 0.5, 0.5, 0.7);
                gizmos.line_2d(last_screen_pos, first_screen_pos, line_color);
            }
        }
    } else {
        println!("Glyph '{}' has no outline", glyph.name);
    }
}
