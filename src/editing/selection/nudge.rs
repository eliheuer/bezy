use bevy::prelude::*;
use crate::core::settings::{CMD_NUDGE_AMOUNT, NUDGE_AMOUNT, SHIFT_NUDGE_AMOUNT};
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::Selected;
use crate::systems::sort_manager::SortPointEntity;
use crate::editing::sort::ActiveSortState;

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
pub fn handle_nudge_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (Entity, &mut Transform),
        (With<Selected>, With<SortPointEntity>),
    >,
    mut event_writer: EventWriter<EditEvent>,
    mut nudge_state: ResMut<NudgeState>,
    time: Res<Time>,
    _active_sort_state: Res<ActiveSortState>, // Keep for potential future use
) {
    // Debug: Log that the system is being called
    debug!("[NUDGE] handle_nudge_input called - selected points: {}", query.iter().count());
    
    // Debug: Check if any arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];
    
    let pressed_arrows: Vec<KeyCode> = arrow_keys.iter()
        .filter(|&&key| keyboard_input.just_pressed(key))
        .copied()
        .collect();
    
    if !pressed_arrows.is_empty() {
        debug!("[NUDGE] Arrow keys pressed: {:?}", pressed_arrows);
        debug!("[NUDGE] Selected points count: {}", query.iter().count());
    }

    // Check for arrow key presses
    let nudge_amount = if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        SHIFT_NUDGE_AMOUNT
    } else if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) ||
              keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight) {
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
        let selected_count = query.iter().count();
        if selected_count > 0 {
            debug!("[NUDGE] Nudging {} selected points by {:?}", selected_count, nudge_direction);
            
            nudge_state.is_nudging = true;
            nudge_state.last_nudge_time = time.elapsed_secs();

            for (entity, mut transform) in query.iter_mut() {
                debug!("[NUDGE] Moving point {:?} from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       entity, transform.translation.x, transform.translation.y, 
                       transform.translation.x + nudge_direction.x, transform.translation.y + nudge_direction.y);
                
                // Update the transform position
                transform.translation.x += nudge_direction.x;
                transform.translation.y += nudge_direction.y;
            }

            // Create an edit event for undo/redo
            event_writer.write(EditEvent {
                edit_type: EditType::NudgeLeft, // Use an existing variant
            });
        } else {
            debug!("[NUDGE] Arrow key pressed but no selected points found");
        }
    } else {
        // Reset nudge state if no keys are pressed
        nudge_state.is_nudging = false;
    }
}

/// System to reset nudge state after a short delay
pub fn reset_nudge_state(
    mut nudge_state: ResMut<NudgeState>,
    time: Res<Time>,
) {
    if nudge_state.is_nudging && time.elapsed_secs() - nudge_state.last_nudge_time > 0.1 {
        nudge_state.is_nudging = false;
    }
}

/// Plugin for the nudge system
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NudgeState>()
            .add_systems(Update, (
                handle_nudge_input,
                reset_nudge_state,
            ));
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