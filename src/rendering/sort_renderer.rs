//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Uses our thread-safe FontData structures for performance.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics};
use crate::rendering::cameras::DesignCamera;
use crate::ui::panes::design_space::ViewPort;
use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR};
use bevy::prelude::*;

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    let font_metrics = app_state.workspace.info.metrics();

    // Render inactive sorts
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, sort, &font_metrics, &app_state);
    }

    // Render active sorts
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, sort, &font_metrics, &app_state);
    }
}

/// Get the unicode value for a given glyph name
fn get_unicode_for_glyph(glyph_name: &str, app_state: &AppState) -> Option<String> {
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
        if let Some(&first_codepoint) = glyph_data.unicode_values.first() {
            return Some(format!("{:04X}", first_codepoint as u32));
        }
    }
    None
}

/// Helper function to check if a sort is within the viewport (for performance)
fn is_sort_in_viewport(sort: &Sort) -> bool {
    // For now, always return true (no culling)
    // TODO: Implement proper viewport intersection test when we have camera info
    true
}

/// Draw a metrics box for a sort
fn draw_sort_metrics_box(
    gizmos: &mut Gizmos,
    sort: &Sort,
    font_metrics: &FontMetrics,
    color: Color,
) {
    let descender = font_metrics.descender as f32;
    let ascender = font_metrics.ascender as f32;
    
    let bottom_left = sort.position + Vec2::new(0.0, descender);
    let top_right = sort.position + Vec2::new(sort.advance_width, ascender);
    
    // Draw metrics box outline
    gizmos.rect_2d(
        (bottom_left + top_right) / 2.0, // center
        top_right - bottom_left,         // size
        color,
    );
}

/// Draw simplified glyph outline
fn draw_contour_path_at_position(
    gizmos: &mut Gizmos,
    glyph_data: &crate::core::state::GlyphData,
    position: Vec2,
    color: Color,
) {
    let Some(outline_data) = &glyph_data.outline else {
        return;
    };

    for contour_data in &outline_data.contours {
        if contour_data.points.is_empty() {
            continue;
        }

        // Draw simple line segments between points
        let mut prev_point: Option<Vec2> = None;
        
        for point_data in &contour_data.points {
            let current_point = position + Vec2::new(point_data.x as f32, point_data.y as f32);
            
            if let Some(prev) = prev_point {
                gizmos.line_2d(prev, current_point, color);
            }
            
            prev_point = Some(current_point);
        }
        
        // Close the contour if we have points
        if let (Some(first), Some(last)) = (contour_data.points.first(), prev_point) {
            let first_point = position + Vec2::new(first.x as f32, first.y as f32);
            gizmos.line_2d(last, first_point, color);
        }
    }
}

/// Render an inactive sort with metrics box and glyph outline only
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Draw metrics box
    let _bounds = sort.get_metrics_bounds(font_metrics);
    
    // Get Unicode display for this glyph
    let unicode_display = get_unicode_for_glyph(&sort.glyph_name, app_state);
    
    let descender = font_metrics.descender as f32;
    let ascender = font_metrics.ascender as f32;
    
    // Render metrics box bounds
    gizmos.rect_2d(
        sort.position + Vec2::new(sort.advance_width / 2.0, (ascender + descender) / 2.0),
        Vec2::new(sort.advance_width, ascender - descender),
        SORT_INACTIVE_METRICS_COLOR,
    );

    // Render glyph outline if available
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        if let Some(outline_data) = &glyph_data.outline {
            // Draw each contour in the outline
            for contour_data in &outline_data.contours {
                // Convert our ContourData to norad::Contour for rendering
                let norad_contour = contour_data.to_norad_contour();
                // TODO: Get actual viewport from system - for now skip rendering complex outlines
                info!("Rendering glyph outline for {} with {} contours", sort.glyph_name, outline_data.contours.len());
            }
        }
    }

    // Show unicode in corner if available
    if let Some(unicode) = unicode_display {
        info!("Sort {}: Unicode {}", sort.glyph_name, unicode);
    }
}

/// Render an active sort with full outline detail
fn render_active_sort(
    gizmos: &mut Gizmos,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get Unicode display for this glyph
    let unicode_display = get_unicode_for_glyph(&sort.glyph_name, app_state);
    
    let descender = font_metrics.descender as f32;
    let ascender = font_metrics.ascender as f32;
    
    // Render enhanced metrics box for active sort
    gizmos.rect_2d(
        sort.position + Vec2::new(sort.advance_width / 2.0, (ascender + descender) / 2.0),
        Vec2::new(sort.advance_width, ascender - descender),
        SORT_ACTIVE_METRICS_COLOR,
    );

    // Render full glyph outline if available
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        if let Some(outline_data) = &glyph_data.outline {
            // Draw each contour in the outline with full detail
            for contour_data in &outline_data.contours {
                // Convert our ContourData to norad::Contour for rendering
                let _norad_contour = contour_data.to_norad_contour();
                // TODO: Render actual glyph outlines with control points
                info!("Rendering active glyph outline for {} with {} contours", sort.glyph_name, outline_data.contours.len());
            }
        }
    }

    // Show unicode in corner if available
    if let Some(unicode) = unicode_display {
        info!("Active Sort {}: Unicode {}", sort.glyph_name, unicode);
    }
}

// Stub functions for sort text management to satisfy the plugin
pub fn render_sort_text() {}
pub fn update_sort_text_content() {}
pub fn update_sort_text_positions() {} 