// This should be a top level component that contains the toolbar
// and the rest of the non-debug UI that is not part of the zoomable design space

use crate::ui::panes::glyph_pane::spawn_glyph_pane;
use crate::ui::toolbars::edit_mode_toolbar::spawn_primitive_controls;
use bevy::prelude::*;

/// Spawns all non-debug HUD elements (toolbars, etc.)
/// Note: The edit mode toolbar is now spawned automatically by the EditModeToolbarPlugin
pub fn spawn_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    spawn_glyph_pane(commands, asset_server);
    spawn_primitive_controls(commands, asset_server);
}
