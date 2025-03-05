use super::EditModeSystem;
use bevy::prelude::*;

/// Plugin to register selection mode systems
pub struct SelectModePlugin;

impl Plugin for SelectModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectModeActive>();
    }
}

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

/// Selection mode for manipulating points
pub struct SelectMode;

impl EditModeSystem for SelectMode {
    fn update(&self, commands: &mut Commands) {
        // Mark select mode as active
        commands.insert_resource(SelectModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entering Select Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Select Mode");
    }
}
