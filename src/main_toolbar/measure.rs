use super::EditModeSystem;
use bevy::prelude::*;

pub struct MeasureMode;

impl EditModeSystem for MeasureMode {
    fn update(&self, _commands: &mut Commands) {
        // Implementation for measure mode update
    }

    fn on_enter(&self) {
        // Called when entering measure mode
    }

    fn on_exit(&self) {
        // Called when exiting measure mode
    }
}
