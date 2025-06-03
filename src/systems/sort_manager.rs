//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

use crate::core::sort::{Sort, SortEvent, ActiveSort, InactiveSort, ActiveSortState};
use crate::core::state::AppState;
use bevy::prelude::*;

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