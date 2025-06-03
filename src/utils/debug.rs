//! Debug utilities for font rendering and metrics
//!
//! This module contains debug functions that were previously scattered
//! throughout the rendering code. It provides clean separation between
//! debug functionality and core rendering logic.

use crate::core::state::{AppState, FontMetrics};
use bevy::prelude::*;
use norad::Glyph;

/// Debug font metrics information
///
/// Logs detailed information about the current font's metrics
/// including units per em, x-height, cap-height, ascender, and descender.
pub fn debug_font_metrics(app_state: &AppState) {
    debug!("=== Font Metrics Debug ===");
    debug!(
        "Has font_info: {}",
        app_state.workspace.font.ufo.font_info.is_some()
    );

    if app_state.workspace.font.ufo.font_info.is_some() {
        let metrics = &app_state.workspace.info.metrics;
        debug!("Units per em: {}", metrics.units_per_em);
        debug!("X-height: {:?}", metrics.x_height);
        debug!("Cap-height: {:?}", metrics.cap_height);
        debug!("Ascender: {:?}", metrics.ascender);
        debug!("Descender: {:?}", metrics.descender);
    }
}

/// Debug glyph information
///
/// Logs information about a specific glyph including its name,
/// advance width, and outline status.
pub fn debug_glyph_info(glyph: &Glyph, context: &str) {
    debug!(
        "{}: Glyph '{}' - advance width: {:?}, has outline: {}",
        context,
        glyph.name,
        glyph.advance.as_ref().map(|a| a.width),
        glyph.outline.is_some()
    );
}

/// Debug viewport information
///
/// Logs viewport properties like zoom and flip state.
pub fn debug_viewport_info(viewport: &crate::ui::panes::design_space::ViewPort) {
    debug!(
        "Viewport - zoom: {}, flipped_y: {}",
        viewport.zoom, viewport.flipped_y
    );
}

/// Debug layer information
///
/// Logs information about font layers and available glyphs.
pub fn debug_layer_info(app_state: &AppState) {
    match app_state.workspace.font.ufo.get_default_layer() {
        Some(layer) => {
            debug!("Default layer found with {} glyphs", layer.iter_contents().count());
            
            // Log first few glyph names for reference
            let glyph_names: Vec<String> = layer
                .iter_contents()
                .take(5)
                .map(|glyph| glyph.name.to_string())
                .collect();
            
            if !glyph_names.is_empty() {
                debug!("Sample glyphs: {}", glyph_names.join(", "));
            }
        }
        None => {
            debug!("No default layer found");
        }
    }
}

/// Debug placeholder glyph creation
///
/// Logs when a placeholder glyph is created due to missing glyphs.
pub fn debug_placeholder_creation(metrics: &FontMetrics) {
    warn!(
        "Creating placeholder glyph - no real glyphs found. \
         Width will be {} units (40% of UPM: {})",
        (metrics.units_per_em as f32 * 0.4) as i32,
        metrics.units_per_em as i32
    );
}

/// Debug missing glyph information
///
/// Logs when a specific glyph cannot be found.
pub fn debug_missing_glyph(glyph_name: &str) {
    warn!("Glyph '{}' not found in font", glyph_name);
}

/// Debug codepoint lookup
///
/// Logs information about codepoint-to-glyph mapping attempts.
pub fn debug_codepoint_lookup(codepoint: &str, found: bool) {
    if found {
        debug!("Successfully found glyph for codepoint U+{}", codepoint);
    } else {
        warn!("No glyph found for codepoint U+{}", codepoint);
    }
} 