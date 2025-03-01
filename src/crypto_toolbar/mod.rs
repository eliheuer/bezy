use bevy::prelude::*;

mod ui;

pub use ui::*;

/// Plugin that adds the crypto toolbar functionality
pub struct CryptoToolbarPlugin;

impl Plugin for CryptoToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectButtonState>()
           .add_systems(Startup, spawn_crypto_toolbar)
           .add_systems(Update, handle_connect_button_interaction);

        info!("CryptoToolbarPlugin initialized with Connect button interaction");
    }
} 