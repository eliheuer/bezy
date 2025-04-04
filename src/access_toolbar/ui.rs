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


/// Spawn the access toolbar with a Connect button in the upper right corner
pub fn spawn_access_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(TOOLBAR_MARGIN),      // Use theme margin for consistent spacing
            right: Val::Px(TOOLBAR_MARGIN),    // Use theme margin for consistent spacing
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
            margin: UiRect::all(Val::ZERO),    // Set margin to zero since we're using position offsets
            column_gap: Val::Px(TOOLBAR_ITEM_SPACING),    // Use consistent spacing between buttons
            ..default()
        },))
        .with_children(|parent| {
            spawn_access_button(parent, "\u{E008}", &asset_server);
        });
}

/// Creates a button with standard styling
fn spawn_access_button(
    commands: &mut ChildBuilder,
    label: &str,
    asset_server: &AssetServer,
) {
    commands
        .spawn((
            Button,
            ConnectButton,
            Node {
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
                border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
                margin: UiRect::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(TOOLBAR_BORDER_COLOR),
            BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),
            BackgroundColor(TOOLBAR_BACKGROUND_COLOR),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load(DEFAULT_FONT_PATH),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(TOOLBAR_ICON_COLOR),
            ));
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
            *bg_color = TOOLBAR_BACKGROUND_COLOR.into();
            border_color.0 = TOOLBAR_BORDER_COLOR;
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *bg_color = TOOLBAR_BACKGROUND_COLOR.into();
                    border_color.0 = TOOLBAR_BORDER_COLOR;
                }
                Interaction::Hovered => {
                    *bg_color = HOVERED_BUTTON.into();
                    border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
                }
                Interaction::None => {
                    *bg_color = TOOLBAR_BACKGROUND_COLOR.into();
                    border_color.0 = TOOLBAR_BORDER_COLOR;
                }
            }
        }
    }
}
