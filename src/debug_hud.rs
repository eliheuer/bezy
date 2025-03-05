// A debug display for the edit mode toolbar state

use crate::data::AppState;
use crate::theme::get_default_text_style;
use crate::theme::DEFAULT_FONT_PATH;
use crate::ufo::get_basic_font_info_from_state;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub struct FontInfoText;

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
            font_size: 256.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(32.0),
            ..default()
        },
        RenderLayers::layer(1), // UI layer
    ));
}
