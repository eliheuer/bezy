//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use bevy::prelude::*;
use crate::core::state::FontMetrics;
use crate::ui::theme::METRICS_GUIDE_COLOR;
use norad::Glyph;

/// Draw complete font metrics for a glyph at a specific position
pub fn draw_metrics_at_position(
    gizmos: &mut Gizmos,
    glyph: &Glyph,
    metrics: &FontMetrics,
    position: Vec2,
) {
    draw_metrics_at_position_with_color(gizmos, glyph, metrics, position, METRICS_GUIDE_COLOR);
}

/// Draw complete font metrics for a glyph at a specific position with custom color
pub fn draw_metrics_at_position_with_color(
    gizmos: &mut Gizmos,
    glyph: &Glyph,
    metrics: &FontMetrics,
    position: Vec2,
    color: Color,
) {
    let upm = metrics.units_per_em as f32;
    let ascender = metrics.ascender.unwrap_or((upm * 0.8).round() as f64) as f32;
    let descender = metrics.descender.unwrap_or(-(upm * 0.2).round() as f64) as f32;
    
    // Calculate x-height and cap-height based on UPM
    let x_height = (upm * 0.5).round();
    let cap_height = (upm * 0.7).round();

    let width = glyph.width as f32;

    // All coordinates are offset by the position
    let offset_x = position.x;
    let offset_y = position.y;

    // Draw baseline
    draw_line(
        gizmos,
        (offset_x, offset_y),
        (offset_x + width, offset_y),
        color,
    );

    // Draw x-height line
    draw_line(
        gizmos,
        (offset_x, offset_y + x_height),
        (offset_x + width, offset_y + x_height),
        color,
    );

    // Draw cap-height line
    draw_line(
        gizmos,
        (offset_x, offset_y + cap_height),
        (offset_x + width, offset_y + cap_height),
        color,
    );

    // Draw ascender line
    draw_line(
        gizmos,
        (offset_x, offset_y + ascender),
        (offset_x + width, offset_y + ascender),
        color,
    );

    // Draw descender line
    draw_line(
        gizmos,
        (offset_x, offset_y + descender),
        (offset_x + width, offset_y + descender),
        color,
    );

    // Draw UPM top line
    draw_line(
        gizmos,
        (offset_x, offset_y + upm),
        (offset_x + width, offset_y + upm),
        color,
    );

    // Draw vertical side-bearing lines
    draw_line(
        gizmos,
        (offset_x, offset_y + descender),
        (offset_x, offset_y + ascender),
        color,
    );
    draw_line(
        gizmos,
        (offset_x + width, offset_y + descender),
        (offset_x + width, offset_y + ascender),
        color,
    );
}

/// Draw a line in world space
fn draw_line(
    gizmos: &mut Gizmos,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    gizmos.line_2d(Vec2::new(start.0, start.1), Vec2::new(end.0, end.1), color);
}
