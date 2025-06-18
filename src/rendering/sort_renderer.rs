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

    // Draw simple metrics box
    draw_simple_metrics_box(gizmos, viewport, glyph, font_metrics, sort.position, SORT_INACTIVE_METRICS_COLOR);

    // Draw simple glyph outline using the contours directly
    if !glyph.contours.is_empty() {
        draw_simple_glyph_contours(gizmos, viewport, &glyph.contours, sort.position);
    }
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

    // Draw highlighted metrics box
    draw_simple_metrics_box(gizmos, viewport, glyph, font_metrics, sort.position, SORT_ACTIVE_METRICS_COLOR);

    // Draw glyph outline using the contours directly
    if !glyph.contours.is_empty() {
        draw_simple_glyph_contours(gizmos, viewport, &glyph.contours, sort.position);
    }
}

/// Draw a simple metrics box
fn draw_simple_metrics_box(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &norad::Glyph,
    font_metrics: &crate::core::state::FontMetrics,
    position: Vec2,
    color: Color,
) {
    let upm = font_metrics.units_per_em as f32;
    let width = glyph.width as f32;
    
    // Draw simple box - position.y is the baseline
    // Descender goes below baseline (negative Y in font space)
    // Ascender goes above baseline (positive Y in font space)
    let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
    let ascender = font_metrics.ascender.unwrap_or(800.0) as f32;
    
    let min_x = position.x;
    let max_x = position.x + width;
    let min_y = position.y + descender; // Below baseline
    let max_y = position.y + ascender;  // Above baseline
    
    // Convert to screen space
    let bl_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(min_x, min_y));
    let br_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(max_x, min_y));
    let tl_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(min_x, max_y));
    let tr_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(max_x, max_y));
    
    // Draw box outline
    gizmos.line_2d(bl_screen, br_screen, color);
    gizmos.line_2d(br_screen, tr_screen, color);
    gizmos.line_2d(tr_screen, tl_screen, color);
    gizmos.line_2d(tl_screen, bl_screen, color);
    
    // Draw baseline (horizontal line at Y=0 relative to position)
    let baseline_start = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(min_x, position.y));
    let baseline_end = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(max_x, position.y));
    gizmos.line_2d(baseline_start, baseline_end, Color::srgba(1.0, 0.0, 0.0, 1.0));
}

/// Draw a simple glyph outline using the contours directly
fn draw_simple_glyph_contours(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contours: &[norad::Contour],
    position: Vec2,
) {
    for contour in contours {
        if contour.points.is_empty() {
            continue;
        }
        
        // Draw simple lines between consecutive points
        for i in 0..contour.points.len() {
            let current = &contour.points[i];
            let next = &contour.points[(i + 1) % contour.points.len()];
            
            let start_pos = position + Vec2::new(current.x as f32, -current.y as f32);
            let end_pos = position + Vec2::new(next.x as f32, -next.y as f32);
            
            let start_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(start_pos.x, start_pos.y));
            let end_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::new(end_pos.x, end_pos.y));
            
            gizmos.line_2d(start_screen, end_screen, Color::WHITE);
        }
    }
}

// Stub functions for sort text management to satisfy the plugin
pub fn manage_sort_unicode_text() {}
pub fn update_sort_unicode_text_positions() {}
pub fn update_sort_unicode_text_colors() {} 