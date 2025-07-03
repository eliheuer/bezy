//! Shared sort rendering for both text buffer and freeform/entity sorts

use bevy::prelude::*;
use crate::core::state::font_metrics::FontMetrics;
use crate::rendering::glyph_outline::draw_glyph_outline_at_position;
use crate::rendering::metrics::draw_metrics_at_position;
use kurbo::BezPath;
use crate::core::state::font_data::OutlineData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortRenderStyle {
    TextBuffer,
    Freeform,
}

/// Draws a sort (outline, metrics, handles) at the given position with the given style
pub fn render_sort_visuals(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    style: SortRenderStyle,
) {
    let orange = Color::srgb(1.0, 0.5, 0.0);
    let cyan = Color::srgb(0.0, 1.0, 1.0);
    let (outline_color, metrics_color, handle_color) = match style {
        SortRenderStyle::TextBuffer => (
            orange,
            orange.with_alpha(0.8),
            orange,
        ),
        SortRenderStyle::Freeform => (
            cyan,
            cyan.with_alpha(0.8),
            cyan,
        ),
    };

    // Draw outline
    draw_glyph_outline_at_position(gizmos, outline, position);
    // Draw metrics
    draw_metrics_at_position(gizmos, advance_width, metrics, position, metrics_color);
    // Draw handles (for now, just a circle at the position)
    gizmos.circle_2d(position, 12.0, handle_color);
    // TODO: Use different handle shapes for each style if desired
} 