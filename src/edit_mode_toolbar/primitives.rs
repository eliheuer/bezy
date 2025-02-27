use crate::edit_mode_toolbar::EditModeSystem;
use bevy::prelude::*;

pub struct PrimitivesMode;

impl EditModeSystem for PrimitivesMode {
    fn update(&self, _commands: &mut Commands) {
        // TODO: Implement primitives mode behavior
    }

    fn on_enter(&self) {
        // Called when entering square mode
    }

    fn on_exit(&self) {
        // Called when exiting square mode
    }
}
