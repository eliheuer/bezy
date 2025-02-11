// This should be a top level component that contains the toolbar
// and the rest of the UI that is not part of the zoomable design space or world

use crate::toolbar::spawn_main_toolbar;
use bevy::prelude::*;

/// Spawns all HUD elements including toolbars and overlays
pub fn spawn_hud(commands: &mut Commands, asset_server: &AssetServer) {
    // Spawn main toolbar
    spawn_main_toolbar(commands, asset_server);

    // Spawn glyph preview
    spawn_glyph_preview(commands, asset_server);
}

fn spawn_glyph_preview(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        Text::new("\u{E000}"),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 512.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(-190.0),
            right: Val::Px(8.0),
            ..default()
        },
    ));
}
