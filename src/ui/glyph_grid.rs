//! Glyph Grid System
//!
//! Creates a grid of sorts at startup showing all glyphs in the loaded font.
//! This provides an overview of all available glyphs/codepoints, similar to
//! traditional font editors but with freeform sort placement instead of fixed squares.
//!
//! The grid snaps to the checkerboard grid and arranges glyphs in rows of 32
//! codepoints by default. Users can rearrange these sorts as needed.

use bevy::prelude::*;
use crate::core::settings::BezySettings;
use crate::core::state::AppState;
use crate::editing::sort::{Sort, SortEvent, SortBounds};
use crate::core::state::navigation::get_all_codepoints;

/// System to create the glyph grid after font load
/// 
/// This system runs once after the font is loaded and no sorts are present.
pub fn create_glyph_grid_once(
    mut sort_events: EventWriter<SortEvent>,
    app_state: Res<AppState>,
    settings: Res<BezySettings>,
    sorts_query: Query<Entity, With<Sort>>,
    mut has_run: Local<bool>,
) {
    debug!("[GlyphGrid] System running. has_run: {}, font glyphs: {}, sorts in world: {}", *has_run, app_state.workspace.font.glyphs.len(), sorts_query.iter().count());
    if *has_run {
        debug!("[GlyphGrid] Already ran, skipping.");
        return;
    }
    if !settings.glyph_grid.enabled {
        debug!("[GlyphGrid] Disabled in settings, skipping.");
        *has_run = true;
        return;
    }
    if app_state.workspace.font.glyphs.is_empty() {
        debug!("[GlyphGrid] Font not loaded yet, skipping.");
        return;
    }
    if !sorts_query.is_empty() {
        debug!("[GlyphGrid] Sorts already present, skipping.");
        *has_run = true;
        return;
    }

    let grid_size = crate::ui::theme::CHECKERBOARD_DEFAULT_UNIT_SIZE;
    let font_metrics = &app_state.workspace.info.metrics;
    let upm = font_metrics.units_per_em as f32;
    let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
    let max_row_width = 16.0 * upm;
    let row_gap = grid_size;

    // Get all codepoints in Unicode order
    let codepoints = crate::core::state::navigation::get_all_codepoints(&app_state);
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut placed = 0;
    let mut row_sorts: Vec<(String, f32, f32)> = Vec::new(); // (glyph_name, advance_width, row_height)
    let mut max_row_height = 0.0;

    for codepoint_hex in codepoints {
        let codepoint = u32::from_str_radix(&codepoint_hex, 16).ok().and_then(std::char::from_u32);
        if let Some(cp) = codepoint {
            let glyph_name = app_state.workspace.font.glyphs.iter()
                .find(|(_name, glyph)| glyph.unicode_values.contains(&cp))
                .map(|(name, _)| name.clone());
            if let Some(glyph_name) = glyph_name {
                let glyph_data = &app_state.workspace.font.glyphs[&glyph_name];
                let advance_width = glyph_data.advance_width as f32;
                let snapped_advance = (advance_width / grid_size).ceil() * grid_size;
                let sort_bounds = SortBounds {
                    min: Vec2::new(0.0, descender),
                    max: Vec2::new(advance_width, upm),
                };
                let sort_height = upm - descender;

                // If this glyph would overflow the row, wrap to next row
                if current_x + snapped_advance > max_row_width && current_x > 0.0 {
                    // Place all sorts in the row at the same baseline Y
                    current_y -= max_row_height + row_gap;
                    current_x = 0.0;
                    max_row_height = 0.0;
                }

                // Place sort so the baseline is at current_y, and the top of the UPM is at current_y + (upm - 0)
                let position = Vec2::new(current_x, current_y);
                debug!("[GlyphGrid] Spawning sort for codepoint U+{} (glyph '{}') at position {:?}, bounds: {:?}", codepoint_hex, glyph_name, position, sort_bounds);
                sort_events.write(SortEvent::CreateSort {
                    glyph_name: glyph_name.clone(),
                    position,
                });

                current_x += snapped_advance;
                placed += 1;
                if sort_height > max_row_height {
                    max_row_height = sort_height;
                }
            }
        }
    }

    debug!("[GlyphGrid] Creation complete: {} codepoints placed", placed);
    *has_run = true;
}

/// System to snap existing sorts to the grid
#[allow(dead_code)]
pub fn snap_sorts_to_grid(
    mut sort_query: Query<&mut Transform, With<Sort>>,
    settings: Res<BezySettings>,
) {
    let grid_size = settings.glyph_grid.grid_size;
    for mut transform in sort_query.iter_mut() {
        let position = transform.translation.truncate();
        let snapped_x = (position.x / grid_size).round() * grid_size;
        let snapped_y = (position.y / grid_size).round() * grid_size;
        transform.translation.x = snapped_x;
        transform.translation.y = snapped_y;
    }
}

/// Debug system to print all Sort entities and their positions
pub fn debug_print_sorts(
    sorts_query: Query<(&crate::editing::sort::Sort, &Transform)>,
) {
    let mut count = 0;
    for (sort, transform) in sorts_query.iter() {
        info!("[GlyphGrid][DEBUG] Sort '{}' at position {:?}", sort.glyph_name, transform.translation);
        count += 1;
    }
    info!("[GlyphGrid][DEBUG] Total sorts in world: {}", count);
}

/// Plugin for the glyph grid system
pub struct GlyphGridPlugin;

impl Plugin for GlyphGridPlugin {
    fn build(&self, app: &mut App) {
        info!("[GlyphGrid] Registering GlyphGridPlugin");
        app.add_systems(Update, create_glyph_grid_once);
        app.add_systems(Update, debug_print_sorts);
        info!("[GlyphGrid] GlyphGridPlugin registration complete");
    }
} 