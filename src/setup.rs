use crate::stub::load_ufo;
use crate::toolbar::spawn_main_toolbar;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load UFO font
    load_ufo();

    // Spawn UI camera
    commands.spawn(Camera2d);

    // Spawn main toolbar
    spawn_main_toolbar(&mut commands, &asset_server, &mut texture_atlas_layouts);

    // Spawn debug text
    commands.spawn((
        Text::new("\u{E000}"),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 768.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(-256.0),
            right: Val::Px(16.0),
            ..default()
        },
    ));
}
