//! Sort management systems
//!
//! This module contains simplified versions of sort management systems
//! to get basic sort functionality working.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort, SortEvent, ActiveSortState};
use crate::core::state::AppState;
use bevy::prelude::*;

/// Basic system to handle sort events
pub fn handle_sort_events(
    mut commands: Commands,
    mut sort_events: EventReader<SortEvent>,
    mut active_sort_state: ResMut<ActiveSortState>,
    app_state: NonSend<AppState>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort { glyph_name, position } => {
                info!("Creating sort for glyph '{}' at position ({:.1}, {:.1})", glyph_name, position.x, position.y);
                
                // Get advance width from the font
                let advance_width = if let Some(font) = &app_state.font {
                    let default_layer = font.default_layer();
                    if let Some(glyph) = default_layer.get_glyph(glyph_name) {
                        glyph.width as f32
                    } else {
                        600.0
                    }
                } else {
                    600.0
                };

                let sort = Sort::new(glyph_name.clone(), *position, advance_width);
                commands.spawn((sort, InactiveSort));
            }
            SortEvent::ActivateSort { sort_entity } => {
                info!("Activating sort {:?}", sort_entity);
                if let Ok(mut entity_commands) = commands.get_entity(*sort_entity) {
                    entity_commands.remove::<InactiveSort>().insert(ActiveSort);
                    active_sort_state.active_sort_entity = Some(*sort_entity);
                }
            }
            SortEvent::DeactivateSort => {
                info!("Deactivating current sort");
                for entity in active_sorts_query.iter() {
                    commands.entity(entity).remove::<ActiveSort>().insert(InactiveSort);
                }
                active_sort_state.active_sort_entity = None;
            }
            _ => {} // Handle other events as needed
        }
    }
}

/// System to spawn initial sorts when font is loaded
pub fn spawn_initial_sort(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
    app_state: NonSend<AppState>,
) {
    // Only spawn if there are no sorts yet and we have a font loaded
    if !sorts_query.is_empty() || app_state.font.is_none() {
        return;
    }

    let font = app_state.font.as_ref().unwrap();
    let default_layer = font.default_layer();

    info!("Spawning initial sorts for font");

    const GLYPHS_PER_ROW: usize = 16;
    const HORIZONTAL_PADDING: f32 = 64.0;
    const MAX_INITIAL_SORTS: usize = 12; // Limit to prevent performance issues

    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut glyph_count_in_row = 0;

    // Create sorts for only the first few glyphs to avoid performance issues
    let glyphs: Vec<_> = default_layer.iter().collect();
    
    for glyph in glyphs.iter().take(MAX_INITIAL_SORTS) { // Only process first 12 glyphs
        if glyph_count_in_row >= GLYPHS_PER_ROW {
            current_y += 1200.0; // Move to next row (positive Y goes down in screen space)
            current_x = 0.0;
            glyph_count_in_row = 0;
        }

        // Position sorts with proper baseline - Y=0 should be the baseline
        // Add some offset so glyphs don't start at the very edge
        let sort_position = Vec2::new(current_x + 50.0, current_y + 200.0);

        sort_events.write(SortEvent::CreateSort {
            glyph_name: glyph.name().to_string(),
            position: sort_position,
        });

        let advance = glyph.width as f32;
        current_x += advance + HORIZONTAL_PADDING;
        glyph_count_in_row += 1;
    }
}

/// Auto-activate the first sort
pub fn auto_activate_first_sort(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // Only activate if no sorts are active
    if !active_sorts_query.is_empty() {
        return;
    }

    // Activate the first inactive sort
    if let Some(first_sort) = sorts_query.iter().next() {
        sort_events.write(SortEvent::ActivateSort { sort_entity: first_sort });
    }
}

/// Ensure only one sort is active at a time
pub fn enforce_single_active_sort(
    mut commands: Commands,
    active_sorts: Query<Entity, With<ActiveSort>>,
) {
    let active_count = active_sorts.iter().count();
    if active_count > 1 {
        warn!("Multiple active sorts detected ({}), keeping only the first", active_count);
        
        // Keep the first one, deactivate the rest
        for (index, entity) in active_sorts.iter().enumerate() {
            if index > 0 {
                commands.entity(entity).remove::<ActiveSort>().insert(InactiveSort);
            }
        }
    }
}

// Stub systems to satisfy the plugin requirements - these can be expanded later
pub fn sync_sort_transforms() {}
pub fn handle_glyph_navigation_changes() {}
pub fn spawn_sort_point_entities() {}
pub fn manage_sort_crosshairs() {}
pub fn manage_newly_spawned_crosshairs() {}
pub fn respawn_sort_points_on_glyph_change() {}
pub fn debug_sort_point_entities() {}
pub fn update_sort_glyph_data() {}
pub fn update_sort_from_crosshair_move() {}
pub fn sync_points_to_sort_move() {}
pub fn sync_crosshair_to_sort_move() {}
pub fn render_sort_crosshairs() {} 