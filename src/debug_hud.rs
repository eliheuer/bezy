// A debug display for the main toolbar state

use crate::theme::get_debug_text_style;
use crate::toolbar::CurrentEditMode;
use crate::ufo::get_basic_font_info;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainToolbarDebugText;

pub fn spawn_main_toolbar_debug(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        MainToolbarDebugText,
        Text::new(""),
        get_debug_text_style(&asset_server),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(128.0 - 16.0),
            left: Val::Px(32.0 + 4.0), // + = optical adjustment
            ..default()
        },
    ));
}

pub fn update_main_toolbar_debug(
    mut text_query: Query<&mut Text, With<MainToolbarDebugText>>,
    current_mode: Res<CurrentEditMode>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        text.0 = format!("Edit Mode: {:?}", current_mode.0);
    }
}

pub fn spawn_debug_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(get_basic_font_info()),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(32.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("أشهد يا إلهي"),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            right: Val::Px(32.0),
            ..default()
        },
    ));
}
