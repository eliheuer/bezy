//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

#![allow(deprecated)]

use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::editing::sort::{
    ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent,
};
#[allow(unused_imports)]
use crate::rendering::cameras::DesignCamera;

#[allow(unused_imports)]
use crate::ui::theme::{SORT_HORIZONTAL_PADDING, SORT_VERTICAL_PADDING};
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

/// Helper to calculate the desired position of the crosshair.
/// Places it at the lower-left of the sort's metrics box, offset inward by 64 units.
#[allow(dead_code)]
fn get_crosshair_position(
    _sort: &Sort,
    transform: &Transform,
    app_state: &AppState,
) -> Vec2 {
    let metrics = &app_state.workspace.info.metrics;

    // Get the descender (bottom of the metrics box)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;

    // Left edge is at x=0 for the sort's origin
    let left_edge = 0.0;

    // Position at lower-left corner, offset inward by 64 units
    let offset = Vec2::new(left_edge + 64.0, descender + 64.0);

    transform.translation.truncate() + offset
}

/// Component to mark point entities that belong to a sort
#[derive(Component, Debug, Copy, Clone)]
pub struct SortPointEntity {
    /// The sort entity this point belongs to
    pub sort_entity: Entity,
}

/// Component to mark crosshair entities that can move a sort
#[derive(Component, Debug)]
pub struct SortCrosshair {
    /// The sort entity this crosshair controls
    #[allow(dead_code)]
    pub sort_entity: Entity,
}

/// Component to mark crosshairs that were just spawned and shouldn't be moved yet
#[derive(Component, Debug)]
pub struct NewlySpawnedCrosshair {
    /// Number of frames to wait before allowing movement
    #[allow(dead_code)]
    pub frames_remaining: u32,
}

/// System to handle sort events
pub fn handle_sort_events(
    mut commands: Commands,
    mut sort_events: EventReader<SortEvent>,
    mut active_sort_state: ResMut<ActiveSortState>,
    app_state: Res<AppState>,
    _sorts_query: Query<&Sort>,
    _active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort {
                glyph_name,
                position,
                layout_mode,
            } => {
                info!("Handling CreateSort event for '{}' at {:?} with layout mode {:?}", glyph_name, position, layout_mode);

                // Get advance width from the virtual font
                let advance_width = if let Some(glyph_data) =
                    app_state.workspace.font.get_glyph(glyph_name)
                {
                    glyph_data.advance_width as f32
                } else {
                    600.0 // Default fallback
                };

                let entity = create_sort(
                    &mut commands,
                    glyph_name.clone(),
                    *position,
                    advance_width,
                    layout_mode.clone(),
                );
                info!(
                    "Created sort entity {:?} for '{}' with layout mode {:?}",
                    entity, glyph_name, layout_mode
                );
            }
            SortEvent::ActivateSort { entity } => {
                // Only activate if it's not already active
                if active_sort_state.active_sort_entity != Some(*entity) {
                    // Deactivate current active sort
                    if let Some(current_active) =
                        active_sort_state.active_sort_entity
                    {
                        commands
                            .entity(current_active)
                            .remove::<ActiveSort>()
                            .insert(InactiveSort);
                    }
                    // Activate the new sort
                    commands
                        .entity(*entity)
                        .remove::<InactiveSort>()
                        .insert(ActiveSort);
                    active_sort_state.active_sort_entity = Some(*entity);
                    info!("Activated sort entity {:?}", entity);
                }
            }
            SortEvent::DeactivateSort { entity } => {
                commands
                    .entity(*entity)
                    .remove::<ActiveSort>()
                    .insert(InactiveSort);
                if active_sort_state.active_sort_entity == Some(*entity) {
                    active_sort_state.active_sort_entity = None;
                }
                info!("Deactivated sort entity {:?}", entity);
            }
            SortEvent::DeleteSort { entity } => {
                delete_sort(&mut commands, &mut active_sort_state, *entity);
            }
        }
    }
}

/// Create a new sort and add it to the world
fn create_sort(
    commands: &mut Commands,
    glyph_name: String,
    position: Vec2,
    _advance_width: f32, // Not used in new structure
    layout_mode: crate::core::state::SortLayoutMode,
) -> Entity {
    let sort = Sort {
        glyph_name: glyph_name.clone(),
        layout_mode: layout_mode.clone(),
    };

    info!(
        "Creating sort '{}' at position ({:.1}, {:.1}) with layout mode {:?}",
        glyph_name, position.x, position.y, layout_mode
    );

    // Spawn the sort entity as inactive by default
    commands
        .spawn((
            sort,
            InactiveSort,
            Transform::from_translation(position.extend(0.0)),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Selectable, // Make the sort entity itself selectable
        ))
        .id()
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
    _commands: Commands,
    _changed_sorts: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    _sort_point_entities: Query<(Entity, &SortPointEntity)>,
    // mut selection_state: ResMut<SelectionState>,
    _app_state: Res<AppState>,
    mut _local_previous_glyphs: Local<HashMap<Entity, String>>,
    _newly_spawned_crosshairs: Query<
        &SortCrosshair,
        With<NewlySpawnedCrosshair>,
    >,
) {
    // TODO: Re-enable when selection is working
    /*
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
    */
}

/// Spawn point entities for a sort's glyph outline
pub fn spawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort: &Sort,
    transform: &Transform,
    app_state: &AppState,
    selection_state: &mut ResMut<SelectionState>,
) {
    // Get the glyph from the virtual font using our current architecture
    if let Some(glyph_data) =
        app_state.workspace.font.get_glyph(&sort.glyph_name)
    {
        if let Some(outline) = &glyph_data.outline {
            for (contour_idx, contour) in outline.contours.iter().enumerate() {
                for (point_idx, point_data) in contour.points.iter().enumerate()
                {
                    let is_on_curve = matches!(
                        point_data.point_type,
                        crate::core::state::PointTypeData::Move
                            | crate::core::state::PointTypeData::Line
                            | crate::core::state::PointTypeData::Curve
                    );

                    // Calculate world position: sort position + point offset
                    let point_pos = transform.translation.truncate()
                        + Vec2::new(point_data.x as f32, point_data.y as f32);

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
                        PointType { is_on_curve },
                        PointCoordinates {
                            x: point_pos.x,
                            y: point_pos.y,
                        },
                        GlyphPointReference {
                            glyph_name: sort.glyph_name.clone(),
                            contour_index: contour_idx,
                            point_index: point_idx,
                        },
                        SortPointEntity { sort_entity },
                        Name::new(entity_name),
                    ));

                    // If the point was selected before, restore selection state
                    if was_selected {
                        let entity = entity_cmds.id();
                        entity_cmds.insert(Selected);
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
            debug!(
                "Despawned point entity {:?} for sort {:?}",
                entity, sort_entity
            );
        }
    }
}

/// System to spawn/despawn point entities when sorts become active/inactive
pub fn spawn_sort_point_entities(
    mut commands: Commands,
    // Detect when sorts change from inactive to active
    added_active_sorts: Query<(Entity, &Sort, &Transform), Added<ActiveSort>>,
    // Detect when sorts change from active to inactive
    mut removed_active_sorts: RemovedComponents<ActiveSort>,
    // Find existing point entities for sorts
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
    app_state: Res<AppState>,
) {
    // Spawn point entities for newly active sorts
    for (sort_entity, sort, transform) in added_active_sorts.iter() {
        info!(
            "Spawning point entities for newly active sort {:?}",
            sort_entity
        );
        spawn_point_entities_for_sort(
            &mut commands,
            sort_entity,
            sort,
            transform,
            &app_state,
            &mut selection_state,
        );
    }

    // Despawn point entities for sorts that are no longer active
    for sort_entity in removed_active_sorts.read() {
        info!(
            "Despawning point entities for inactive sort {:?}",
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

/// Update glyph data for all sorts
#[allow(dead_code)]
pub fn update_sort_glyph_data() {}
/// Manage crosshair visibility and positioning  
#[allow(dead_code)]
pub fn manage_sort_crosshairs() {}
/// Update sort position when crosshair moves
#[allow(dead_code)]
pub fn update_sort_from_crosshair_move() {}
/// Render crosshairs for active sorts
#[allow(dead_code)]
pub fn render_sort_crosshairs() {}
/// Handle newly spawned crosshairs
#[allow(dead_code)]
pub fn manage_newly_spawned_crosshairs() {}
/// Sync crosshair position when sort moves
#[allow(dead_code)]
pub fn sync_crosshair_to_sort_move() {}
/// Sync point positions when sort moves
#[allow(dead_code)]
pub fn sync_points_to_sort_move() {}
/// Debug function for sort point entities
#[allow(dead_code)]
pub fn debug_sort_point_entities() {}

/// System to spawn initial sorts when the font is loaded.
/// This preserves the natural UFO file order like the backup version.
pub fn spawn_initial_sort(
    _commands: Commands,
    _sorts_query: Query<Entity, With<Sort>>,
    _app_state: Res<AppState>,
    _has_run: Local<bool>,
) {
    // DISABLED: Skip creating initial sorts grid to keep design space clean
}

/// System to automatically activate the first sort that is created
pub fn auto_activate_first_sort(
    _commands: Commands,
    _active_sort_state: ResMut<ActiveSortState>,
    _sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    _active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // DISABLED: Skip auto-activation to keep design space clean
}
