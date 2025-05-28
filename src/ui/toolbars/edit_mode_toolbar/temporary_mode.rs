//! Temporary Mode Switching
//!
//! This module handles temporary mode switching using the spacebar key.
//! When spacebar is held down, the mode switches to Pan mode. When released,
//! it switches back to the previous mode. This provides a better UX than
//! the hybrid approach.

use super::{CurrentEditMode, EditMode};
use bevy::prelude::*;

/// Resource to track temporary mode switching state
#[derive(Resource, Default)]
pub struct TemporaryModeState {
    /// The mode that was active before switching to temporary mode
    pub previous_mode: Option<EditMode>,
    /// Whether we're currently in temporary mode
    pub in_temporary_mode: bool,
}

/// System to handle spacebar for temporary Pan mode switching
///
/// This provides a clean UX where:
/// - Hold spacebar: Switch to Pan mode (shows in toolbar)
/// - Release spacebar: Switch back to previous mode
///
/// This replaces the old hybrid approach that didn't work well
/// with tools like the pen tool.
pub fn handle_temporary_mode_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_mode: ResMut<CurrentEditMode>,
    mut temp_state: ResMut<TemporaryModeState>,
) {
    // Handle spacebar press - switch to temporary Pan mode
    if keyboard.just_pressed(KeyCode::Space) {
        if !temp_state.in_temporary_mode {
            // Store the current mode and switch to Pan
            temp_state.previous_mode = Some(current_mode.0);
            temp_state.in_temporary_mode = true;

            // Exit the current mode
            let old_system = current_mode.0.get_system();
            old_system.on_exit();

            // Switch to Pan mode
            current_mode.0 = EditMode::Pan;
            let new_system = current_mode.0.get_system();
            new_system.on_enter();

            info!("Temporarily switched to Pan mode (spacebar held)");
        }
    }

    // Handle spacebar release - switch back to previous mode
    if keyboard.just_released(KeyCode::Space) {
        if temp_state.in_temporary_mode {
            if let Some(previous_mode) = temp_state.previous_mode {
                // Exit Pan mode
                let old_system = current_mode.0.get_system();
                old_system.on_exit();

                // Switch back to the previous mode
                current_mode.0 = previous_mode;
                let new_system = current_mode.0.get_system();
                new_system.on_enter();

                info!(
                    "Switched back to {:?} mode (spacebar released)",
                    previous_mode
                );
            }

            // Reset temporary mode state
            temp_state.previous_mode = None;
            temp_state.in_temporary_mode = false;
        }
    }
}
