use super::EditModeSystem;
use bevy::prelude::*;

pub struct TextMode;

impl EditModeSystem for TextMode {
    fn update(&self, _commands: &mut Commands) {
        // Implementation for text mode update
    }

    fn on_enter(&self) {
        // Called when entering text mode
    }

    fn on_exit(&self) {
        // Called when exiting text mode
    }
}
