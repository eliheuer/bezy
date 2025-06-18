//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::AppState;
use crate::ui::panes::design_space::ViewPort;
use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR};
use bevy::prelude::*;

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: NonSend<AppState>,
    viewport: Res<ViewPort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    // Only render if we have a font loaded
    let Some(font) = &app_state.font else {
        return;
    };

    let Some(font_metrics) = &app_state.metrics else {
        return;
    };

    let default_layer = font.default_layer();

    // Render inactive sorts as simple outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, &viewport, sort, font_metrics, default_layer);
    }

    // Render active sorts with highlights
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, font_metrics, default_layer);
    }
}

/// Render an inactive sort with simple outline
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &crate::core::state::FontMetrics,
    default_layer: &norad::Layer,
) {
    let Some(glyph) = default_layer.get_glyph(&sort.glyph_name) else {
        return;
    };

    // Draw simple metrics using the metrics module
    crate::rendering::metrics::draw_metrics_at_position_with_color(
        gizmos, viewport, glyph, font_metrics, sort.position, SORT_INACTIVE_METRICS_COLOR
    );

    // For performance, don't draw glyph outlines for inactive sorts
    // Only show metrics to keep rendering lightweight
}

/// Render an active sort with highlights
fn render_active_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &crate::core::state::FontMetrics,
    default_layer: &norad::Layer,
) {
    let Some(glyph) = default_layer.get_glyph(&sort.glyph_name) else {
        return;
    };

    // Draw comprehensive metrics using the metrics module
    crate::rendering::metrics::draw_metrics_at_position_with_color(
        gizmos, viewport, glyph, font_metrics, sort.position, SORT_ACTIVE_METRICS_COLOR
    );

    // Draw full glyph outline with control handles for active sorts
    if !glyph.contours.is_empty() {
        crate::rendering::glyph_outline::draw_glyph_outline_at_position(
            gizmos, viewport, &glyph.contours, sort.position
        );
    }
}





// Stub functions for sort text management to satisfy the plugin
pub fn manage_sort_unicode_text() {}
pub fn update_sort_unicode_text_positions() {}
pub fn update_sort_unicode_text_colors() {} 