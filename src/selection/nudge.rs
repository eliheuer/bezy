use crate::selection::components::Selected;
use bevy::prelude::*;

/// The amount to nudge by in each direction (in design units)
const NUDGE_AMOUNT: f32 = 1.0;
/// The amount to nudge when shift is held (for larger movements)
const SHIFT_NUDGE_AMOUNT: f32 = 10.0;
/// The amount to nudge when command/ctrl is held (for even larger movements)
const CMD_NUDGE_AMOUNT: f32 = 100.0;

/// Component to track the last edit type for undo purposes
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct LastEditType {
    pub edit_type: Option<EditType>,
}

/// Event to signal that an edit has been made that should be added to the undo stack
#[derive(Event, Debug, Clone)]
pub struct EditEvent {
    pub edit_type: EditType,
}

/// The type of edit that was made
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EditType {
    /// Added a new point
    AddPoint,
    /// Moved a point
    MovePoint,
    /// Deleted a point
    DeletePoint,
    /// Nudged points left
    NudgeLeft,
    /// Nudged points right
    NudgeRight,
    /// Nudged points up
    NudgeUp,
    /// Nudged points down
    NudgeDown,
}

/// System to handle keyboard input for nudging selected points
pub fn handle_nudge_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Selected>>,
    mut event_writer: EventWriter<EditEvent>,
) {
    // Early return if no arrow key is pressed
    if !keyboard_input.just_pressed(KeyCode::ArrowLeft)
        && !keyboard_input.just_pressed(KeyCode::ArrowRight)
        && !keyboard_input.just_pressed(KeyCode::ArrowUp)
        && !keyboard_input.just_pressed(KeyCode::ArrowDown)
    {
        return;
    }

    // Calculate nudge amount based on modifiers
    let mut amount = NUDGE_AMOUNT;
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);
    let cmd_pressed = keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight)
        || keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    if cmd_pressed {
        amount = CMD_NUDGE_AMOUNT;
    } else if shift_pressed {
        amount = SHIFT_NUDGE_AMOUNT;
    }

    // Determine direction and edit type
    let (direction, edit_type) =
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            (Vec3::new(-amount, 0.0, 0.0), EditType::NudgeLeft)
        } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            (Vec3::new(amount, 0.0, 0.0), EditType::NudgeRight)
        } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            (Vec3::new(0.0, amount, 0.0), EditType::NudgeUp)
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            (Vec3::new(0.0, -amount, 0.0), EditType::NudgeDown)
        } else {
            return;
        };

    // Only proceed if we have selected points to nudge
    let count = query.iter().count();
    if count == 0 {
        return;
    }

    // Apply nudge to all selected entities
    for mut transform in &mut query {
        transform.translation += direction;
    }

    // Log the nudge operation
    info!("Nudged {} points by {:?}", count, direction);

    // Send edit event for undo system
    event_writer.send(EditEvent { edit_type });
}

/// Plugin to set up nudging functionality
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EditEvent>()
            .register_type::<EditType>()
            .register_type::<LastEditType>()
            .add_systems(Update, handle_nudge_shortcuts);
    }
}
