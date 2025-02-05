use super::EditModeSystem;
use bevy::prelude::*;

pub struct SelectMode;

impl EditModeSystem for SelectMode {
    fn update(&self, commands: &mut Commands) {
        // Implementation for select mode update
    }

    fn on_enter(&self) {
        // Called when entering select mode
    }

    fn on_exit(&self) {
        // Called when exiting select mode
    }
}
