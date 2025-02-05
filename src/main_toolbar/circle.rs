use super::EditModeSystem;
use bevy::prelude::*;

pub struct CircleMode;

impl EditModeSystem for CircleMode {
    fn update(&self, _commands: &mut Commands) {
        // Implementation for circle mode update
    }

    fn on_enter(&self) {
        // Called when entering circle mode
    }

    fn on_exit(&self) {
        // Called when exiting circle mode
    }
}
