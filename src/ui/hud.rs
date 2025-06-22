//! Head-up display (HUD) management for the Bezy font editor
//!
//! This module manages the top-level UI components that are not part of the
//! zoomable design space, including toolbars and information panels.

use bevy::prelude::*;

/// Spawns all non-debug HUD elements (toolbars, etc.)
/// 
/// Note: Many components are now spawned automatically by their respective plugins.
/// This function serves as a coordination point for UI elements that need manual spawning.
pub fn spawn_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // HUD components will be added as we port more UI modules
    debug!("HUD spawning initiated");
    
    // Placeholder for future HUD components:
    // - Glyph navigation pane
    // - Coordinate display pane  
    // - Status bar
    // - Menu system
    
    // For now, just log that we're setting up the HUD
    info!("HUD setup complete - components will be added during port");
}

/// Plugin to manage HUD-related systems
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud_system);
    }
}

/// System to set up the HUD during startup
fn setup_hud_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_hud(&mut commands, &asset_server);
} 