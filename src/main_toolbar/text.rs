use super::EditModeSystem;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
pub struct TextMode;

impl EditModeSystem for TextMode {
    fn update(&self, _commands: &mut Commands) {
        // TODO: Implement text mode behavior
    }

    fn on_enter(&self) {
        // Called when entering select mode
    }

    fn on_exit(&self) {
        // Called when exiting select mode
    }
}
