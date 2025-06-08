//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::apply_sort_grid_snap;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};
use bevy::prelude::*;




/// Helper to calculate the desired position of the crosshair.
/// Places it at the lower-left of the sort's metrics box, offset inward by 64 units.
fn get_crosshair_position(sort: &Sort, app_state: &AppState) -> Vec2 {
    let metrics = &app_state.workspace.info.metrics;
    
    // Get the descender (bottom of the metrics box)
    let descender = metrics.descender.unwrap_or(-250.0) as f32;
    
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
                let advance_width = if let Some(default_layer) =
                    app_state.workspace.font.ufo.get_default_layer()
                {
                    if let Some(glyph) = default_layer.get_glyph(glyph_name) {
                        glyph
                            .advance
                            .as_ref()
                            .map(|a| a.width as f32)
                            .unwrap_or(600.0)
                    } else {
                        600.0 // Default fallback
                    }
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
    glyph_name: norad::GlyphName,
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
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    mut sorts_query: Query<(Entity, &mut Sort), With<ActiveSort>>,
) {
    if !glyph_navigation.is_changed() {
        return;
    }

    // Get the current glyph name
    if let Some(current_glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
        // Update the active sort to show the current glyph
        for (_entity, mut sort) in sorts_query.iter_mut() {
            if sort.glyph_name != current_glyph_name {
                info!(
                    "Updating active sort from '{}' to '{}'",
                    sort.glyph_name, current_glyph_name
                );
                sort.glyph_name = current_glyph_name.clone();
                
                // Update advance width from the new glyph
                if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
                    if let Some(glyph) = default_layer.get_glyph(&current_glyph_name) {
                        sort.advance_width = glyph
                            .advance
                            .as_ref()
                            .map(|a| a.width as f32)
                            .unwrap_or(600.0);
                    }
                }
            }
        }
    }
}

/// System to respawn sort points when the glyph changes
pub fn respawn_sort_points_on_glyph_change(
    mut commands: Commands,
    changed_sorts: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
    app_state: Res<AppState>,
    mut local_previous_glyphs: Local<std::collections::HashMap<Entity, String>>,
    newly_spawned_crosshairs: Query<&SortCrosshair, With<NewlySpawnedCrosshair>>,
) {
    for (sort_entity, sort) in changed_sorts.iter() {
        // Skip if this sort has a newly spawned crosshair (to avoid conflicts during initial setup)
        let has_newly_spawned_crosshair = newly_spawned_crosshairs.iter()
            .any(|crosshair| crosshair.sort_entity == sort_entity);
        
        if has_newly_spawned_crosshair {
            debug!("respawn_sort_points_on_glyph_change: Skipping sort {:?} because it has a newly spawned crosshair", sort_entity);
            continue;
        }
        
        let current_glyph_name = sort.glyph_name.to_string();
        let should_respawn = local_previous_glyphs
            .get(&sort_entity)
            .map_or(true, |prev_name| prev_name != &current_glyph_name);

        if should_respawn {
            info!("respawn_sort_points_on_glyph_change: Sort {:?} glyph changed to '{}', respawning point entities", 
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
            debug!("respawn_sort_points_on_glyph_change: Sort {:?} changed but glyph name '{}' is the same, skipping respawn", 
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
    selection_state: &mut ResMut<SelectionState>,
) {
    // Get the glyph from the virtual font
    let glyph = if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        default_layer.get_glyph(&sort.glyph_name)
    } else {
        None
    };

    if let Some(glyph) = glyph {
        if let Some(outline) = &glyph.outline {
            for (contour_idx, contour) in outline.contours.iter().enumerate() {
                for (point_idx, point) in contour.points.iter().enumerate() {
                    let is_on_curve = matches!(point.typ, norad::PointType::Move | norad::PointType::Line | norad::PointType::Curve);
                    
                    // Calculate world position: sort position + point offset
                    let point_pos = sort.position + Vec2::new(point.x, point.y);
                    
                    let entity_name = format!(
                        "SortPoint_{}_{}_{}",
                        sort.glyph_name, contour_idx, point_idx
                    );

                    // Check if this point was previously selected
                    // We can't easily track this across respawns, so we'll start fresh
                    let was_selected = false;

                    let mut entity_cmds = commands.spawn((
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
                        crate::editing::selection::components::PointType { is_on_curve },
                        crate::editing::selection::nudge::PointCoordinates {
                            position: point_pos,
                        },
                        crate::editing::selection::components::GlyphPointReference {
                            glyph_name: glyph.name.to_string(),
                            contour_index: contour_idx,
                            point_index: point_idx,
                        },
                        SortPointEntity { sort_entity },
                        Name::new(entity_name),
                    ));

                    // If the point was selected before, restore selection state
                    if was_selected {
                        let entity = entity_cmds.id();
                        entity_cmds.insert(
                            crate::editing::selection::components::Selected,
                        );
                        selection_state.selected.insert(entity);
                        info!(
                            "Restored selection for point at ({}, {})",
                            point_pos.x, point_pos.y
                        );
                    }
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
    for (entity, sort_point) in sort_point_entities.iter() {
        if sort_point.sort_entity == sort_entity {
            // Remove from selection state if selected
            if selection_state.selected.contains(&entity) {
                selection_state.selected.remove(&entity);
                info!("Removed despawned entity {:?} from selection", entity);
            }
            
            commands.entity(entity).despawn();
            debug!("Despawned point entity {:?} for sort {:?}", entity, sort_entity);
        }
    }
}

/// System to spawn point entities for newly activated sorts
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
    // Handle newly activated sorts - spawn point entities
    for (sort_entity, sort) in added_active_sorts.iter() {
        info!(
            "Spawning point entities for newly activated sort: {:?}",
            sort_entity
        );
        spawn_point_entities_for_sort(
            &mut commands,
            sort_entity,
            sort,
            &app_state,
            &mut selection_state,
        );
    }

    // Handle deactivated sorts - despawn point entities
    for sort_entity in removed_active_sorts.read() {
        info!(
            "Despawning point entities for deactivated sort: {:?}",
            sort_entity
        );
        despawn_point_entities_for_sort(
            &mut commands,
            sort_entity,
            &sort_point_entities,
            &mut selection_state,
        );
    }
}

/// System to update sort glyph data when points are moved
pub fn update_sort_glyph_data(
    query: Query<
        (&Transform, &GlyphPointReference, &SortPointEntity),
        (With<Selected>, Changed<Transform>),
    >,
    sorts_query: Query<&Sort>,
    mut app_state: ResMut<AppState>,
) {
    for (transform, glyph_ref, sort_point) in query.iter() {
        if let Ok(sort) = sorts_query.get(sort_point.sort_entity) {
            let point_world_pos = transform.translation.truncate();
            let point_relative_pos = point_world_pos - sort.position;

            if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer_mut() {
                if let Some(glyph) = default_layer.get_glyph_mut(&sort.glyph_name) {
                    if let Some(outline) = &mut glyph.outline {
                        if let Some(contour) = outline.contours.get_mut(glyph_ref.contour_index) {
                            if let Some(point) = contour.points.get_mut(glyph_ref.point_index) {
                                point.x = point_relative_pos.x;
                                point.y = point_relative_pos.y;
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Sort Crosshair Systems
// =============================================================================

/// Manages the lifecycle of sort crosshairs.
/// Spawns crosshair for newly active sorts and despawns for deactivated ones.
pub fn manage_sort_crosshairs(
    mut commands: Commands,
    added_active: Query<(Entity, &Sort), Added<ActiveSort>>,
    mut removed_active: RemovedComponents<ActiveSort>,
    crosshairs: Query<(Entity, &SortCrosshair)>,
    app_state: Res<AppState>,
) {
    // Spawn crosshair for new active sort
    for (sort_entity, sort) in added_active.iter() {
        info!("Spawning crosshair for active sort {:?}", sort_entity);
        let crosshair_pos = get_crosshair_position(sort, &app_state);
        let transform = Transform::from_translation(crosshair_pos.extend(20.0)); // High Z-value

        let crosshair_entity = commands.spawn((
            SortCrosshair { sort_entity },
            NewlySpawnedCrosshair { frames_remaining: 3 }, // Wait 3 frames before allowing movement
            Selectable,
            PointType { is_on_curve: true }, // For selection rendering
            PointCoordinates { position: crosshair_pos },
            transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new(format!("SortCrosshair_{:?}", sort_entity)),
        )).id();
        
        info!("Spawned crosshair entity {:?} for sort {:?} at position ({:.1}, {:.1})", 
              crosshair_entity, sort_entity, crosshair_pos.x, crosshair_pos.y);
    }

    // Despawn crosshair for deactivated sorts
    for sort_entity in removed_active.read() {
        info!("Despawning crosshair for inactive sort {:?}", sort_entity);
        
        for (crosshair_entity, crosshair) in crosshairs.iter() {
            if crosshair.sort_entity == sort_entity {
                commands.entity(crosshair_entity).despawn();
                info!("Despawned crosshair entity {:?}", crosshair_entity);
            }
        }
    }
}

/// Updates the position of a Sort when its crosshair is moved.
pub fn update_sort_from_crosshair_move(
    mut set: ParamSet<(
        Query<(&Transform, &SortCrosshair), (With<Selected>, Changed<Transform>)>,
        Query<(&mut Transform, &mut Sort)>,
    )>,
    app_state: Res<AppState>,
) {
    let mut moves_to_apply = Vec::new();
    for (crosshair_transform, crosshair) in set.p0().iter() {
        // Calculate what the sort position should be based on crosshair position
        let crosshair_pos = crosshair_transform.translation.truncate();
        debug!("Crosshair moved to ({:.1}, {:.1})", crosshair_pos.x, crosshair_pos.y);
        
        // We need to reverse the offset calculation to get the sort position
        let metrics = &app_state.workspace.info.metrics;
        let descender = metrics.descender.unwrap_or(-250.0) as f32;
        let offset = Vec2::new(64.0, descender + 64.0);
        
        // The sort position is the crosshair position minus the offset
        let sort_pos = crosshair_pos - offset;
        
        // Apply sort grid snapping to the calculated position
        let snapped_sort_pos = apply_sort_grid_snap(sort_pos);
        debug!("Calculated sort position: ({:.1}, {:.1}) -> snapped: ({:.1}, {:.1})", 
               sort_pos.x, sort_pos.y, snapped_sort_pos.x, snapped_sort_pos.y);
        
        moves_to_apply.push((crosshair.sort_entity, snapped_sort_pos));
    }

    if moves_to_apply.is_empty() {
        return;
    }

    for (sort_entity, new_pos) in moves_to_apply {
        if let Ok((mut sort_transform, mut sort)) = set.p1().get_mut(sort_entity) {
            if sort.position.distance_squared(new_pos) > 0.001 {
                debug!("Moving sort {:?} from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       sort_entity, sort.position.x, sort.position.y, new_pos.x, new_pos.y);
                sort.position = new_pos;
                sort_transform.translation = new_pos.extend(0.0);
            }
        }
    }
}

/// Renders the crosshair gizmo for active sorts.
pub fn render_sort_crosshairs(
    mut gizmos: Gizmos,
    query: Query<(&Transform, Has<Selected>), With<SortCrosshair>>,
) {
    for (transform, is_selected) in query.iter() {
        let pos = transform.translation.truncate(); // Use truncate for Vec2
        let size = 20.0;
        let color = if is_selected {
            Color::srgb(1.0, 1.0, 0.0) // Yellow
        } else {
            Color::srgb(0.2, 0.4, 1.0) // Blue
        };
        // Use line_2d for better rendering over 2D elements
        gizmos.line_2d(pos + Vec2::X * size, pos - Vec2::X * size, color);
        gizmos.line_2d(pos + Vec2::Y * size, pos - Vec2::Y * size, color);
    }
}

/// System to manage newly spawned crosshairs and prevent immediate movement
pub fn manage_newly_spawned_crosshairs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut NewlySpawnedCrosshair)>,
) {
    for (entity, mut newly_spawned) in query.iter_mut() {
        if newly_spawned.frames_remaining > 0 {
            newly_spawned.frames_remaining -= 1;
            debug!("Newly spawned crosshair {:?} has {} frames remaining", entity, newly_spawned.frames_remaining);
        } else {
            // Remove the component to allow normal movement
            commands.entity(entity).remove::<NewlySpawnedCrosshair>();
            debug!("Removed NewlySpawnedCrosshair from entity {:?} - now allowing movement", entity);
        }
    }
}

/// When a sort is moved, its crosshair must also be moved.
pub fn sync_crosshair_to_sort_move(
    mut crosshair_query: Query<(&mut Transform, &SortCrosshair), Without<NewlySpawnedCrosshair>>,
    changed_sorts: Query<(Entity, &Sort), Changed<Sort>>,
    app_state: Res<AppState>,
) {
    if changed_sorts.is_empty() {
        return;
    }

    for (sort_entity, sort) in changed_sorts.iter() {
        debug!("sync_crosshair_to_sort_move: Sort {:?} changed, syncing crosshair position", sort_entity);
        for (mut crosshair_transform, crosshair) in crosshair_query.iter_mut() {
            if crosshair.sort_entity == sort_entity {
                let old_pos = crosshair_transform.translation.truncate();
                let new_pos = get_crosshair_position(sort, &app_state);
                crosshair_transform.translation = new_pos.extend(crosshair_transform.translation.z);
                debug!("sync_crosshair_to_sort_move: Moved crosshair from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       old_pos.x, old_pos.y, new_pos.x, new_pos.y);
            }
        }
    }
}



/// When a sort is moved, all of its point entities must also be moved.
pub fn sync_points_to_sort_move(
    mut point_query: Query<(&mut Transform, &SortPointEntity, &GlyphPointReference)>,
    changed_sorts: Query<(Entity, &Sort), Changed<Sort>>,
    app_state: Res<AppState>,
) {
    if changed_sorts.is_empty() {
        return;
    }
    
    let ufo = &app_state.workspace.font.ufo;
    if let Some(layer) = ufo.get_default_layer() {
        for (sort_entity, sort) in changed_sorts.iter() {
            if let Some(glyph) = layer.get_glyph(&sort.glyph_name) {
                if let Some(outline) = &glyph.outline {
                     for (mut point_transform, sort_point, glyph_ref) in point_query.iter_mut() {
                        if sort_point.sort_entity == sort_entity {
                            if let Some(contour) = outline.contours.get(glyph_ref.contour_index) {
                                if let Some(point) = contour.points.get(glyph_ref.point_index) {
                                    let new_pos = sort.position + Vec2::new(point.x, point.y);
                                    point_transform.translation = new_pos.extend(0.0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Initial Sort and Auto-Activation
// =============================================================================

/// System to spawn an initial sort when the font is loaded.
pub fn spawn_initial_sort(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
    app_state: Res<AppState>,
    _glyph_navigation: Res<GlyphNavigation>,
) {
    // Only spawn if there are no sorts yet
    if !sorts_query.is_empty() {
        return;
    }

    // Check if the font is loaded
    if app_state.workspace.font.ufo.layers.is_empty() {
        warn!("Font not loaded yet, cannot spawn initial sort.");
        return;
    }

    let default_layer = app_state.workspace.font.ufo.get_default_layer().unwrap();
    let metrics = &app_state.workspace.info.metrics;

    const GLYPHS_PER_ROW: usize = 32;
    let ascender = metrics.ascender.unwrap_or(800.0) as f32;
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    let row_height = (ascender - descender) * 1.2;

    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut glyph_count_in_row = 0;
    let mut max_row_width: f32 = 0.0;

    // Process all glyphs including 'a'
    let glyphs_to_spawn: Vec<_> = default_layer.iter_contents().collect();

    for glyph in glyphs_to_spawn {
        if glyph_count_in_row >= GLYPHS_PER_ROW {
            current_y -= row_height;
            current_x = 0.0;
            glyph_count_in_row = 0;
        }

        sort_events.send(SortEvent::CreateSort {
            glyph_name: glyph.name.clone(),
            position: Vec2::new(current_x, current_y),
        });

        let advance = glyph.advance.as_ref().map_or(600.0, |a| a.width as f32);
        current_x += advance;
        max_row_width = max_row_width.max(current_x);
        glyph_count_in_row += 1;
    }

    info!("Spawned initial sorts for all glyphs in a grid.");
}

/// System to automatically activate the first sort that is created
pub fn auto_activate_first_sort(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    if active_sorts_query.is_empty() && sorts_query.iter().count() == 1 {
        if let Ok(sort_entity) = sorts_query.get_single() {
            info!("Auto-activating first sort: {:?}", sort_entity);
            
            // Use the same activation logic as manual activation to ensure consistency
            commands
                .entity(sort_entity)
                .remove::<InactiveSort>()
                .insert(ActiveSort);
            active_sort_state.active_sort_entity = Some(sort_entity);
            debug!("auto_activate_first_sort: Set active_sort_entity to {:?}", sort_entity);
        }
    } else {
        if !active_sorts_query.is_empty() {
            debug!("auto_activate_first_sort: Already have {} active sorts", active_sorts_query.iter().count());
        }
        if sorts_query.iter().count() != 1 {
            debug!("auto_activate_first_sort: Have {} total sorts", sorts_query.iter().count());
        }
    }
}

/// Debug system to log sort point entity information
pub fn debug_sort_point_entities(
    sort_point_entities: Query<
        (
            Entity,
            &Transform,
            &GlobalTransform,
            &SortPointEntity,
            &GlyphPointReference,
        ),
        With<Selectable>,
    >,
) {
    if sort_point_entities.is_empty() {
        return;
    }
    // info!("=== Sort Point Entities Debug ===");
    // for (entity, transform, global_transform, sort_point, glyph_ref) in
    //     sort_point_entities.iter().take(1)
    // {
    //     info!(
    //         "Entity {:?}: Local({:.1}, {:.1}) Global({:.1}, {:.1}) Sort:{:?} Glyph:{}",
    //         entity,
    //         transform.translation.x,
    //         transform.translation.y,
    //         global_transform.translation().x,
    //         global_transform.translation().y,
    //         sort_point.sort_entity,
    //         glyph_ref.glyph_name
    //     );
    // }
} 