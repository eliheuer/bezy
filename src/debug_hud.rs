// A hud for debugging

use crate::theme::get_debug_text_style;
use crate::toolbar::CurrentEditMode;
use bevy::prelude::*;

#[derive(Component)]
pub struct DebugText;

pub fn spawn_debug_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DebugText,
        Text::new(""),
        get_debug_text_style(&asset_server),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(128.0 - 16.0),
            left: Val::Px(36.0),
            ..default()
        },
    ));
}

pub fn update_debug_text(
    mut text_query: Query<&mut Text, With<DebugText>>,
    current_mode: Res<CurrentEditMode>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        text.0 = format!("Edit Mode: {:?}", current_mode.0);
    }
}
