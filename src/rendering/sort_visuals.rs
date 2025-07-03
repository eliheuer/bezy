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
    metrics_color: Color,
    style: SortRenderStyle,
    is_selected: bool,
) {
    // Draw outline
    draw_glyph_outline_at_position(gizmos, outline, position);
    // Draw metrics
    draw_metrics_at_position(gizmos, advance_width, metrics, position, metrics_color);
    
    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);
    
    if is_selected {
        // Selected handle: bigger and yellow
        let selected_color = Color::srgb(1.0, 1.0, 0.0); // Bright yellow
        let selected_radius = 32.0; // Bigger than normal
        gizmos.circle_2d(
            handle_position,
            selected_radius,
            selected_color
        );
        
        // Add a small inner circle for better visibility
        gizmos.circle_2d(
            handle_position,
            selected_radius * 0.5,
            selected_color
        );
    } else {
        // Normal handle: smaller and uses metrics color
        let normal_radius = 16.0;
        gizmos.circle_2d(
            handle_position,
            normal_radius,
            metrics_color
        );
    }
} 