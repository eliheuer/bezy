//! Drawing algorithms and helpers

#![allow(deprecated)]

use crate::core::state::{AppState, FontMetrics, GlyphNavigation};
use crate::ui::panes::design_space::ViewPort;
use crate::ui::theme::DEBUG_SHOW_ORIGIN_CROSS;
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

/// System to draw metric lines (baseline, x-height, etc.) on the design canvas
#[allow(dead_code)]
pub fn draw_metrics_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    glyph_navigation: Res<GlyphNavigation>,
) {
    // Early exit if no font is loaded
    if app_state.workspace.font.glyphs.is_empty() {
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

    // Find a glyph to use for metrics display
    let glyph = find_glyph_for_metrics(&glyph_navigation, &app_state);
    
    // Draw the metrics using the found or placeholder glyph
    draw_metrics(
        &mut gizmos,
        &viewport,
        &glyph,
        &app_state.workspace.info.metrics,
    );
}

/// Finds the best glyph to use for determining the width of metric lines
#[allow(dead_code)]
fn find_glyph_for_metrics(
    glyph_navigation: &GlyphNavigation,
    app_state: &AppState,
) -> Glyph {
    // Try to get the specifically requested glyph first.
    // If found, convert it to norad format and return.
    if let Some(glyph_name) = glyph_navigation.find_glyph(app_state) {
        if let Some(glyph_data) = app_state.workspace.font.get_glyph(&glyph_name) {
            return glyph_data.to_norad_glyph();
        }
    }

    // If no specific glyph is found, fall back to creating a standard placeholder glyph.
    create_placeholder_glyph(&app_state.workspace.info.metrics)
}

/// Creates a placeholder glyph when no suitable glyph is found for metrics
#[allow(dead_code)]
fn create_placeholder_glyph(
    metrics: &crate::core::state::FontMetrics
) -> Glyph {
    let mut placeholder = Glyph::new("placeholder");
    // Note: In norad 0.16.0, advance is set directly as width/height fields
    placeholder.width = metrics.units_per_em as f64;
    placeholder.height = 0.0;
    placeholder
}

/// Draws the actual metric lines using the glyph and viewport
#[allow(dead_code)]
fn draw_metrics(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
) {
    // Use the shared metrics rendering function
    crate::rendering::metrics::draw_metrics_at_position(gizmos, viewport, glyph, metrics, Vec2::ZERO);
}

/// Event for signaling app state changes that might affect rendering
#[derive(Event)]
#[allow(dead_code)]
pub struct AppStateChanged;

/// System to detect when app state changes
#[allow(dead_code)]
pub fn detect_app_state_changes(
    app_state: Res<AppState>,
    mut event_writer: EventWriter<AppStateChanged>,
) {
    if app_state.is_changed() {
        event_writer.write(AppStateChanged);
    }
}
