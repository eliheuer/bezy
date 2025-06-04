//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use crate::core::state::FontMetrics;
use crate::ui::panes::design_space::{DPoint, ViewPort};
use crate::ui::theme::METRICS_GUIDE_COLOR;
use bevy::prelude::*;
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
    metrics_color: Color,
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

    // All coordinates are offset by the position
    let offset_x = position.x;
    let offset_y = position.y;

    // Draw the standard metrics bounding box (descender to ascender)
    draw_rect(
        gizmos,
        viewport,
        (offset_x, offset_y + descender as f32),
        (offset_x + width as f32, offset_y + ascender as f32),
        metrics_color,
    );

    // Draw the full UPM bounding box (from 0 to UPM height)
    draw_rect(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y + upm as f32),
        metrics_color,
    );

    // Draw baseline
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y),
        metrics_color,
    );

    // Draw x-height line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + x_height as f32),
        (offset_x + width as f32, offset_y + x_height as f32),
        metrics_color,
    );

    // Draw cap-height line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + cap_height as f32),
        (offset_x + width as f32, offset_y + cap_height as f32),
        metrics_color,
    );

    // Draw ascender line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + ascender as f32),
        (offset_x + width as f32, offset_y + ascender as f32),
        metrics_color,
    );

    // Draw descender line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + descender as f32),
        (offset_x + width as f32, offset_y + descender as f32),
        metrics_color,
    );

    // Draw UPM top line
    draw_line(
        gizmos,
        viewport,
        (offset_x, offset_y + upm as f32),
        (offset_x + width as f32, offset_y + upm as f32),
        metrics_color,
    );
}

/// Draw font metrics at the origin (for the main metrics system)
pub fn draw_metrics_at_origin(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &Glyph,
    metrics: &FontMetrics,
) {
    draw_metrics_at_position(gizmos, viewport, glyph, metrics, Vec2::ZERO);
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