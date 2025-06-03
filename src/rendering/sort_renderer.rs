//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Inactive sorts show as metrics boxes (similar to existing metrics display).
//! Active sorts show the actual glyph outlines for editing.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics};
use crate::ui::panes::design_space::{DPoint, ViewPort};
use crate::ui::theme::{PATH_LINE_COLOR, ON_CURVE_POINT_COLOR};
use bevy::prelude::*;

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    // Get viewport for coordinate transformations
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };

    let font_metrics = &app_state.workspace.info.metrics;

    // Render inactive sorts as metrics boxes
    for sort in inactive_sorts_query.iter() {
        render_sort_metrics_box(&mut gizmos, &viewport, sort, font_metrics);
    }

    // Render active sorts with full outline detail
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, font_metrics);
    }
}

/// Render a sort as a metrics box (for inactive sorts)
fn render_sort_metrics_box(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
) {
    // Use the shared metrics rendering function to ensure consistency
    crate::rendering::metrics::draw_metrics_at_position(
        gizmos,
        viewport,
        &sort.glyph,
        font_metrics,
        sort.position,
    );
}

/// Render an active sort with full glyph outline
fn render_active_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
) {
    // First render the metrics box using the shared function
    render_sort_metrics_box(gizmos, viewport, sort, font_metrics);
    
    // Then render the actual glyph outline if it exists
    if let Some(outline) = &sort.glyph.outline {
        render_glyph_outline(gizmos, viewport, outline, sort.position);
    }
}

/// Render a glyph outline at the given position
fn render_glyph_outline(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    outline: &norad::glyph::Outline,
    offset: Vec2,
) {
    // Render each contour in the outline
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }

        // Draw lines between consecutive points
        for i in 0..contour.points.len() {
            let current_point = &contour.points[i];
            let next_index = (i + 1) % contour.points.len();
            let next_point = &contour.points[next_index];

            let current_pos = offset + Vec2::new(current_point.x as f32, current_point.y as f32);
            let next_pos = offset + Vec2::new(next_point.x as f32, next_point.y as f32);

            // Draw line between points
            draw_line(gizmos, viewport, current_pos, next_pos, PATH_LINE_COLOR);

            // Draw the point itself
            let point_screen = viewport.to_screen(DPoint::from((current_pos.x, current_pos.y)));
            gizmos.circle_2d(point_screen, 3.0, ON_CURVE_POINT_COLOR);
        }
    }
}

/// Draw a line in design space
fn draw_line(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    start: Vec2,
    end: Vec2,
    color: Color,
) {
    let start_screen = viewport.to_screen(DPoint::from((start.x, start.y)));
    let end_screen = viewport.to_screen(DPoint::from((end.x, end.y)));
    gizmos.line_2d(start_screen, end_screen, color);
} 