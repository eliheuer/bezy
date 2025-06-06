//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

use crate::editing::sort::{Sort, SortEvent, ActiveSort, InactiveSort, ActiveSortState};
use crate::editing::selection::components::{
    Selectable, Selected, PointType, GlyphPointReference, SelectionState
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::core::state::{AppState, GlyphNavigation};
use bevy::prelude::*;

/// Component to mark point entities that belong to a sort
#[derive(Component, Debug)]
pub struct SortPointEntity {
    /// The sort entity this point belongs to
    pub sort_entity: Entity,
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
                let advance_width = if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
                    if let Some(glyph) = default_layer.get_glyph(glyph_name) {
                        glyph.advance.as_ref().map(|a| a.width as f32).unwrap_or(600.0)
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
            SortEvent::MoveSort { sort_entity, new_position } => {
                move_sort(&mut commands, *sort_entity, *new_position);
            }
            SortEvent::DeleteSort { sort_entity } => {
                delete_sort(&mut commands, &mut active_sort_state, *sort_entity);
            }
        }
    }
}

/// Create a new sort and add it to the world
fn create_sort(commands: &mut Commands, glyph_name: norad::GlyphName, position: Vec2, advance_width: f32) {
    let sort = Sort::new(glyph_name.clone(), position, advance_width);
    
    info!("Creating sort '{}' at position ({:.1}, {:.1})", 
          glyph_name, position.x, position.y);

    // Spawn the sort entity as inactive by default
    commands.spawn((
        sort,
        InactiveSort,
        Transform::from_translation(position.extend(0.0)),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
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
        commands.entity(active_entity)
            .remove::<ActiveSort>()
            .insert(InactiveSort);
    }

    // Activate the new sort
    commands.entity(sort_entity)
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
        commands.entity(active_entity)
            .remove::<ActiveSort>()
            .insert(InactiveSort);
        
        info!("Deactivated sort entity {:?}", active_entity);
    }

    active_sort_state.active_sort_entity = None;
}

/// Move a sort to a new position
fn move_sort(commands: &mut Commands, sort_entity: Entity, new_position: Vec2) {
    commands.entity(sort_entity).insert(Transform::from_translation(new_position.extend(0.0)));
    
    // Update the sort component's position as well
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
pub fn sync_sort_transforms(
    mut sorts_query: Query<(&mut Sort, &Transform), Changed<Transform>>,
) {
    for (mut sort, transform) in sorts_query.iter_mut() {
        let new_position = transform.translation.truncate();
        // Only update if the position actually changed to avoid triggering Changed<Sort>
        if (sort.position - new_position).length() > f32::EPSILON {
            sort.position = new_position;
        }
    }
}

/// System to ensure only one sort is active at a time
pub fn enforce_single_active_sort(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    let active_sorts: Vec<Entity> = active_sorts_query.iter().collect();
    
    match active_sorts.len() {
        0 => {
            // No active sorts, clear the state
            active_sort_state.active_sort_entity = None;
        }
        1 => {
            // Exactly one active sort, update the state
            active_sort_state.active_sort_entity = Some(active_sorts[0]);
        }
        _ => {
            // Multiple active sorts - this shouldn't happen, fix it
            warn!("Multiple active sorts detected, deactivating all but the first");
            
            for &entity in &active_sorts[1..] {
                commands.entity(entity)
                    .remove::<ActiveSort>()
                    .insert(InactiveSort);
            }
            
            active_sort_state.active_sort_entity = Some(active_sorts[0]);
        }
    }
}

/// System to spawn point entities for active sorts
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
        info!("Spawning point entities for newly activated sort: {:?}", sort_entity);
        spawn_point_entities_for_sort(&mut commands, sort_entity, sort, &app_state, &mut selection_state);
    }

    // Handle deactivated sorts - despawn point entities
    for sort_entity in removed_active_sorts.read() {
        info!("Despawning point entities for deactivated sort: {:?}", sort_entity);
        despawn_point_entities_for_sort(&mut commands, sort_entity, &sort_point_entities, &mut selection_state);
    }
}

/// Spawn point entities for a sort's glyph outline
fn spawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort: &Sort,
    app_state: &AppState,
    _selection_state: &mut SelectionState,
) {
    // Get the glyph from the virtual font
    if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        if let Some(glyph) = default_layer.get_glyph(&sort.glyph_name) {
            // Only proceed if the glyph has an outline
            if let Some(outline) = &glyph.outline {
                info!("Sort '{}' has {} contours", sort.glyph_name, outline.contours.len());
                
                // Iterate through all contours
                for (contour_idx, contour) in outline.contours.iter().enumerate() {
                    if contour.points.is_empty() {
                        continue;
                    }

                    info!("Contour {} has {} points", contour_idx, contour.points.len());

                    // Spawn entities for each point
                    for (point_idx, point) in contour.points.iter().enumerate() {
                        // Apply the sort's position offset to the point
                        let point_pos = sort.position + Vec2::new(point.x as f32, point.y as f32);

                        // Determine if point is on-curve or off-curve
                        let is_on_curve = match point.typ {
                            norad::PointType::Move
                            | norad::PointType::Line
                            | norad::PointType::Curve => true,
                            _ => false,
                        };

                        // Use a unique name for the point entity
                        let entity_name = format!(
                            "SortPoint_{:?}_{}_{}_{}",
                            sort_entity, contour_idx, point_idx, sort.glyph_name
                        );

                        info!(
                            "Spawning sort point entity '{}' at ({:.1}, {:.1}) - on_curve: {}",
                            entity_name, point_pos.x, point_pos.y, is_on_curve
                        );

                        // Create the transform with proper positioning
                        let transform = Transform::from_translation(Vec3::new(
                            point_pos.x,
                            point_pos.y,
                            0.0,
                        ));

                        // Spawn the point entity with all required components
                        let entity_id = commands.spawn((
                            transform,
                            GlobalTransform::from(transform), // Explicitly set GlobalTransform
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Selectable,
                            PointType { is_on_curve },
                            PointCoordinates {
                                position: point_pos,
                            },
                            GlyphPointReference {
                                glyph_name: sort.glyph_name.to_string(),
                                contour_index: contour_idx,
                                point_index: point_idx,
                            },
                            SortPointEntity { sort_entity },
                            Name::new(entity_name),
                        )).id();

                        info!("Spawned sort point entity {:?} at world position ({:.1}, {:.1})", 
                              entity_id, point_pos.x, point_pos.y);
                    }
                }
            } else {
                info!("Sort '{}' has no outline", sort.glyph_name);
            }
        } else {
            warn!("Glyph '{}' not found in virtual font", sort.glyph_name);
        }
    } else {
        warn!("No default layer found in virtual font");
    }
}

/// Despawn point entities for a sort
fn despawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort_point_entities: &Query<(Entity, &SortPointEntity)>,
    selection_state: &mut SelectionState,
) {
    // Find and despawn all point entities belonging to this sort
    for (point_entity, sort_point) in sort_point_entities.iter() {
        if sort_point.sort_entity == sort_entity {
            // Remove from selection state if selected
            selection_state.selected.remove(&point_entity);
            
            // Despawn the entity
            commands.entity(point_entity).despawn();
        }
    }
}

/// System to update virtual font glyph data when sort points are edited
pub fn update_sort_glyph_data(
    query: Query<
        (&Transform, &GlyphPointReference, &SortPointEntity),
        (With<Selected>, Changed<Transform>),
    >,
    sorts_query: Query<&Sort>,
    mut app_state: ResMut<AppState>,
) {
    // Early return if no points were modified
    if query.is_empty() {
        return;
    }

    // Process each modified point
    for (transform, point_ref, sort_point) in query.iter() {
        // Get the sort this point belongs to
        if let Ok(sort) = sorts_query.get(sort_point.sort_entity) {
            // Get the virtual font's glyph data
            if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer_mut() {
                if let Some(glyph) = default_layer.get_glyph_mut(&sort.glyph_name) {
                    if let Some(outline) = glyph.outline.as_mut() {
                        // Make sure the contour index is valid
                        if point_ref.contour_index < outline.contours.len() {
                            let contour = &mut outline.contours[point_ref.contour_index];

                            // Make sure the point index is valid
                            if point_ref.point_index < contour.points.len() {
                                // Update the point position in the virtual font
                                // Convert world position back to glyph-local position by subtracting sort offset
                                let point = &mut contour.points[point_ref.point_index];
                                let local_pos = Vec2::new(transform.translation.x, transform.translation.y) - sort.position;
                                point.x = local_pos.x;
                                point.y = local_pos.y;

                                debug!(
                                    "Updated virtual font glyph '{}' point {} in contour {} from sort {:?}",
                                    sort.glyph_name, point_ref.point_index, point_ref.contour_index, sort_point.sort_entity
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System to spawn an initial sort when the font is loaded (replaces hardcoded default glyph)
pub fn spawn_initial_sort(
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
) {
    // Only create initial sort if:
    // 1. No sorts exist yet
    // 2. Font is loaded (has a path)
    // 3. Font has a default layer
    if !sorts_query.is_empty() {
        return; // Already have sorts
    }
    
    if app_state.workspace.font.path.is_none() {
        return; // Font not loaded yet
    }
    
    // Try to find a glyph to use for the initial sort
    let mut found_glyph_name = None;
    
    if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        // First try to find the glyph using navigation
        if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
            if default_layer.get_glyph(&glyph_name).is_some() {
                found_glyph_name = Some(glyph_name);
                info!("Using glyph '{}' for initial sort", found_glyph_name.as_ref().unwrap());
            }
        }
        
        // If not found, try common glyphs
        if found_glyph_name.is_none() {
            let common_glyphs = ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
            for glyph_name_str in common_glyphs.iter() {
                let name = norad::GlyphName::from(*glyph_name_str);
                if default_layer.get_glyph(&name).is_some() {
                    found_glyph_name = Some(name);
                    info!("Using fallback glyph '{}' for initial sort", glyph_name_str);
                    break;
                }
            }
        }
    }
    
    // Create the initial sort at origin (0, 0) with left baseline
    if let Some(glyph_name) = found_glyph_name {
        sort_events.send(SortEvent::CreateSort {
            glyph_name,
            position: Vec2::ZERO, // Left baseline at origin
        });
        info!("Created initial sort at origin (0, 0)");
    }
}

/// System to automatically activate the first sort when created (for initial sort)
pub fn auto_activate_first_sort(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // Only auto-activate if there are no active sorts and we have exactly one inactive sort
    if active_sorts_query.is_empty() && sorts_query.iter().count() == 1 {
        if let Ok(sort_entity) = sorts_query.get_single() {
            info!("Auto-activating first sort: {:?}", sort_entity);
            
            commands.entity(sort_entity)
                .remove::<InactiveSort>()
                .insert(ActiveSort);
            
            active_sort_state.active_sort_entity = Some(sort_entity);
        }
    }
}

/// System to handle glyph navigation changes and update the active sort
pub fn handle_glyph_navigation_changes(
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    mut active_sorts_query: Query<&mut Sort, With<ActiveSort>>,
) {
    // Only update if glyph navigation has changed
    if !glyph_navigation.is_changed() {
        return;
    }
    
    // Get the active sort
    if let Ok(mut sort) = active_sorts_query.get_single_mut() {
        // Try to find the new glyph
        if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
            let mut found_glyph_name = None;
            
            // First try to find the glyph using navigation
            if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
                if default_layer.get_glyph(&glyph_name).is_some() {
                    found_glyph_name = Some(glyph_name);
                    info!("Updated active sort to glyph '{}'", found_glyph_name.as_ref().unwrap());
                }
            }
            
            // If not found, try common glyphs (fallback)
            if found_glyph_name.is_none() {
                let common_glyphs = ["H", "h", "A", "a", "O", "o", "space", ".notdef"];
                for glyph_name_str in common_glyphs.iter() {
                    let name = norad::GlyphName::from(*glyph_name_str);
                    if default_layer.get_glyph(&name).is_some() {
                        found_glyph_name = Some(name);
                        info!("Updated active sort to fallback glyph '{}'", glyph_name_str);
                        break;
                    }
                }
            }
            
            // Update the sort's glyph name and advance width if we found one
            if let Some(new_glyph_name) = found_glyph_name {
                sort.glyph_name = new_glyph_name.clone();
                
                // Update advance width from the virtual font
                if let Some(glyph) = default_layer.get_glyph(&new_glyph_name) {
                    sort.advance_width = glyph
                        .advance
                        .as_ref()
                        .map(|a| a.width as f32)
                        .unwrap_or(0.0);
                }
            }
        }
    }
}

/// System to respawn sort point entities when the sort's glyph changes
pub fn respawn_sort_points_on_glyph_change(
    mut commands: Commands,
    changed_sorts_query: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
    app_state: Res<AppState>,
    // Track the previous glyph name per sort entity
    mut local_previous_glyphs: Local<std::collections::HashMap<Entity, String>>,
) {
    for (sort_entity, sort) in changed_sorts_query.iter() {
        let current_glyph_name = sort.glyph_name.to_string();
        
        // Only respawn if the glyph name actually changed for this specific sort
        let should_respawn = match local_previous_glyphs.get(&sort_entity) {
            Some(prev_name) => prev_name != &current_glyph_name,
            None => true, // First time seeing this sort, always respawn
        };
        
        if should_respawn {
            info!("Sort {:?} glyph changed to '{}', respawning point entities", 
                  sort_entity, current_glyph_name);
            
            // First despawn existing point entities for this sort
            despawn_point_entities_for_sort(&mut commands, sort_entity, &sort_point_entities, &mut selection_state);
            
            // Then spawn new point entities for the updated glyph
            spawn_point_entities_for_sort(&mut commands, sort_entity, sort, &app_state, &mut selection_state);
            
            // Update the tracked glyph name for this sort
            local_previous_glyphs.insert(sort_entity, current_glyph_name);
        } else {
            debug!("Sort {:?} changed but glyph name unchanged ({}), skipping respawn", 
                   sort_entity, current_glyph_name);
        }
    }
}

/// Debug system to log sort point entity information
pub fn debug_sort_point_entities(
    sort_point_entities: Query<(Entity, &Transform, &GlobalTransform, &SortPointEntity, &GlyphPointReference), With<Selectable>>,
) {
    if !sort_point_entities.is_empty() {
        info!("=== Sort Point Entities Debug ===");
        for (entity, transform, global_transform, sort_point, glyph_ref) in sort_point_entities.iter().take(3) {
            info!(
                "Entity {:?}: Local({:.1}, {:.1}) Global({:.1}, {:.1}) Sort:{:?} Glyph:{}",
                entity,
                transform.translation.x, transform.translation.y,
                global_transform.translation().x, global_transform.translation().y,
                sort_point.sort_entity,
                glyph_ref.glyph_name
            );
        }
        info!("=== End Debug ===");
    }
} 