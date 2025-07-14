//! Shared sort rendering for both text buffer and freeform/entity sorts

use crate::core::state::font_data::OutlineData;
use crate::core::state::font_metrics::FontMetrics;
use crate::editing::selection::components::GlyphPointReference;
use crate::editing::selection::nudge::NudgeState;
use crate::rendering::glyph_outline::{
    draw_glyph_outline_at_position, draw_glyph_outline_from_live_transforms,
};
use crate::rendering::metrics::draw_metrics_at_position;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;
use kurbo::BezPath;

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
) {
    // Draw outline
    draw_glyph_outline_at_position(gizmos, outline, position);
    // Draw metrics
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_color,
    );

    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);

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

/// Enhanced sort rendering that uses live Transform positions during nudging
/// This ensures perfect synchronization between points and outline
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn render_sort_visuals_with_live_sync(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
    // Live rendering parameters
    sort_entity: Option<Entity>,
    sort_transform: Option<&Transform>,
    glyph_name: Option<&str>,
    #[allow(clippy::type_complexity)] point_query: Option<
        &Query<
            (
                Entity,
                &Transform,
                &GlyphPointReference,
                &crate::editing::selection::components::PointType,
            ),
            With<SortPointEntity>,
        >,
    >,
    selected_query: Option<
        &Query<Entity, With<crate::editing::selection::components::Selected>>,
    >,
    app_state: Option<&crate::core::state::AppState>,
    nudge_state: Option<&NudgeState>,
) {
    // Determine if we should use live rendering
    let nudge_active = nudge_state.is_some_and(|ns| ns.is_nudging);
    let has_sort_entity = sort_entity.is_some();
    let has_sort_transform = sort_transform.is_some();
    let has_glyph_name = glyph_name.is_some();
    let has_point_query = point_query.is_some();
    let has_app_state = app_state.is_some();

    let has_selected_query = selected_query.is_some();

    let use_live_rendering = nudge_active
        && has_sort_entity
        && has_sort_transform
        && has_glyph_name
        && has_point_query
        && has_selected_query
        && has_app_state;

    // Debug logging
    debug!("[LIVE RENDER CHECK] nudge_active={}, has_sort_entity={}, has_sort_transform={}, has_glyph_name={}, has_point_query={}, has_selected_query={}, has_app_state={}, use_live_rendering={}", 
           nudge_active, has_sort_entity, has_sort_transform, has_glyph_name, has_point_query, has_selected_query, has_app_state, use_live_rendering);

    // Draw outline with appropriate method
    if use_live_rendering {
        debug!("[LIVE RENDER] *** USING LIVE TRANSFORM POSITIONS FOR OUTLINE RENDERING ***");
        draw_glyph_outline_from_live_transforms(
            gizmos,
            sort_entity.unwrap(),
            sort_transform.unwrap(),
            glyph_name.unwrap(),
            point_query.unwrap(),
            app_state.unwrap(),
            selected_query.unwrap(),
        );
    } else {
        // Use normal rendering from glyph data
        debug!("[NORMAL RENDER] Using glyph data for outline rendering");
        draw_glyph_outline_at_position(gizmos, outline, position);
    }

    // Draw metrics (always from original data)
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_color,
    );

    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);

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
