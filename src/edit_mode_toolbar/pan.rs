use super::EditModeSystem;
use bevy::prelude::*;

pub struct PanMode;

impl EditModeSystem for PanMode {
    fn update(&self, _commands: &mut Commands) {
        // Implementation for pan mode update
    }

    fn on_enter(&self) {
        // Called when entering pan mode
    }

    fn on_exit(&self) {
        // Called when exiting pan mode
    }
}
