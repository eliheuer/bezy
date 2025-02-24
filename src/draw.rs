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
    // Get the primary viewport
    if let Ok(viewport) = viewports.get_single() {
        // Only proceed if we have a font loaded
        if app_state.workspace.font.ufo.font_info.is_some() {
            // Get the 'H' glyph (Unicode codepoint U+0048)
            let codepoint = 0x0048u32; // 'H' character
            
            // Try to find the glyph with this codepoint
            if let Some(glyph) = find_glyph_by_codepoint(&app_state.workspace.font.ufo, codepoint) {
                // Draw the points for this glyph
                draw_glyph_points(&mut gizmos, *viewport, &glyph);
            } else {
                info!("Glyph for codepoint U+{:04X} not found", codepoint);
            }
        }
    }
}

/// Find a glyph by Unicode codepoint
fn find_glyph_by_codepoint(ufo: &norad::Ufo, codepoint: u32) -> Option<norad::Glyph> {
    // Iterate through all glyphs and find one with the matching codepoint
    for (glyph_name, glyph) in ufo.iter_glyphs() {
        if let Some(codepoints) = &glyph.codepoints {
            if codepoints.contains(&codepoint) {
                info!("Found glyph '{}' for codepoint U+{:04X}", glyph_name, codepoint);
                return Some(glyph.clone());
            }
        }
    }
    None
}

/// Draw points from a glyph
fn draw_glyph_points(
    gizmos: &mut Gizmos,
    viewport: ViewPort,
    glyph: &norad::Glyph,
) {
    // Only proceed if the glyph has an outline
    if let Some(outline) = &glyph.outline {
        // Log information about the glyph for debugging
        info!("Drawing points for glyph '{}' with {} contours", 
              glyph.name, outline.contours.len());
        
        // Color for drawing points
        let point_color = Color::srgba(0.0, 0.5, 1.0, 1.0);
        
        // Iterate through all contours
        for (contour_idx, contour) in outline.contours.iter().enumerate() {
            info!("Contour {} has {} points", contour_idx, contour.points.len());
            
            // Draw each point in the contour
            for (point_idx, point) in contour.points.iter().enumerate() {
                let point_pos = (point.x as f32, point.y as f32);
                let screen_pos = viewport.to_screen(DPoint::from(point_pos));
                
                // Draw a circle for each point
                gizmos.circle_2d(screen_pos, 4.0, point_color);
                
                // Useful debug info
                info!("Point {}: ({}, {}), type: {:?}", 
                      point_idx, point.x, point.y, point.typ);
            }
        }
    } else {
        info!("Glyph '{}' has no outline", glyph.name);
    }
}
