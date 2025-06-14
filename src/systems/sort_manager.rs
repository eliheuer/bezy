//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort is active at a time and manages the state transitions.

use crate::core::state::AppState;
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};
use bevy::prelude::*;
use smol_str::SmolStr;

/// System to handle sort events
pub fn handle_sort_events(
    mut commands: Commands,
    mut sort_events: EventReader<SortEvent>,
    mut active_sort_state: ResMut<ActiveSortState>,
    app_state: NonSend<AppState>,
    _sorts_query: Query<(Entity, &Sort)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort { glyph_name, position } => {
                let advance_width = if let Some(font) = &app_state.font {
                    let default_layer = font.default_layer();
                    if let Some(glyph) = default_layer.get_glyph(glyph_name) {
                        glyph.width as f32
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
                sort_entity: _,
                new_position: _,
            } => {
                // move_sort(&mut commands, *sort_entity, *new_position);
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
    glyph_name: SmolStr,
    position: Vec2,
    advance_width: f32,
) {
    let sort = Sort::new(glyph_name.clone(), position, advance_width);

    info!(
        "Creating sort '{}' at position ({:.1}, {:.1})",
        glyph_name, position.x, position.y
    );

    commands.spawn((
        sort,
        InactiveSort,
        Transform::from_translation(position.extend(0.0)),
        GlobalTransform::default(),
    ));
}

/// Activate a sort for editing
fn activate_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    sort_entity: Entity,
    active_sorts_query: &Query<Entity, With<ActiveSort>>,
) {
    for active_entity in active_sorts_query.iter() {
        commands
            .entity(active_entity)
            .remove::<ActiveSort>()
            .insert(InactiveSort);
    }

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

/// Delete a sort
fn delete_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    sort_entity: Entity,
) {
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
        if (sort.position - new_position).length_squared() > f32::EPSILON {
            sort.position = new_position;
        }
    }
}

/// System to ensure only one sort is active at a time
pub fn enforce_single_active_sort(
    mut commands: Commands,
    active_sorts: Query<Entity, With<ActiveSort>>,
) {
    if active_sorts.iter().len() > 1 {
        let mut to_deactivate = active_sorts.iter().collect::<Vec<_>>();
        let to_keep = to_deactivate.remove(0);
        warn!(
            "Multiple active sorts detected ({}), deactivating all but {:?}",
            to_deactivate.len() + 1,
            to_keep
        );

        for entity in to_deactivate {
            commands
                .entity(entity)
                .remove::<ActiveSort>()
                .insert(InactiveSort);
        }
    }
}

/// System to spawn the initial sort if none exist
pub fn spawn_initial_sort(
    mut sort_events: EventWriter<SortEvent>,
    sorts_query: Query<Entity, With<Sort>>,
    app_state: NonSend<AppState>,
) {
    if sorts_query.is_empty() {
        if let Some(font) = &app_state.font {
            if let Some(glyph_name) = font.default_layer().iter().next().map(|g| g.name().clone()) {
                info!("Spawning initial sort for glyph '{}'", glyph_name.as_str());
                sort_events.write(SortEvent::CreateSort {
                    glyph_name: glyph_name.as_ref().into(),
                    position: Vec2::ZERO,
                });
            }
        }
    }
}

/// System to automatically activate the first sort if no sort is active
pub fn auto_activate_first_sort(
    mut commands: Commands,
    mut active_sort_state: ResMut<ActiveSortState>,
    sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    if active_sorts_query.is_empty() {
        if let Some(entity) = sorts_query.iter().next() {
            info!("Auto-activating first sort: {:?}", entity);
            activate_sort(
                &mut commands,
                &mut active_sort_state,
                entity,
                &active_sorts_query,
            );
        }
    }
} 