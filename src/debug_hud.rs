// A debug display for the main toolbar state

use crate::data::AppState;
use crate::theme::get_default_text_style;
use crate::theme::DEFAULT_FONT_PATH;
use crate::toolbar::CurrentEditMode;
use crate::ufo::get_basic_font_info_from_state;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub struct MainToolbarDebugText;

#[derive(Component)]
pub struct FontInfoText;

pub fn spawn_main_toolbar_debug(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        MainToolbarDebugText,
        Text::new(""),
        get_default_text_style(&asset_server),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(128.0 - 16.0),
            left: Val::Px(32.0 + 4.0), // + = optical adjustment
            ..default()
        },
        RenderLayers::layer(1), // UI layer
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

pub fn update_font_info_text(
    mut text_query: Query<&mut Text, With<FontInfoText>>,
    app_state: Res<AppState>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        text.0 = get_basic_font_info_from_state(&app_state);
    }
}

pub fn spawn_debug_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
) {
    commands.spawn((
        FontInfoText,
        Text::new(get_basic_font_info_from_state(&app_state)),
        get_default_text_style(&asset_server),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(32.0),
            left: Val::Px(32.0),
            ..default()
        },
        RenderLayers::layer(1), // UI layer
    ));
    commands.spawn((
        // Text::new("أشهد يا إلهي بانك خلقتني لعرفانك وعبادتك"),
        Text::new("أشهد يا إلهي بانك خلقتني"),
        // TextFont {
        //     font: asset_server.load(DEFAULT_FONT_PATH),
        //     font_size: 64.0,
        //     ..default()
        // },
        get_default_text_style(&asset_server),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(32.0),
            right: Val::Px(32.0),
            ..default()
        },
        RenderLayers::layer(1), // UI layer
    ));
    commands.spawn((
        Text::new("\u{E000}"),
        TextFont {
            font: asset_server.load(DEFAULT_FONT_PATH),
            font_size: 512.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(-190.0),
            right: Val::Px(24.0),
            ..default()
        },
        RenderLayers::layer(1), // UI layer
    ));
}
