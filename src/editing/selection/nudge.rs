use crate::core::settings::{
    CMD_NUDGE_AMOUNT, NUDGE_AMOUNT, SHIFT_NUDGE_AMOUNT,
};
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::Selected;
use crate::editing::sort::ActiveSortState;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;

/// Resource to track nudge state for preventing selection loss during nudging
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct NudgeState {
    /// Whether we're currently nudging (to prevent selection loss)
    pub is_nudging: bool,
    /// Timestamp of the last nudge operation
    pub last_nudge_time: f32,
}

/// System to handle keyboard input for nudging selected points
/// This is the idiomatic Bevy ECS approach: direct system that queries and mutates
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn handle_nudge_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &mut Transform,
                &crate::editing::selection::components::GlyphPointReference,
                Option<&crate::systems::sort_manager::SortPointEntity>,
            ),
            (With<Selected>, With<SortPointEntity>),
        >,
        Query<(&crate::editing::sort::Sort, &Transform)>,
    )>,
    _app_state: ResMut<crate::core::state::AppState>,
    mut event_writer: EventWriter<EditEvent>,
    mut nudge_state: ResMut<NudgeState>,
    time: Res<Time>,
    _active_sort_state: Res<ActiveSortState>, // Keep for potential future use
) {
    // Debug: Log that the system is being called
    debug!(
        "[NUDGE] handle_nudge_input called - selected points: {}",
        queries.p0().iter().count()
    );

    // Debug: Check if any arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];

    let pressed_arrows: Vec<KeyCode> = arrow_keys
        .iter()
        .filter(|&&key| keyboard_input.just_pressed(key))
        .copied()
        .collect();

    if !pressed_arrows.is_empty() {
        debug!("[NUDGE] Arrow keys pressed: {:?}", pressed_arrows);
        debug!(
            "[NUDGE] Selected points count: {}",
            queries.p0().iter().count()
        );
    }

    // Check for arrow key presses
    let nudge_amount = if keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight)
    {
        SHIFT_NUDGE_AMOUNT
    } else if keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight)
    {
        CMD_NUDGE_AMOUNT
    } else {
        NUDGE_AMOUNT
    };

    let mut nudge_direction = Vec2::ZERO;

    // Check each arrow key
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        nudge_direction.x = -nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        nudge_direction.x = nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        nudge_direction.y = nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        nudge_direction.y = -nudge_amount;
    }

    // If we have a nudge direction, apply it to all selected points
    if nudge_direction != Vec2::ZERO {
        let selected_count = queries.p0().iter().count();
        if selected_count > 0 {
            debug!(
                "[NUDGE] Nudging {} selected points by {:?}",
                selected_count, nudge_direction
            );

            debug!("[NUDGE] Setting is_nudging = true");
            nudge_state.is_nudging = true;
            nudge_state.last_nudge_time = time.elapsed_secs();

            // SIMPLIFIED APPROACH: Just update transforms immediately
            // The system ordering should ensure rendering happens after this

            for (entity, mut transform, _point_ref, _sort_point_entity_opt) in
                queries.p0().iter_mut()
            {
                let old_pos = transform.translation.truncate();
                let new_pos = old_pos + nudge_direction;

                debug!("[NUDGE] Moving point {:?} from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       entity, old_pos.x, old_pos.y, new_pos.x, new_pos.y);

                // Update transform immediately - the regular sync system will update glyph data
                transform.translation.x = new_pos.x;
                transform.translation.y = new_pos.y;
            }

            // Create an edit event for undo/redo
            event_writer.write(EditEvent {
                edit_type: EditType::NudgeLeft, // Use an existing variant
            });
        } else {
            debug!("[NUDGE] Arrow key pressed but no selected points found");
        }
    } else {
        // If nudging was active but no keys are pressed, sync immediately and reset state
        if nudge_state.is_nudging {
            debug!("[NUDGE] Keys released, syncing immediately and resetting nudge state");
            nudge_state.is_nudging = false;
        }
    }
}

/// System to reset nudge state after a short delay
pub fn reset_nudge_state(mut nudge_state: ResMut<NudgeState>, time: Res<Time>) {
    if nudge_state.is_nudging
        && time.elapsed_secs() - nudge_state.last_nudge_time > 0.5
    {
        debug!("[NUDGE] Resetting nudge state after timeout");
        nudge_state.is_nudging = false;
    }
}

/// System to sync nudged points back to font data when nudging completes
#[allow(clippy::type_complexity)]
pub fn sync_nudged_points_on_completion(
    nudge_state: Res<NudgeState>,
    query: Query<
        (
            &Transform,
            &crate::editing::selection::components::GlyphPointReference,
            Option<&SortPointEntity>,
        ),
        With<Selected>,
    >,
    sort_query: Query<(&crate::editing::sort::Sort, &Transform)>,
    mut app_state: ResMut<crate::core::state::AppState>,
    mut last_nudge_state: Local<bool>,
) {
    // Only sync when transitioning from nudging to not nudging
    if *last_nudge_state && !nudge_state.is_nudging {
        info!("[NUDGE] Nudging completed, syncing points to font data");

        let app_state = app_state.bypass_change_detection();
        let mut sync_count = 0;

        for (transform, point_ref, sort_point_entity_opt) in query.iter() {
            // Calculate relative position from sort entity
            let (relative_x, relative_y) =
                if let Some(sort_point_entity) = sort_point_entity_opt {
                    if let Ok((_sort, sort_transform)) =
                        sort_query.get(sort_point_entity.sort_entity)
                    {
                        let world_pos = transform.translation.truncate();
                        let sort_pos = sort_transform.translation.truncate();
                        let rel = world_pos - sort_pos;
                        (rel.x as f64, rel.y as f64)
                    } else {
                        (
                            transform.translation.x as f64,
                            transform.translation.y as f64,
                        )
                    }
                } else {
                    (
                        transform.translation.x as f64,
                        transform.translation.y as f64,
                    )
                };

            let updated = app_state.set_point_position(
                &point_ref.glyph_name,
                point_ref.contour_index,
                point_ref.point_index,
                relative_x,
                relative_y,
            );

            if updated {
                sync_count += 1;
                debug!(
                    "[NUDGE] Synced point: glyph='{}' contour={} point={} pos=({:.1}, {:.1})",
                    point_ref.glyph_name,
                    point_ref.contour_index,
                    point_ref.point_index,
                    relative_x,
                    relative_y
                );
            }
        }

        if sync_count > 0 {
            info!(
                "[NUDGE] Successfully synced {} points to font data",
                sync_count
            );
        }
    }

    *last_nudge_state = nudge_state.is_nudging;
}

/// Plugin for the nudge system
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NudgeState>().add_systems(
            Update,
            (
                handle_nudge_input,
                reset_nudge_state,
                sync_nudged_points_on_completion,
            )
                .chain()
                .before(super::systems::update_glyph_data_from_selection),
        );
    }
}

/// Event for nudge operations
#[derive(Event)]
pub struct EditEvent {
    pub edit_type: EditType,
}

/// Point coordinates component
#[derive(Component, Debug, Clone, Copy)]
pub struct PointCoordinates {
    pub x: f32,
    pub y: f32,
}
