//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Uses our thread-safe FontData structures for performance.

use crate::core::state::{AppState, GlyphNavigation, FontData};
use crate::editing::selection::components::{
    GlyphPointReference, Selectable, SelectionState,
};
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};
use bevy::prelude::*;
use std::collections::HashMap;

/// Convert a Unicode character to its hex codepoint string
fn char_to_hex_codepoint(unicode_char: char) -> String {
    format!("{:04X}", unicode_char as u32)
}

/// Build a map of Unicode codepoints to glyph names for efficient lookups
fn build_codepoint_glyph_map(font_data: &FontData) -> HashMap<String, String> {
    let mut codepoint_to_glyph = HashMap::new();
    
    for glyph_data in font_data.glyphs.values() {
        for &unicode_char in &glyph_data.unicode_values {
            let codepoint_hex = char_to_hex_codepoint(unicode_char);
            codepoint_to_glyph.insert(codepoint_hex, glyph_data.name.clone());
        }
    }
    
    codepoint_to_glyph
}

/// Find a glyph by its Unicode codepoint (like "0041" for letter A)
pub fn find_glyph_by_unicode(
    font_data: &FontData, 
    codepoint_hex: &str
) -> Option<String> {
    build_codepoint_glyph_map(font_data).get(codepoint_hex).cloned()
}

/// Helper to calculate the desired position of the crosshair.
/// Places it at the lower-left of the sort's metrics box, offset inward by 64 units.
fn get_crosshair_position(sort: &Sort, app_state: &AppState) -> Vec2 {
    // Get the center of the glyph for the crosshair position
    let glyph_data = app_state.workspace.font.get_glyph(&sort.glyph_name);
    let metrics = app_state.workspace.info.metrics();
    
    // Default to center of advance width if we can't get specific glyph bounds
    let center_x = sort.position.x + (sort.advance_width / 2.0);
    let center_y = sort.position.y + (metrics.ascender as f32 / 2.0);
    
    Vec2::new(center_x, center_y)
}

/// Component to mark point entities that belong to a sort
#[derive(Component, Debug)]
pub struct SortPointEntity {
    /// The sort entity this point belongs to
    pub sort_entity: Entity,
}

/// Component to mark crosshair entities that can move a sort
#[derive(Component, Debug)]
pub struct SortCrosshair {
    /// The sort entity this crosshair controls
    pub sort_entity: Entity,
}

/// Component to mark crosshairs that were just spawned and shouldn't be moved yet
#[derive(Component, Debug)]
pub struct NewlySpawnedCrosshair {
    /// Number of frames to wait before allowing movement
    pub frames_remaining: u32,
}

/// System to handle sort events
pub fn handle_sort_events(
    mut commands: Commands,
    mut sort_events: EventReader<SortEvent>,
    mut active_sort_state: ResMut<ActiveSortState>,
    app_state: Res<AppState>,
    _sorts_query: Query<(Entity, &Sort)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort { glyph_name, position } => {
                // Get advance width from our thread-safe font data
                let advance_width = if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
                    glyph_data.advance_width as f32
                } else {
                    warn!("Glyph '{}' not found in font", glyph_name);
                    600.0 // Default width
                };

                // Get metrics for proper positioning
                let _metrics = app_state.workspace.info.metrics();

                let sort = Sort::new(glyph_name.clone(), *position, advance_width);

                let sort_entity = commands.spawn((sort, InactiveSort)).id();
                info!("Created sort for glyph '{}' at position {:?}", glyph_name, position);
            }
            SortEvent::ActivateSort { sort_entity } => {
                activate_sort(&mut commands, &mut active_sort_state, *sort_entity, &active_sorts_query);
            }
            SortEvent::DeactivateSort => {
                deactivate_active_sort(&mut commands, &active_sort_state);
            }
            SortEvent::_MoveSort { sort_entity: _, new_position: _ } => {
                // TODO: Implement sort moving
            }
            SortEvent::_DeleteSort { sort_entity: _ } => {
                // TODO: Implement sort deletion
            }
        }
    }
}

/// Create a new sort and add it to the world
fn create_sort(
    commands: &mut Commands,
    glyph_name: String,
    position: Vec2,
    advance_width: f32,
) {
    let sort = Sort::new(glyph_name.clone(), position, advance_width);

    info!(
        "Creating sort '{}' at position ({:.1}, {:.1})",
        glyph_name, position.x, position.y
    );

    // Spawn the sort entity as inactive by default
    commands.spawn((
        sort,
        InactiveSort,
        Transform::from_translation(position.extend(0.0)),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Selectable, // Make the sort entity itself selectable
    ));
}

/// Activate a sort (making it the only active sort)
fn activate_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    entity: Entity,
    active_sorts_query: &Query<Entity, With<ActiveSort>>,
) {
    // Deactivate any currently active sorts
    for active_entity in active_sorts_query.iter() {
        commands.entity(active_entity).remove::<ActiveSort>();
        commands.entity(active_entity).insert(InactiveSort);
    }

    // Activate the new sort
    commands.entity(entity).remove::<InactiveSort>();
    commands.entity(entity).insert(ActiveSort);
    active_sort_state.active_sort_entity = Some(entity);

    info!("Activated sort entity {:?}", entity);
}

/// Deactivate the currently active sort
fn deactivate_active_sort(
    commands: &mut Commands,
    active_sort_state: &ActiveSortState,
) {
    if let Some(current_entity) = active_sort_state.active_sort_entity {
        commands.entity(current_entity).remove::<ActiveSort>();
        commands.entity(current_entity).insert(InactiveSort);
        info!("Deactivated sort entity {:?}", current_entity);
    }
}

/// Move a sort to a new position
fn move_sort(commands: &mut Commands, sort_entity: Entity, new_position: Vec2) {
    commands
        .entity(sort_entity)
        .insert(Transform::from_translation(new_position.extend(0.0)));

    // Note: This will be handled by a separate system that syncs Transform with Sort.position
}

/// Delete a sort
fn delete_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    sort_entity: Entity,
) {
    // If this was the active sort, clear the active state
    if active_sort_state.active_sort_entity == Some(sort_entity) {
        active_sort_state.active_sort_entity = None;
    }

    commands.entity(sort_entity).despawn();

    info!("Deleted sort entity {:?}", sort_entity);
}

/// Sync sort position with transform changes
pub fn sync_sort_transforms(mut sorts_query: Query<(&mut Sort, &Transform), Changed<Transform>>) {
    for (mut sort, transform) in sorts_query.iter_mut() {
        let new_position = transform.translation.truncate();
        if sort.position != new_position {
            sort.position = new_position;
        }
    }
}

/// Ensure only one sort is active at a time
pub fn enforce_single_active_sort(
    mut commands: Commands,
    active_sorts: Query<Entity, With<ActiveSort>>,
) {
    let active_count = active_sorts.iter().count();
    if active_count > 1 {
        warn!("Multiple active sorts detected ({}), deactivating extras", active_count);
        
        // Keep the first one, deactivate the rest
        for (index, entity) in active_sorts.iter().enumerate() {
            if index > 0 {
                commands.entity(entity).remove::<ActiveSort>().insert(InactiveSort);
            }
        }
    }
}

pub fn handle_glyph_navigation_changes(
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    mut sorts_query: Query<(Entity, &mut Sort), With<ActiveSort>>,
) {
    if !glyph_navigation.is_changed() {
        return;
    }

    // Only proceed if we have glyph data
    let Some(current_glyph) = &glyph_navigation.current_glyph else {
        return;
    };

    // Update all active sorts to use this glyph
    for (_entity, mut sort) in sorts_query.iter_mut() {
        if sort.glyph_name != *current_glyph {
            info!(
                "Switching active sort from '{}' to '{}'",
                sort.glyph_name, current_glyph
            );
            
            // Get advance width from the font for the new glyph
            let advance_width = if let Some(glyph_data) = app_state.workspace.font.get_glyph(current_glyph) {
                glyph_data.advance_width as f32
            } else {
                600.0 // Default fallback
            };

            sort.glyph_name = current_glyph.clone();
            sort.advance_width = advance_width;
        }
    }
}

/// System to spawn initial sorts when font is loaded - matches backup approach
pub fn spawn_initial_sort(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
    app_state: Res<AppState>,
    _glyph_navigation: Res<GlyphNavigation>,
) {
    // If there are already sorts, don't spawn more
    if !sorts_query.is_empty() {
        return;
    }

    // Check if we have a valid font loaded
    if app_state.workspace.font.glyphs.is_empty() {
        return;
    }

    const GLYPHS_PER_ROW: usize = 16;
    const SORT_SPACING: f32 = 800.0;
    const START_X: f32 = 100.0;
    const START_Y: f32 = 100.0;

    // Create some initial sorts - grab first few glyphs
    for (index, glyph_name) in app_state.workspace.font.glyphs.keys().take(20).enumerate() {
        let x = START_X + (index % GLYPHS_PER_ROW) as f32 * SORT_SPACING;
        let y = START_Y - (index / GLYPHS_PER_ROW) as f32 * SORT_SPACING;
        let position = Vec2::new(x, y);

        sort_events.send(SortEvent::CreateSort {
            glyph_name: glyph_name.clone(),
            position,
        });
    }
}

/// Auto-activate the first sort
pub fn auto_activate_first_sort(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // Only activate if no sorts are currently active
    if !active_sorts_query.is_empty() {
        return;
    }

    // Only activate if we have inactive sorts
    if let Some(first_sort_entity) = sorts_query.iter().next() {
        info!("Auto-activating first sort: {:?}", first_sort_entity);
        
        commands
            .entity(first_sort_entity)
            .remove::<InactiveSort>()
            .insert(ActiveSort);
            
        active_sort_state.active_sort_entity = Some(first_sort_entity);
    }
}

/// System to generate sorts from Unicode input
pub fn generate_sorts_from_unicode_input(
    mut sort_events: EventWriter<SortEvent>,
    glyph_navigation: Res<GlyphNavigation>,
    _app_state: Res<AppState>,
) {
    let Some(current_glyph) = &glyph_navigation.current_glyph else {
        return;
    };

    // Generate a sort for the current glyph at origin
    let position = Vec2::ZERO;
    
    sort_events.send(SortEvent::CreateSort {
        glyph_name: current_glyph.clone(),
        position,
    });
}

/// System to create sorts at startup with some default glyphs
pub fn create_startup_sorts(
    mut sort_events: EventWriter<SortEvent>,
    app_state: Res<AppState>,
) {
    let test_glyphs = vec!["A", "B", "C", "a", "b", "c"];
    
    for (i, glyph_name) in test_glyphs.iter().enumerate() {
        // Only create if glyph exists in font
        if app_state.workspace.font.get_glyph(glyph_name).is_some() {
            let position = Vec2::new(i as f32 * 800.0, 0.0);
            sort_events.send(SortEvent::CreateSort {
                glyph_name: glyph_name.to_string(),
                position,
            });
        }
    }
}

pub fn spawn_sort_point_entities(
    mut commands: Commands,
    added_active_sorts: Query<(Entity, &Sort), Added<ActiveSort>>,
    mut removed_active_sorts: RemovedComponents<ActiveSort>,
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut _selection_state: ResMut<SelectionState>,
    _app_state: Res<AppState>,
) {
    // Spawn point entities for newly activated sorts
    for (sort_entity, sort) in added_active_sorts.iter() {
        info!("Spawning point entities for activated sort: {}", sort.glyph_name);
        // TODO: Implement point entity spawning when selection system is ready
    }

    // Despawn point entities for deactivated sorts
    for sort_entity in removed_active_sorts.read() {
        info!("Despawning point entities for deactivated sort: {:?}", sort_entity);
        for (entity, sort_point_entity) in sort_point_entities.iter() {
            if sort_point_entity.sort_entity == sort_entity {
                commands.entity(entity).despawn();
            }
        }
    }
}

// Stub implementations for now - we can implement these as needed
pub fn respawn_sort_points_on_glyph_change() {}
pub fn update_sort_glyph_data() {}
pub fn manage_sort_crosshairs() {}
pub fn update_sort_from_crosshair_move() {}
pub fn render_sort_crosshairs() {}
pub fn manage_newly_spawned_crosshairs() {}
pub fn sync_crosshair_to_sort_move() {}
pub fn sync_points_to_sort_move() {}
pub fn debug_sort_point_entities() {}

pub fn make_first_sort_active_system(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
) {
    // Only activate if no sort is currently active
    if active_sort_state.active_sort_entity.is_none() {
        if let Some(first_sort_entity) = sorts_query.iter().next() {
            commands
                .entity(first_sort_entity)
                .remove::<InactiveSort>()
                .insert(ActiveSort);
              
            active_sort_state.active_sort_entity = Some(first_sort_entity);
        }
    }
} 