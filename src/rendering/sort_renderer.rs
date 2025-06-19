//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Uses our thread-safe FontData structures for performance.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics, GlyphData, OutlineData, ContourData, PointTypeData};
use crate::ui::panes::design_space::{ViewPort, DPoint};
use crate::ui::theme::{
    SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR, 
    PATH_LINE_COLOR, HANDLE_LINE_COLOR, ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS,
    OFF_CURVE_POINT_COLOR, OFF_CURVE_POINT_RADIUS, USE_SQUARE_FOR_ON_CURVE,
    ON_CURVE_SQUARE_ADJUSTMENT, ON_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_INNER_CIRCLE_RATIO,
};
use bevy::prelude::*;

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewport: Res<ViewPort>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    let font_metrics = app_state.workspace.info.metrics();

    // Render inactive sorts as metrics boxes with glyph outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, &viewport, sort, &font_metrics, &app_state);
    }

    // Render active sorts with full outline detail
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, &font_metrics, &app_state);
    }
}

/// Render an inactive sort with metrics box and glyph outline only
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our thread-safe font data
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        // First render the metrics box using the inactive color
        draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            glyph_data,
            font_metrics,
            sort.position,
            SORT_INACTIVE_METRICS_COLOR,
        );
        
        // Then render the glyph outline if it exists (without control handles for inactive sorts)
        if let Some(outline) = &glyph_data.outline {
            // Render the glyph outline with cubic curves
            crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
        }
    }
}

/// Render an active sort with full glyph outline and control handles
fn render_active_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our thread-safe font data
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        // First render the metrics box using the active color
        draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            glyph_data,
            font_metrics,
            sort.position,
            SORT_ACTIVE_METRICS_COLOR,
        );
        
        // Then render the full glyph outline with control handles if it exists
        if let Some(outline) = &glyph_data.outline {
            // Use the proper cubic curve rendering functions from the glyph_outline module
            crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
            
            // Render control handles for better curve visualization
            crate::rendering::glyph_outline::draw_control_handles_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
            
            // Also render the glyph points (on-curve and off-curve)
            crate::rendering::glyph_outline::draw_glyph_points_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
        }
    }
}

/// Draw metrics box and guidelines for a glyph at a specific position
fn draw_metrics_at_position_with_color(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph_data: &GlyphData,
    font_metrics: &FontMetrics,
    position: Vec2,
    metrics_color: Color,
) {
    let upm = font_metrics.units_per_em;
    let x_height = font_metrics.x_height.unwrap_or_else(|| (upm * 0.5).round());
    let cap_height = font_metrics.cap_height.unwrap_or_else(|| (upm * 0.7).round());
    let ascender = font_metrics.ascender;
    let descender = font_metrics.descender;
    let width = glyph_data.advance_width;

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
}



/// Helper function to draw a line in viewport coordinates
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

/// Helper function to draw a rectangle in viewport coordinates
fn draw_rect(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    bottom_left: (f32, f32),
    top_right: (f32, f32),
    color: Color,
) {
    let bl_screen = viewport.to_screen(DPoint::from(bottom_left));
    let tr_screen = viewport.to_screen(DPoint::from(top_right));
    
    let center = (bl_screen + tr_screen) / 2.0;
    let size = tr_screen - bl_screen;
    
    gizmos.rect_2d(center, size, color);
}

// Stub functions for sort text management to satisfy the plugin
pub fn render_sort_text() {}
pub fn update_sort_text_content() {}
pub fn update_sort_text_positions() {} 