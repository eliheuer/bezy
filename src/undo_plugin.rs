use bevy::prelude::*;
use std::sync::Arc;

use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::undo::UndoState;

/// Resource that holds the undo/redo stack
#[derive(Resource, Debug)]
pub struct UndoStateResource {
    /// The undo stack containing all edit session states
    undo_stack: UndoState<Arc<EditSession>>,
    /// The last edit type that was processed
    last_edit_type: Option<EditType>,
}

impl Default for UndoStateResource {
    fn default() -> Self {
        // We will initialize with a proper state when an EditSession becomes available
        Self {
            undo_stack: UndoState::new(Arc::new(EditSession::default())),
            last_edit_type: None,
        }
    }
}

impl UndoStateResource {
    /// Get the last edit type
    pub fn last_edit_type(&self) -> Option<EditType> {
        self.last_edit_type
    }

    /// Set the last edit type
    pub fn set_last_edit_type(&mut self, edit_type: EditType) {
        self.last_edit_type = Some(edit_type);
    }

    /// Push a new state onto the undo stack
    pub fn push_undo_state(&mut self, state: Arc<EditSession>) {
        self.undo_stack.push(state);
    }

    /// Update the current undo state
    pub fn update_current_undo(&mut self, state: Arc<EditSession>) {
        self.undo_stack.update_current(state);
    }
}

/// System to initialize the undo stack with the first edit session
pub fn initialize_undo_stack(
    mut commands: Commands,
    edit_sessions: Query<&EditSession>,
    undo_state: Option<Res<UndoStateResource>>,
) {
    // Only run if we have an edit session but no undo state yet
    if undo_state.is_some() || edit_sessions.is_empty() {
        return;
    }

    if let Ok(session) = edit_sessions.get_single() {
        info!("Initializing undo stack with initial edit session");
        let undo_resource = UndoStateResource {
            undo_stack: UndoState::new(Arc::new(session.clone())),
            last_edit_type: None,
        };
        commands.insert_resource(undo_resource);
    }
}

/// System to handle undo/redo keyboard shortcuts
pub fn handle_undo_redo_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    undo_state: Option<ResMut<UndoStateResource>>,
    mut edit_sessions: Query<&mut EditSession>,
) {
    let Some(mut undo_state) = undo_state else {
        return;
    };

    // Check for Command/Control key
    let modifier_pressed = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if !modifier_pressed {
        return;
    }

    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    // Undo with Command+Z
    if keyboard.just_pressed(KeyCode::KeyZ) && !shift_pressed {
        info!("Undo shortcut detected (Cmd+Z)");
        if let Some(prev_state) = undo_state.undo_stack.undo() {
            info!("Undoing last action");

            if let Ok(mut session) = edit_sessions.get_single_mut() {
                *session = prev_state.as_ref().clone();
                info!("Restored EditSession from undo stack");
            } else {
                warn!("Could not find EditSession to apply undo");
            }
        } else {
            info!("Nothing to undo");
        }
    }
    // Redo with Command+Shift+Z
    else if keyboard.just_pressed(KeyCode::KeyZ) && shift_pressed {
        info!("Redo shortcut detected (Cmd+Shift+Z)");
        if let Some(next_state) = undo_state.undo_stack.redo() {
            info!("Redoing previously undone action");

            if let Ok(mut session) = edit_sessions.get_single_mut() {
                *session = next_state.as_ref().clone();
                info!("Restored EditSession from redo stack");
            } else {
                warn!("Could not find EditSession to apply redo");
            }
        } else {
            info!("Nothing to redo");
        }
    }
}

/// Plugin to set up the undo/redo system
pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the systems
            .add_systems(
                Update,
                (initialize_undo_stack, handle_undo_redo_shortcuts),
            );
    }
}
