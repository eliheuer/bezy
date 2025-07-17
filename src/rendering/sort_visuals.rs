//! Shared sort rendering for both text buffer and freeform/entity sorts

use crate::core::state::font_data::OutlineData;
use crate::core::state::font_metrics::FontMetrics;
use crate::core::state::FontIRAppState;
use crate::editing::selection::components::GlyphPointReference;
use crate::editing::selection::nudge::NudgeState;
use crate::rendering::glyph_outline::{
    draw_glyph_outline_at_position, draw_glyph_outline_from_live_transforms,
};
use crate::rendering::fontir_glyph_outline::{
    draw_fontir_glyph_outline_at_position, get_fontir_glyph_paths,
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
    render_sort_visuals_with_selection(
        gizmos,
        outline,
        advance_width,
        metrics,
        position,
        metrics_color,
        style,
        false, // not selected
        false, // not active
    );
}

/// Draws a sort with selection and activation state support
#[allow(clippy::too_many_arguments)]
pub fn render_sort_visuals_with_selection(
    gizmos: &mut Gizmos,
    outline: &Option<OutlineData>,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
    is_selected: bool,
    is_active: bool,
) {
    // Determine colors based on state
    let handle_color = if is_selected {
        Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
    } else if is_active {
        Color::srgb(0.0, 1.0, 0.0) // Green for active
    } else {
        metrics_color // Default metrics color
    };

    let metrics_render_color = if is_active {
        Color::srgb(0.0, 1.0, 0.0) // Green metrics for active sorts
    } else {
        metrics_color // Default metrics color
    };

    // Draw outline
    draw_glyph_outline_at_position(gizmos, outline, position);
    // Draw metrics
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_render_color,
    );

    // Draw handle at descender position (matching click detection logic)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);

    // Normal handle: smaller and uses appropriate color
    let normal_size = 16.0;

    match style {
        SortRenderStyle::TextBuffer => {
            // Square handle for text sorts
            let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
            gizmos.rect_2d(handle_position, square_size, handle_color);
        }
        SortRenderStyle::Freeform => {
            // Circle handle for freeform sorts
            gizmos.circle_2d(handle_position, normal_size, handle_color);
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
    // FIXED: Check if we have selected points, not just if nudging is active
    // This ensures outline stays synced with Transform positions even after nudging stops
    let nudge_active = nudge_state.is_some_and(|ns| ns.is_nudging);
    let has_sort_entity = sort_entity.is_some();
    let has_sort_transform = sort_transform.is_some();
    let has_glyph_name = glyph_name.is_some();
    let has_point_query = point_query.is_some();
    let has_app_state = app_state.is_some();
    let has_selected_query = selected_query.is_some();

    // Check if there are any selected points for this sort
    let has_selected_points = if let (
        Some(_sort_entity_val),
        Some(point_query),
        Some(selected_query),
    ) = (sort_entity, point_query, selected_query)
    {
        point_query.iter().any(|(entity, _, _, _)| {
            // Check if this point is selected
            selected_query.get(entity).is_ok()
        })
    } else {
        false
    };

    // Use live rendering during nudging OR when there are selected points
    // This ensures the outline doesn't revert when nudging stops but points are still selected
    let use_live_rendering = (nudge_active || has_selected_points)
        && has_sort_entity
        && has_sort_transform
        && has_glyph_name
        && has_point_query
        && has_selected_query
        && has_app_state;

    // Debug logging
    debug!("[LIVE RENDER CHECK] nudge_active={}, has_selected_points={}, has_sort_entity={}, has_sort_transform={}, has_glyph_name={}, has_point_query={}, has_selected_query={}, has_app_state={}, use_live_rendering={}", 
           nudge_active, has_selected_points, has_sort_entity, has_sort_transform, has_glyph_name, has_point_query, has_selected_query, has_app_state, use_live_rendering);

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

    // Normal handle: smaller and uses appropriate color based on selection state
    let normal_size = 16.0;

    // Determine handle color based on selection state
    let handle_color = if let (Some(entity), Some(selected_query)) =
        (sort_entity, selected_query)
    {
        // Check if this sort is selected
        if selected_query
            .iter()
            .any(|selected_entity| selected_entity == entity)
        {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
        } else {
            metrics_color // Default metrics color
        }
    } else {
        metrics_color // Default metrics color if we can't check selection
    };

    match style {
        SortRenderStyle::TextBuffer => {
            // Square handle for text sorts
            let square_size = Vec2::new(normal_size * 2.0, normal_size * 2.0);
            gizmos.rect_2d(handle_position, square_size, handle_color);

            // Add bigger square indicator for selected sorts
            if let (Some(entity), Some(selected_query)) =
                (sort_entity, selected_query)
            {
                if selected_query
                    .iter()
                    .any(|selected_entity| selected_entity == entity)
                {
                    let big_square_size =
                        Vec2::new(normal_size * 3.0, normal_size * 3.0);
                    gizmos.rect_2d(
                        handle_position,
                        big_square_size,
                        Color::srgba(1.0, 1.0, 0.0, 0.5),
                    );
                }
            }
        }
        SortRenderStyle::Freeform => {
            // Circle handle for freeform sorts
            gizmos.circle_2d(handle_position, normal_size, handle_color);

            // Add bigger circle indicator for selected sorts
            if let (Some(entity), Some(selected_query)) =
                (sort_entity, selected_query)
            {
                if selected_query
                    .iter()
                    .any(|selected_entity| selected_entity == entity)
                {
                    gizmos.circle_2d(
                        handle_position,
                        normal_size * 1.5,
                        Color::srgba(1.0, 1.0, 0.0, 0.5),
                    );
                }
            }
        }
    }
}

/// FontIR-compatible version of render_sort_visuals
pub fn render_fontir_sort_visuals(
    gizmos: &mut Gizmos,
    fontir_app_state: &FontIRAppState,
    glyph_name: &str,
    advance_width: f32,
    metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
    style: SortRenderStyle,
) {
    // Get FontIR glyph paths
    if let Some(paths) = get_fontir_glyph_paths(fontir_app_state, glyph_name) {
        // Draw FontIR outline
        draw_fontir_glyph_outline_at_position(gizmos, &paths, position);
    }
    
    // Draw metrics (same as before)
    draw_metrics_at_position(
        gizmos,
        advance_width,
        metrics,
        position,
        metrics_color,
    );

    // Draw handle at descender position (same as before)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let handle_position = position + Vec2::new(0.0, descender);
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
