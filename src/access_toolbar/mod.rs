use bevy::prelude::*;
mod ui;
pub use ui::*;

/// Plugin that adds the access toolbar "connect" button functionality
pub struct AccessToolbarPlugin;

impl Plugin for AccessToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectButtonState>()
            .add_systems(Startup, spawn_access_toolbar)
            .add_systems(Update, handle_connect_button_interaction);
        info!("AccessToolbarPlugin initialized");
    }
} 