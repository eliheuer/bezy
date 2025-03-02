use bevy::prelude::*;
use crate::theme::*;

#[derive(Component)]
pub struct ConnectButton;

/// Component for text color in UI elements
#[derive(Component)]
pub struct TextColor(pub Color);

#[derive(Resource, Default)]
pub struct ConnectButtonState {
    pub is_connected: bool,
}

/// Spawn the crypto toolbar with a Connect button in the upper right corner
pub fn spawn_crypto_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn a container for the crypto toolbar
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            right: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },))
        .with_children(|parent| {
            // Add the Connect button
            parent
                .spawn(Node {
                    margin: UiRect::all(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|button_container| {
                    button_container
                        .spawn((
                            Button,
                            ConnectButton,
                            Node {
                                width: Val::Px(192.0),
                                height: Val::Px(64.0),
                                padding: UiRect::all(Val::Px(0.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::WHITE),
                            BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|button| {
                            // Add the text label for the button
                            button.spawn((
                                Text::new("Connect"),
                                get_default_text_style(&asset_server),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
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
