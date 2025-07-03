//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use bevy::prelude::*;
use crate::core::state::font_metrics::FontMetrics;
use crate::ui::theme::METRICS_GUIDE_COLOR;
use norad::Glyph;

/// Draw complete font metrics for a glyph at a specific design-space position
pub fn draw_metrics_at_position(
    gizmos: &mut Gizmos,
    glyph: &norad::Glyph,
    metrics: &FontMetrics,
    position: Vec2,
    color: Color,
) {
    let upm = metrics.units_per_em;
    let ascender = metrics.ascender.unwrap_or(upm * 0.8) as f32;
    let descender = metrics.descender.unwrap_or(upm * -0.2) as f32;
    let x_height = metrics.x_height.unwrap_or(upm * 0.5) as f32;
    let cap_height = metrics.cap_height.unwrap_or(upm * 0.7) as f32;
    let advance_width = glyph.width as f32;

    // Baseline (most important)
    gizmos.line_2d(
        position,
        Vec2::new(position.x + advance_width, position.y),
        color,
    );

    // x-height
    let x_height_y = position.y + x_height;
    gizmos.line_2d(
        Vec2::new(position.x, x_height_y),
        Vec2::new(position.x + advance_width, x_height_y),
        color,
    );

    // cap-height
    let cap_height_y = position.y + cap_height;
    gizmos.line_2d(
        Vec2::new(position.x, cap_height_y),
        Vec2::new(position.x + advance_width, cap_height_y),
        color,
    );

    // ascender
    let ascender_y = position.y + ascender;
    gizmos.line_2d(
        Vec2::new(position.x, ascender_y),
        Vec2::new(position.x + advance_width, ascender_y),
        color,
    );

    // descender
    let descender_y = position.y + descender;
    gizmos.line_2d(
        Vec2::new(position.x, descender_y),
        Vec2::new(position.x + advance_width, descender_y),
        color,
    );
}

/// Draw a line in design space
fn draw_line(gizmos: &mut Gizmos, start: (f32, f32), end: (f32, f32), color: Color) {
    gizmos.line_2d(start.into(), end.into(), color);
}

/// Draw a rectangle outline in design space
fn draw_rect(gizmos: &mut Gizmos, top_left: Vec2, bottom_right: (f32, f32), color: Color) {
    let br: Vec2 = bottom_right.into();
    gizmos.line_2d(top_left, Vec2::new(br.x, top_left.y), color);
    gizmos.line_2d(Vec2::new(br.x, top_left.y), br, color);
    gizmos.line_2d(br, Vec2::new(top_left.x, br.y), color);
    gizmos.line_2d(Vec2::new(top_left.x, br.y), top_left, color);
}
