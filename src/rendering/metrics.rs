//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use bevy::prelude::*;
use crate::core::state::FontMetrics;
use crate::ui::theme::METRICS_GUIDE_COLOR;
use crate::ui::panes::design_space::{DPoint, ViewPort};
use norad::Glyph;

/// Draw complete font metrics for a glyph at a specific position
pub fn draw_metrics_at_position(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
    position: Vec2,
) {
    draw_metrics_at_position_with_color(gizmos, viewport, glyph, metrics, position, METRICS_GUIDE_COLOR);
}

/// Draw complete font metrics for a glyph at a specific position with custom color
pub fn draw_metrics_at_position_with_color(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
    position: Vec2,
    color: Color,
) {
    let upm = metrics.units_per_em as f32;
    let ascender = metrics.ascender.unwrap_or(800.0) as f32;
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    
    // Use actual font metrics if available, otherwise fallback to reasonable defaults
    let x_height = metrics.x_height.unwrap_or((upm * 0.5).round() as f64) as f32;
    let cap_height = metrics.cap_height.unwrap_or((upm * 0.7).round() as f64) as f32;

    let width = glyph.width as f32;

    // All coordinates are offset by the position
    let offset_x = position.x;
    let offset_y = position.y;

    // Draw the standard metrics bounding box (descender to ascender)
    draw_rect(
        gizmos,
        viewport,
        (offset_x, offset_y + descender),
        (offset_x + width, offset_y + ascender),
        color,
    );

    // Draw the full UPM bounding box (from 0 to UPM height)
    draw_rect(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width, offset_y + upm),
        color,
    );

    // Draw baseline (most important)
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width, offset_y),
        color,
    );

    // Draw x-height line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + x_height),
        (offset_x + width, offset_y + x_height),
        color,
    );

    // Draw cap-height line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + cap_height),
        (offset_x + width, offset_y + cap_height),
        color,
    );

    // Draw ascender line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + ascender),
        (offset_x + width, offset_y + ascender),
        color,
    );

    // Draw descender line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + descender),
        (offset_x + width, offset_y + descender),
        color,
    );

    // Draw UPM top line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + upm),
        (offset_x + width, offset_y + upm),
        color,
    );
}

/// Draw a line in design space
fn draw_line(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    let start_screen = viewport.to_screen(DPoint::from(start));
    let end_screen = viewport.to_screen(DPoint::from(end));
    gizmos.line_2d(start_screen, end_screen, color);
}

/// Draw a rectangle outline in design space
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
