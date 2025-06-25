//! Sort management system
//!
//! Handles the creation, activation, deactivation, and lifecycle of sorts.
//! Ensures only one sort can be active at a time and manages the state transitions.

#![allow(deprecated)]

use crate::core::state::{AppState, GlyphNavigation};
#[allow(unused_imports)]
use crate::core::settings::apply_sort_grid_snap;
#[allow(unused_imports)]
use crate::rendering::cameras::DesignCamera;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};

use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::window::PrimaryWindow;
use std::collections::HashMap;
#[allow(unused_imports)]
use crate::ui::theme::{SORT_VERTICAL_PADDING, SORT_HORIZONTAL_PADDING};

/// Helper to calculate the desired position of the crosshair.
/// Places it at the lower-left of the sort's metrics box, offset inward by 64 units.
#[allow(dead_code)]
fn get_crosshair_position(sort: &Sort, app_state: &AppState) -> Vec2 {
    let metrics = &app_state.workspace.info.metrics;
    
    // Get the descender (bottom of the metrics box)
    let descender = metrics.descender.unwrap_or(-200.0) as f32;
    
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
    sorts_query: Query<&Sort>,
    active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    for event in sort_events.read() {
        match event {
            SortEvent::CreateSort { glyph_name, position } => {
                info!("Handling CreateSort event for '{}' at {:?}", glyph_name, position);
                // Get advance width from the virtual font
                let advance_width = if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
                    glyph_data.advance_width as f32
                } else {
                    600.0 // Default fallback
                };

                let entity = create_sort(&mut commands, glyph_name.clone(), *position, advance_width);
                info!("Created sort entity {:?} for '{}'", entity, glyph_name);
            }
            SortEvent::ActivateSort { sort_entity } => {
                activate_sort(
                    &mut commands,
                    &mut active_sort_state,
                    *sort_entity,
                    &active_sorts_query,
                    &sorts_query,
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
) -> Entity {
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
    )).id()
}

/// Activate a sort for editing
fn activate_sort(
    commands: &mut Commands,
    active_sort_state: &mut ResMut<ActiveSortState>,
    sort_entity: Entity,
    active_sorts_query: &Query<Entity, With<ActiveSort>>,
    _sorts_query: &Query<&Sort>,
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
    _commands: Commands,
    _changed_sorts: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    _sort_point_entities: Query<(Entity, &SortPointEntity)>,
    // mut selection_state: ResMut<SelectionState>,
    _app_state: Res<AppState>,
    mut _local_previous_glyphs: Local<HashMap<Entity, String>>,
    _newly_spawned_crosshairs: Query<&SortCrosshair, With<NewlySpawnedCrosshair>>,
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
fn spawn_point_entities_for_sort(
    commands: &mut Commands,
    sort_entity: Entity,
    sort: &Sort,
    app_state: &AppState,
    selection_state: &mut ResMut<SelectionState>,
) {
    // Get the glyph from the virtual font using our current architecture
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        if let Some(outline) = &glyph_data.outline {
            for (contour_idx, contour) in outline.contours.iter().enumerate() {
                for (point_idx, point_data) in contour.points.iter().enumerate() {
                    let is_on_curve = matches!(
                        point_data.point_type,
                        crate::core::state::PointTypeData::Move | 
                        crate::core::state::PointTypeData::Line | 
                        crate::core::state::PointTypeData::Curve
                    );
                
                // Calculate world position: sort position + point offset
                let point_pos = sort.position + Vec2::new(point_data.x as f32, point_data.y as f32);
                
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
                        position: point_pos,
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
    mut has_run: Local<bool>,
) {
    // DISABLED: Skip creating initial sorts grid to keep design space clean
    // Only create sorts if none exist and we haven't run before
    if *has_run {
        return;
    }
    
    // Set has_run to true to prevent this system from running again
    *has_run = true;
    
    info!("spawn_initial_sort: Skipped creating initial sorts grid for clean workspace");
    
    // Original implementation commented out:
    /*
    if sorts_query.is_empty() {
        info!("No sorts found, creating startup sorts from font glyphs");
        *has_run = true;
        
        // Use font metrics for proper spacing (match backup version exactly)
        let font_metrics = &app_state.workspace.info.metrics;
        let upm = font_metrics.units_per_em as f32;
        let descender = font_metrics.descender.unwrap_or(-(upm as f64 * 0.2)) as f32;
        
        // The backup version uses UPM - descender for metrics box height (not ascender - descender)
        let metrics_box_height = upm - descender;
        
        // Use consistent fixed spacing from theme (like the backup version)
        let vertical_padding = crate::ui::theme::SORT_VERTICAL_PADDING;
        let horizontal_padding = crate::ui::theme::SORT_HORIZONTAL_PADDING;
        let row_height = metrics_box_height + vertical_padding;
        let col_width = upm + horizontal_padding;
        
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let mut glyph_count_in_row = 0;
        const GLYPHS_PER_ROW: usize = 16; // Match the backup version
        
        // Use simple alphabetical order instead of Unicode order to test
        // This should be more predictable and stable
        let mut glyph_names: Vec<_> = app_state.workspace.font.glyphs.keys().collect();
        glyph_names.sort(); // Simple alphabetical sort by glyph name
        
        info!("Creating {} sorts in alphabetical order", glyph_names.len());
        
        // Create sorts for all glyphs, arranged in a grid with proper spacing (match backup logic)
        // Create sorts directly instead of using events to ensure deterministic ordering
        for glyph_name in glyph_names {
            if glyph_count_in_row >= GLYPHS_PER_ROW {
                // Move to the next row, dropping by the full height of the metrics
                // box plus the desired padding (exactly like backup version)
                current_y -= row_height;
                current_x = 0.0;
                glyph_count_in_row = 0;
            }
            
            // Get advance width from glyph data (like backup version)
            let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
                glyph_data.advance_width as f32
            } else {
                600.0 // Default fallback
            };
            
            // Create sort directly instead of using events to ensure deterministic ordering
            let sort = Sort::new(glyph_name.clone(), Vec2::new(current_x, current_y), advance_width);
            
            info!("Creating sort '{}' at position ({:.1}, {:.1})", glyph_name, current_x, current_y);
            
            // Spawn the sort entity as inactive by default
            commands.spawn((
                sort,
                InactiveSort,
                Transform::from_translation(Vec2::new(current_x, current_y).extend(0.0)),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Selectable, // Make the sort entity itself selectable
            ));
            
            current_x += advance_width + horizontal_padding;
            glyph_count_in_row += 1;
        }
    } else {
        let sort_count = sorts_query.iter().count();
        info!("create_startup_sorts: Skipping because sorts_query is not empty (count={})", sort_count);
    }
    */
}

/// System to automatically activate the first sort that is created
pub fn auto_activate_first_sort(
    _commands: Commands,
    _active_sort_state: ResMut<ActiveSortState>,
    _sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    _active_sorts_query: Query<Entity, With<ActiveSort>>,
) {
    // DISABLED: Skip auto-activating sorts to keep design space clean
    // Original implementation commented out:
    /*
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
    */
}



 