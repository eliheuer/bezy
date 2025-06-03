//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Inactive sorts show as metrics boxes (similar to existing metrics display).
//! Active sorts show the actual glyph outlines for editing.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics};
use crate::ui::panes::design_space::ViewPort;
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

    // Render inactive sorts as metrics boxes with glyph outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, &viewport, sort, font_metrics);
    }

    // Render active sorts with full outline detail
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, font_metrics);
    }
}

/// Render an inactive sort with metrics box and glyph outline
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
) {
    // First render the metrics box using the shared function
    crate::rendering::metrics::draw_metrics_at_position(
        gizmos,
        viewport,
        &sort.glyph,
        font_metrics,
        sort.position,
    );
    
    // Then render the glyph outline if it exists
    if let Some(outline) = &sort.glyph.outline {
        crate::rendering::glyph_outline::draw_glyph_outline_at_position(
            gizmos,
            viewport,
            outline,
            sort.position,
        );
    }
}

/// Render an active sort with full glyph outline and control handles
fn render_active_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
) {
    // First render the metrics box using the shared function
    crate::rendering::metrics::draw_metrics_at_position(
        gizmos,
        viewport,
        &sort.glyph,
        font_metrics,
        sort.position,
    );
    
    // Then render the glyph outline with full detail if it exists
    if let Some(outline) = &sort.glyph.outline {
        crate::rendering::glyph_outline::draw_glyph_outline_at_position(
            gizmos,
            viewport,
            outline,
            sort.position,
        );
    }
} 