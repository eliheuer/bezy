//! Glyph Grid System
//!
//! Creates a grid of sorts at startup showing all glyphs in the loaded font.
//! This provides an overview of all available glyphs/codepoints, similar to
//! traditional font editors but with freeform sort placement instead of fixed 
//! squares in a separate view.
//!
//! This is designed to make working with Arabic fonts easier, or any font with
//! both narrow and wide drawings.
//!
//! The grid snaps to the checkerboard grid and arranges glyphs in rows of 32
//! codepoints by default. Users can rearrange these sorts as needed.

use bevy::prelude::*;
use crate::core::settings::BezySettings;
use crate::core::state::AppState;
use crate::editing::sort::{Sort, SortEvent};
use crate::core::state::navigation::get_all_codepoints;
use crate::core::state::SortLayoutMode;
use crate::ui::theme::{MIN_GLYPH_GRID_GAP_MULTIPLIER, MAX_GLYPH_GRID_ROW_WIDTH_UPM};

/// System to create the glyph grid after a font is loaded
pub fn create_glyph_grid_once(
    mut sort_events: EventWriter<SortEvent>,
    app_state: Res<AppState>,
    settings: Res<BezySettings>,
    sorts_query: Query<Entity, With<Sort>>,
    mut has_run: Local<bool>,
) {
    if !should_create_glyph_grid(
        &app_state, &settings,
        &sorts_query, *has_run
    ) {
        if !*has_run {*has_run = true;} return;
    }
    debug!("[GlyphGrid] Creating glyph grid...");
    let layout_config = GridLayoutConfig::from_app_state(&app_state);
    let glyph_positions = calculate_glyph_positions(&app_state, &layout_config);
    let positions_count = glyph_positions.len();
    for (glyph_name, position) in glyph_positions {
        sort_events.write(SortEvent::CreateSort {
            glyph_name,
            position,
            layout_mode: SortLayoutMode::Freeform,
        });
    }
    debug!("[GlyphGrid] Creation complete: {} sorts placed", positions_count);
    *has_run = true;
}

/// Configuration for grid layout calculations
#[derive(Debug)]
struct GridLayoutConfig {
    grid_size: f32,
    min_gap: f32,
    upm: f32,
    descender: f32,
    max_row_width: f32,
}

impl GridLayoutConfig {
    fn from_app_state(app_state: &AppState) -> Self {
        let grid_size = crate::ui::theme::CHECKERBOARD_DEFAULT_UNIT_SIZE;
        let font_metrics = &app_state.workspace.info.metrics;
        let upm = font_metrics.units_per_em as f32;
        let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
        
        Self {
            grid_size,
            min_gap: MIN_GLYPH_GRID_GAP_MULTIPLIER * grid_size,
            upm,
            descender,
            max_row_width: MAX_GLYPH_GRID_ROW_WIDTH_UPM * upm,
        }
    }
}

/// Determines if the glyph grid should be created
fn should_create_glyph_grid(
    app_state: &AppState,
    settings: &BezySettings,
    sorts_query: &Query<Entity, With<Sort>>,
    has_run: bool,
) -> bool {
    debug!(
        "[GlyphGrid] System running. has_run: {}, glyphs: {}, sorts: {}", 
        has_run, 
        app_state.workspace.font.glyphs.len(), 
        sorts_query.iter().count()
    );
    
    if has_run {
        debug!("[GlyphGrid] Already ran, skipping.");
        return false;
    }
    
    if !settings.glyph_grid.enabled {
        debug!("[GlyphGrid] Disabled in settings, skipping.");
        return false;
    }
    
    if app_state.workspace.font.glyphs.is_empty() {
        debug!("[GlyphGrid] Font not loaded yet, skipping.");
        return false;
    }
    
    if !sorts_query.is_empty() {
        debug!("[GlyphGrid] Sorts already exist, skipping.");
        return false;
    }
    
    true
}

/// Calculates positions for all glyphs in the grid layout
fn calculate_glyph_positions(
    app_state: &AppState,
    config: &GridLayoutConfig,
) -> Vec<(String, Vec2)> {
    let codepoints = get_all_codepoints(app_state);
    let mut positions = Vec::new();
    let mut layout_state = GridLayoutState::new();
    
    for codepoint_hex in codepoints {
        if let Some(glyph_name) = find_glyph_for_codepoint(app_state, &codepoint_hex) {
            if let Some(position) = layout_state.place_glyph(app_state, &glyph_name, config) {
                positions.push((glyph_name, position));
            }
        }
    }
    
    positions
}

/// Finds the glyph name for a given codepoint
fn find_glyph_for_codepoint(app_state: &AppState, codepoint_hex: &str) -> Option<String> {
    let codepoint = u32::from_str_radix(codepoint_hex, 16)
        .ok()
        .and_then(std::char::from_u32)?;
    
    app_state.workspace.font.glyphs.iter()
        .find(|(_name, glyph)| glyph.unicode_values.contains(&codepoint))
        .map(|(name, _)| name.clone())
}

/// Tracks the current state of grid layout positioning
#[derive(Debug)]
struct GridLayoutState {
    current_x: f32,
    current_y: f32,
    max_row_height: f32,
    max_row_height_for_snap: f32,
}

impl GridLayoutState {
    fn new() -> Self {
        Self {
            current_x: 0.0,
            current_y: 0.0,
            max_row_height: 0.0,
            max_row_height_for_snap: 0.0,
        }
    }
    
    fn place_glyph(
        &mut self,
        app_state: &AppState,
        glyph_name: &str,
        config: &GridLayoutConfig,
    ) -> Option<Vec2> {
        let glyph_data = app_state.workspace.font.glyphs.get(glyph_name)?;
        let advance_width = glyph_data.advance_width as f32;
        let snapped_advance = ((advance_width + config.min_gap) / config.grid_size).ceil() * config.grid_size;
        
        // Check if this sort would overflow the row
        if self.current_x + snapped_advance > config.max_row_width {
            self.start_new_row();
        }
        
        let position = self.calculate_snapped_position(config);
        self.update_row_height(app_state, glyph_name, config);
        
        debug!(
            "[GlyphGrid] Placing '{}' at ({:.1}, {:.1})", 
            glyph_name, position.x, position.y
        );
        
        // Move to next position
        self.current_x += snapped_advance;
        
        Some(position)
    }
    
    fn start_new_row(&mut self) {
        self.current_x = 0.0;
        self.current_y -= self.max_row_height_for_snap;
        self.max_row_height = 0.0;
    }
    
    fn calculate_snapped_position(&self, config: &GridLayoutConfig) -> Vec2 {
        let snapped_x = (self.current_x / config.grid_size).round() * config.grid_size;
        let snapped_y = (self.current_y / config.grid_size).round() * config.grid_size;
        Vec2::new(snapped_x, snapped_y)
    }
    
    fn update_row_height(
        &mut self,
        app_state: &AppState,
        glyph_name: &str,
        config: &GridLayoutConfig,
    ) {
        let _glyph_data = &app_state.workspace.font.glyphs[glyph_name];
        let sort_height = config.upm - config.descender;
        
        self.max_row_height = self.max_row_height.max(sort_height);
        self.max_row_height_for_snap = ((self.max_row_height + config.min_gap) / config.grid_size).ceil() * config.grid_size;
    }
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
        info!(
            "[GlyphGrid][DEBUG] Sort '{}' at position {:?}", 
            sort.glyph_name, 
            transform.translation
        );
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