// This should be a top level component that contains the toolbar
// and the rest of the UI that is not part of the zoomable design space or world

use crate::cameras::CoordinateDisplay;
use crate::theme::get_default_text_style;
use crate::toolbar::spawn_main_toolbar;
use bevy::prelude::*;

/// Spawns all HUD elements including toolbars and overlays
pub fn spawn_hud(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Spawn main toolbar
    spawn_main_toolbar(commands, asset_server);

    // Spawn glyph preview
    spawn_glyph_preview(commands, asset_server);

    // Spawn coordinate display
    spawn_coordinate_display(commands, asset_server);
}

fn spawn_glyph_preview(
    commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(-190.0),
        right: Val::Px(8.0),
        ..default()
    });
}

fn spawn_coordinate_display(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(32.0),
            bottom: Val::Px(64.0 + 8.0),
            ..default()
        },
        Text::new("X: 0.0, Y: 0.0"),
        get_default_text_style(asset_server),
        CoordinateDisplay,
    ));
}
