use crate::theme::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct ConnectButton;

#[derive(Resource, Default)]
pub struct ConnectButtonState {
    pub is_connected: bool,
}

/// Component for text color in UI elements.
// In Bevy 0.15.2, text styling is split between TextFont (for font and size) and this component for color.
// While the theme system handles static colors, this component is necessary for dynamic color changes
// at runtime (e.g., changing text color when a button is selected/deselected).
#[derive(Component)]
#[allow(dead_code)]
pub struct TextColor(pub Color);

/// Creates a button with standard styling
fn spawn_button(
    commands: &mut ChildBuilder,
    label: &str,
    asset_server: &Res<AssetServer>,
    width: f32,
) {
    commands
        .spawn((
            Button,
            ConnectButton,
            Node {
                width: Val::Px(width),
                height: Val::Px(64.0),
                padding: UiRect::all(Val::Px(0.0)),
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                get_default_text_style(asset_server),
                TextColor(Color::WHITE),
            ));
        });
}

/// Spawn the access toolbar with a Connect button in the upper right corner
pub fn spawn_access_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            right: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },))
        .with_children(|parent| {
            spawn_button(parent, "\u{E008}", &asset_server, 64.0);
        });
}

/// System to handle the Connect button interaction
pub fn handle_connect_button_interaction(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        With<ConnectButton>,
    >,
    mut button_state: ResMut<ConnectButtonState>,
) {
    // Handle interaction with Connect button
    for (_, interaction, mut bg_color, mut border_color) in
        &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            // Toggle connection state when pressed
            button_state.is_connected = !button_state.is_connected;
        }

        // Update button colors based on state
        if button_state.is_connected {
            *bg_color = PRESSED_BUTTON.into();
            border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *bg_color = PRESSED_BUTTON.into();
                    border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
                }
                Interaction::Hovered => {
                    *bg_color = HOVERED_BUTTON.into();
                    border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
                }
                Interaction::None => {
                    *bg_color = NORMAL_BUTTON.into();
                    border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
                }
            }
        }
    }
} 