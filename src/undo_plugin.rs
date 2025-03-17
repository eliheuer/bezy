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

    /// Get the current size of the undo stack
    pub fn _stack_size(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the current index in the undo stack
    pub fn _current_index(&self) -> usize {
        self.undo_stack.current_index()
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
    mut transforms: Query<(Entity, &mut Transform)>,
    mut debug_count: Local<usize>,
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

        // Debug info about undo stack state
        let stack_size = undo_state.undo_stack.len();
        let current_index = undo_state.undo_stack.current_index();
        info!(
            "Undo stack state before undo: size={}, current_index={}",
            stack_size, current_index
        );

        if let Some(prev_state) = undo_state.undo_stack.undo() {
            info!("Undoing last action");
            *debug_count += 1; // Track undo count for debugging

            if let Ok(mut session) = edit_sessions.get_single_mut() {
                // Replace the current EditSession with the previous state
                *session = prev_state.as_ref().clone();

                // Now explicitly update all transforms based on the restored positions
                let restored_count =
                    apply_edit_session_to_transforms(&session, &mut transforms);

                info!("Restored EditSession from undo stack (undo #{}) - Updated {} transforms", 
                      *debug_count, restored_count);

                // Debug info about undo stack state after undo
                let new_current_index = undo_state.undo_stack.current_index();
                info!(
                    "Undo stack state after undo: current_index={}",
                    new_current_index
                );
            } else {
                warn!("Could not find EditSession to apply undo");
            }
        } else {
            info!(
                "Nothing to undo - undo stack may be empty or at the beginning"
            );
        }
    }
    // Redo with Command+Shift+Z
    else if keyboard.just_pressed(KeyCode::KeyZ) && shift_pressed {
        info!("Redo shortcut detected (Cmd+Shift+Z)");

        // Debug info about undo stack state
        let stack_size = undo_state.undo_stack.len();
        let current_index = undo_state.undo_stack.current_index();
        info!(
            "Undo stack state before redo: size={}, current_index={}",
            stack_size, current_index
        );

        if let Some(next_state) = undo_state.undo_stack.redo() {
            info!("Redoing previously undone action");

            if let Ok(mut session) = edit_sessions.get_single_mut() {
                // Replace the current EditSession with the next state
                *session = next_state.as_ref().clone();

                // Now explicitly update all transforms
                let restored_count =
                    apply_edit_session_to_transforms(&session, &mut transforms);

                info!("Restored EditSession from redo stack - Updated {} transforms", restored_count);

                // Debug info about undo stack state after redo
                let new_current_index = undo_state.undo_stack.current_index();
                info!(
                    "Undo stack state after redo: current_index={}",
                    new_current_index
                );
            } else {
                warn!("Could not find EditSession to apply redo");
            }
        } else {
            info!("Nothing to redo - may be at the most recent state");
        }
    }
}

/// Helper function to ensure transforms are updated when restoring an EditSession
fn apply_edit_session_to_transforms(
    session: &EditSession,
    transforms: &mut Query<(Entity, &mut Transform)>,
) -> usize {
    let mut count = 0;

    // Loop through all stored positions in the EditSession
    for (entity, position) in &session.point_positions {
        if let Ok((_, mut transform)) = transforms.get_mut(*entity) {
            // Update the transform position
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            count += 1;
        }
    }

    count
}

/// Plugin to set up the undo/redo system
pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize the undo state resource
            .init_resource::<UndoStateResource>()
            // Add the systems
            .add_systems(
                Update,
                (initialize_undo_stack, handle_undo_redo_shortcuts),
            );
    }
}
