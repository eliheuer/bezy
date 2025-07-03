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
        let selected_size = 32.0; // Bigger than normal
        
        match style {
            SortRenderStyle::TextBuffer => {
                // Square handle for text sorts
                let square_size = Vec2::new(selected_size * 2.0, selected_size * 2.0);
                gizmos.rect_2d(handle_position, square_size, selected_color);
                
                // Add a smaller inner square for better visibility
                let inner_square_size = Vec2::new(selected_size, selected_size);
                gizmos.rect_2d(handle_position, inner_square_size, selected_color);
            }
            SortRenderStyle::Freeform => {
                // Circle handle for freeform sorts
                gizmos.circle_2d(handle_position, selected_size, selected_color);
                
                // Add a small inner circle for better visibility
                gizmos.circle_2d(handle_position, selected_size * 0.5, selected_color);
            }
        }
    } else {
        // Normal handle: smaller and uses metrics color
        let normal_size = 16.0;
        
        match style {
            SortRenderStyle::TextBuffer => {
                // Square handle for text sorts
                let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
                gizmos.rect_2d(handle_position, square_size, metrics_color);
            }
            SortRenderStyle::Freeform => {
                // Circle handle for freeform sorts
                gizmos.circle_2d(handle_position, normal_size, metrics_color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_render_style_distinction() {
        // Test that TextBuffer and Freeform styles are distinct
        assert_ne!(SortRenderStyle::TextBuffer, SortRenderStyle::Freeform);
        
        // Test that each style is equal to itself
        assert_eq!(SortRenderStyle::TextBuffer, SortRenderStyle::TextBuffer);
        assert_eq!(SortRenderStyle::Freeform, SortRenderStyle::Freeform);
    }

    #[test]
    fn test_sort_render_style_debug() {
        // Test that styles can be debug printed
        assert_eq!(format!("{:?}", SortRenderStyle::TextBuffer), "TextBuffer");
        assert_eq!(format!("{:?}", SortRenderStyle::Freeform), "Freeform");
    }
} 