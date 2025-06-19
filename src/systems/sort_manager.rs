//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, SelectionState,
};
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};
use bevy::prelude::*;
use std::collections::HashMap;

/// Helper to calculate the desired position of the crosshair.
/// Places it at the lower-left of the sort's metrics box, offset inward by 64 units.
fn get_crosshair_position(sort: &Sort, app_state: &AppState) -> Vec2 {
    let metrics = &app_state.workspace.info.metrics();
    
    // Get the descender (bottom of the metrics box)
    let descender = metrics.descender as f32;
    
    // Left edge is at x=0 for the sort's origin
    let left_edge = 0.0;
    
    // Position at lower-left corner, offset inward by 64 units
    let offset = Vec2::new(left_edge + 64.0, descender + 64.0);
    
    sort.position + offset
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
                // Get advance width from the virtual font
                let advance_width = if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
                    glyph_data.advance_width as f32
                } else {
                    600.0 // Default fallback
                };

                create_sort(&mut commands, glyph_name.clone(), *position, advance_width);
            }
            SortEvent::ActivateSort { sort_entity } => {
                activate_sort(
                    &mut commands,
                    &mut active_sort_state,
                    *sort_entity,
                    &active_sorts_query,
                );
            }
            SortEvent::DeactivateSort => {
                deactivate_current_sort(&mut commands, &mut active_sort_state, &active_sorts_query);
            }
            SortEvent::_MoveSort {
                sort_entity,
                new_position,
            } => {
                move_sort(&mut commands, *sort_entity, *new_position);
            }
            SortEvent::_DeleteSort { sort_entity } => {
                delete_sort(&mut commands, &mut active_sort_state, *sort_entity);
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

/// Activate a sort for editing
fn activate_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    sort_entity: Entity,
    active_sorts_query: &Query<Entity, With<ActiveSort>>,
) {
    // First deactivate any currently active sort
    for active_entity in active_sorts_query.iter() {
        commands
            .entity(active_entity)
            .remove::<ActiveSort>()
            .insert(InactiveSort);
    }

    // Activate the new sort
    commands
        .entity(sort_entity)
        .remove::<InactiveSort>()
        .insert(ActiveSort);

    active_sort_state.active_sort_entity = Some(sort_entity);

    info!("Activated sort entity {:?}", sort_entity);
}

/// Deactivate the currently active sort
fn deactivate_current_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    active_sorts_query: &Query<Entity, With<ActiveSort>>,
) {
    for active_entity in active_sorts_query.iter() {
        commands
            .entity(active_entity)
            .remove::<ActiveSort>()
            .insert(InactiveSort);

        info!("Deactivated sort entity {:?}", active_entity);
    }

    active_sort_state.active_sort_entity = None;
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

/// System to sync Transform changes back to Sort position
pub fn sync_sort_transforms(mut sorts_query: Query<(&mut Sort, &Transform), Changed<Transform>>) {
    for (mut sort, transform) in sorts_query.iter_mut() {
        let new_position = transform.translation.truncate();
        // Only update if the position actually changed to avoid triggering Changed<Sort>
        if (sort.position - new_position).length() > f32::EPSILON {
            debug!(
                "sync_sort_transforms: Updating sort position from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                sort.position.x, sort.position.y, new_position.x, new_position.y
            );
            sort.position = new_position;
        }
    }
}

/// System to ensure only one sort is active at a time
pub fn enforce_single_active_sort(
    mut commands: Commands,
    active_sorts: Query<Entity, With<ActiveSort>>,
) {
    let active_count = active_sorts.iter().count();
    if active_count > 1 {
        warn!("Multiple active sorts detected ({}), deactivating all but first", active_count);
        
        // Keep the first one, deactivate the rest
        for (index, entity) in active_sorts.iter().enumerate() {
            if index > 0 {
                commands.entity(entity).remove::<ActiveSort>().insert(InactiveSort);
            }
        }
    }
}

/// System to handle glyph navigation changes
pub fn handle_glyph_navigation_changes(
    _glyph_navigation: Res<GlyphNavigation>,
    _app_state: Res<AppState>,
    _sorts_query: Query<(Entity, &mut Sort), With<ActiveSort>>,
) {
    // TODO: Implement glyph navigation when the navigation system is ported
}

/// System to respawn sort points when the glyph changes
pub fn respawn_sort_points_on_glyph_change(
    mut commands: Commands,
    changed_sorts: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
    app_state: Res<AppState>,
    mut local_previous_glyphs: Local<HashMap<Entity, String>>,
    newly_spawned_crosshairs: Query<&SortCrosshair, With<NewlySpawnedCrosshair>>,
) {
    for (sort_entity, sort) in changed_sorts.iter() {
        // Skip if this sort has a newly spawned crosshair (to avoid conflicts during initial setup)
        let has_newly_spawned_crosshair = newly_spawned_crosshairs.iter()
            .any(|crosshair| crosshair.sort_entity == sort_entity);
        
        if has_newly_spawned_crosshair {
            debug!("Skipping sort {:?} because it has a newly spawned crosshair", sort_entity);
            continue;
        }
        
        let current_glyph_name = sort.glyph_name.to_string();
        let should_respawn = local_previous_glyphs
            .get(&sort_entity)
            .map_or(true, |prev_name| prev_name != &current_glyph_name);

        if should_respawn {
            info!("Sort {:?} glyph changed to '{}', respawning point entities", 
                  sort_entity, current_glyph_name);
            
            // Despawn existing point entities for this sort
            despawn_point_entities_for_sort(
                &mut commands,
                sort_entity,
                &sort_point_entities,
                &mut selection_state,
            );
            
            // Spawn new point entities for the updated sort
            spawn_point_entities_for_sort(
                &mut commands,
                sort_entity,
                sort,
                &app_state,
                &mut selection_state,
            );
            
            local_previous_glyphs.insert(sort_entity, current_glyph_name);
        } else {
            debug!("Sort {:?} changed but glyph name '{}' is the same, skipping respawn", 
                   sort_entity, current_glyph_name);
        }
    }
}

/// Spawn point entities for a sort's glyph outline
fn spawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort: &Sort,
    app_state: &AppState,
    _selection_state: &mut ResMut<SelectionState>,
) {
    // Get the glyph from the virtual font
    let glyph_data = app_state.workspace.font.get_glyph(&sort.glyph_name);

    if let Some(glyph_data) = glyph_data {
        if let Some(outline_data) = &glyph_data.outline {
            for (contour_idx, contour_data) in outline_data.contours.iter().enumerate() {
                for (point_idx, point_data) in contour_data.points.iter().enumerate() {
                    let is_on_curve = matches!(point_data.point_type, crate::core::state::PointTypeData::Move | crate::core::state::PointTypeData::Line | crate::core::state::PointTypeData::Curve);
                    
                    // Calculate world position: sort position + point offset
                    let point_pos = sort.position + Vec2::new(point_data.x as f32, point_data.y as f32);
                    
                    let entity_name = format!(
                        "SortPoint_{}_{}_{}",
                        sort.glyph_name, contour_idx, point_idx
                    );

                    commands.spawn((
                        Transform::from_translation(Vec3::new(
                            point_pos.x,
                            point_pos.y,
                            0.0,
                        )),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Selectable,
                        PointType { is_on_curve },
                        GlyphPointReference {
                            glyph_name: glyph_data.name.clone(),
                            contour_index: contour_idx,
                            point_index: point_idx,
                        },
                        SortPointEntity { sort_entity },
                        Name::new(entity_name),
                    ));
                }
            }
        }
    }
}

/// Despawn point entities for a sort
fn despawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort_point_entities: &Query<(Entity, &SortPointEntity)>,
    selection_state: &mut ResMut<SelectionState>,
) {
    for (entity, sort_point_entity) in sort_point_entities.iter() {
        if sort_point_entity.sort_entity == sort_entity {
            // Remove from selection if selected
            selection_state.selected.remove(&entity);
            
            // Despawn the entity
            commands.entity(entity).despawn();
            debug!("Despawned point entity {:?} for sort {:?}", entity, sort_entity);
        }
    }
}

/// System to spawn/despawn point entities when sorts become active/inactive
pub fn spawn_sort_point_entities(
    mut commands: Commands,
    // Detect when sorts change from inactive to active
    added_active_sorts: Query<(Entity, &Sort), Added<ActiveSort>>,
    // Detect when sorts change from active to inactive
    mut removed_active_sorts: RemovedComponents<ActiveSort>,
    // Find existing point entities for sorts
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
    app_state: Res<AppState>,
) {
    // Spawn point entities for newly active sorts
    for (sort_entity, sort) in added_active_sorts.iter() {
        info!("Spawning point entities for newly active sort {:?}", sort_entity);
        spawn_point_entities_for_sort(
            &mut commands,
            sort_entity,
            sort,
            &app_state,
            &mut selection_state,
        );
    }

    // Despawn point entities for sorts that are no longer active
    for sort_entity in removed_active_sorts.read() {
        info!("Despawning point entities for inactive sort {:?}", sort_entity);
        despawn_point_entities_for_sort(
            &mut commands,
            sort_entity,
            &sort_point_entities,
            &mut selection_state,
        );
    }
}

// Placeholder systems for features not yet implemented
pub fn update_sort_glyph_data() {}
pub fn manage_sort_crosshairs() {}
pub fn update_sort_from_crosshair_move() {}
pub fn render_sort_crosshairs() {}
pub fn manage_newly_spawned_crosshairs() {}
pub fn sync_crosshair_to_sort_move() {}
pub fn sync_points_to_sort_move() {}
pub fn debug_sort_point_entities() {}

/// System to create initial sorts from glyphs
pub fn create_startup_sorts(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
    app_state: Res<AppState>,
) {
    // Only create sorts if none exist
    if sorts_query.is_empty() {
        info!("No sorts found, creating startup sorts from font glyphs");
        
        // Use font metrics for proper spacing
        let font_metrics = app_state.workspace.info.metrics();
        let upm = font_metrics.units_per_em as f32;
        let ascender = font_metrics.ascender as f32;
        let descender = font_metrics.descender as f32;
        let total_height = ascender - descender;
        
        // Calculate spacing based on font metrics for even margins
        let vertical_margin = total_height * 0.3; // 30% margin between rows
        let horizontal_margin = upm * 0.2; // 20% margin between columns
        let row_height = total_height + vertical_margin;
        let col_width = upm + horizontal_margin;
        
        let start_x = 100.0;
        let start_y = 100.0;
        let max_cols = 12; // Reduced to fit better on screen
        
        // Create sorts for all glyphs, arranged in a grid with proper spacing
        for (index, glyph_name) in app_state.workspace.font.glyphs.keys().enumerate() {
            let col = index % max_cols;
            let row = index / max_cols;
            
            let position = Vec2::new(
                start_x + col as f32 * col_width,
                start_y - row as f32 * row_height,
            );
            
            sort_events.write(SortEvent::CreateSort {
                glyph_name: glyph_name.clone(),
                position,
            });
        }
    }
}

/// System to auto-activate the first sort if no sort is active
pub fn make_first_sort_active_system(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // Only activate if no sort is currently active
    if active_sorts_query.is_empty() && !sorts_query.is_empty() {
        if let Some(first_sort_entity) = sorts_query.iter().next() {
            commands
                .entity(first_sort_entity)
                .remove::<InactiveSort>()
                .insert(ActiveSort);
            
            active_sort_state.active_sort_entity = Some(first_sort_entity);
            
            info!("Auto-activating first sort: {:?}", first_sort_entity);
        }
    }
} 