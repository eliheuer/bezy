//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

use crate::editing::sort::{Sort, SortEvent, ActiveSort, InactiveSort, ActiveSortState};
use crate::editing::selection::components::{
    Selectable, Selected, PointType, GlyphPointReference, SelectionState
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::core::state::AppState;
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
    _app_state: Res<AppState>,
    _sorts_query: Query<(Entity, &Sort)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort { glyph, position } => {
                create_sort(&mut commands, glyph.clone(), *position);
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
fn create_sort(commands: &mut Commands, glyph: norad::Glyph, position: Vec2) {
    let sort = Sort::new(glyph, position);
    
    info!("Creating sort '{}' at position ({:.1}, {:.1})", 
          sort.glyph.name, position.x, position.y);

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
        if sort.position != new_position {
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
) {
    // Handle newly activated sorts - spawn point entities
    for (sort_entity, sort) in added_active_sorts.iter() {
        info!("Spawning point entities for newly activated sort: {:?}", sort_entity);
        spawn_point_entities_for_sort(&mut commands, sort_entity, sort, &mut selection_state);
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
    _selection_state: &mut SelectionState,
) {
    // Only proceed if the glyph has an outline
    if let Some(outline) = &sort.glyph.outline {
        info!("Sort '{}' has {} contours", sort.glyph.name, outline.contours.len());
        
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
                    sort_entity, contour_idx, point_idx, sort.glyph.name
                );

                info!(
                    "Spawning sort point entity '{}' at ({:.1}, {:.1}) - on_curve: {}",
                    entity_name, point_pos.x, point_pos.y, is_on_curve
                );

                // Spawn the point entity
                let entity_id = commands.spawn((
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
                    PointCoordinates {
                        position: point_pos,
                    },
                    GlyphPointReference {
                        glyph_name: sort.glyph.name.to_string(),
                        contour_index: contour_idx,
                        point_index: point_idx,
                    },
                    SortPointEntity { sort_entity },
                    Name::new(entity_name),
                )).id();

                info!("Spawned sort point entity {:?}", entity_id);
            }
        }
    } else {
        info!("Sort '{}' has no outline", sort.glyph.name);
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

/// System to update sort glyph data when sort points are edited
pub fn update_sort_glyph_data(
    query: Query<
        (&Transform, &GlyphPointReference, &SortPointEntity),
        (With<Selected>, Changed<Transform>),
    >,
    mut sorts_query: Query<&mut Sort>,
) {
    // Early return if no points were modified
    if query.is_empty() {
        return;
    }

    // Process each modified point
    for (transform, point_ref, sort_point) in query.iter() {
        // Get the sort this point belongs to
        if let Ok(mut sort) = sorts_query.get_mut(sort_point.sort_entity) {
            // Capture the sort position before borrowing the outline mutably
            let sort_position = sort.position;
            
            // Get the outline
            if let Some(outline) = sort.glyph.outline.as_mut() {
                // Make sure the contour index is valid
                if point_ref.contour_index < outline.contours.len() {
                    let contour = &mut outline.contours[point_ref.contour_index];

                    // Make sure the point index is valid
                    if point_ref.point_index < contour.points.len() {
                        // Update the point position
                        // Convert world position back to glyph-local position by subtracting sort offset
                        let point = &mut contour.points[point_ref.point_index];
                        let local_pos = Vec2::new(transform.translation.x, transform.translation.y) - sort_position;
                        point.x = local_pos.x;
                        point.y = local_pos.y;

                        debug!(
                            "Updated sort glyph data for point {} in contour {} of sort {:?}",
                            point_ref.point_index, point_ref.contour_index, sort_point.sort_entity
                        );
                    }
                }
            }
        }
    }
} 