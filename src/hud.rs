// This should be a top level component that contains the toolbar
// and the rest of the non-debug UI that is not part of the zoomable design space

use crate::edit_mode_toolbar::spawn_edit_mode_toolbar;
use bevy::prelude::*;

/// Spawns all non-debug HUD elements (toolbars, etc.)
pub fn spawn_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    spawn_edit_mode_toolbar(commands, asset_server);
}
