use super::EditModeSystem;
use bevy::prelude::*;

pub struct PenMode;

impl EditModeSystem for PenMode {
    fn update(&self, commands: &mut Commands) {
        // Implementation for pen mode update
    }

    fn on_enter(&self) {
        // Called when entering pen mode
    }

    fn on_exit(&self) {
        // Called when exiting pen mode
    }
}
