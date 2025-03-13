use crate::edit_mode_toolbar::{CurrentPrimitiveType, PrimitiveType};
use crate::theme::*;
use bevy::prelude::*;

/// Component for the panel containing rounded rectangle specific settings
#[derive(Component)]
pub struct RoundedRectSettingsPanel;

/// Component for the corner radius input field
#[derive(Component)]
pub struct CornerRadiusInput;

/// Resource to store the current corner radius
#[derive(Resource)]
pub struct CurrentCornerRadius(pub f32);

/// Resource to track when UI elements are being interacted with
#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub is_interacting_with_ui: bool,
}

/// Local state for the corner radius input
#[derive(Default)]
pub struct CornerRadiusInputState {
    pub text: String,
    pub focused: bool,
}

impl Default for CurrentCornerRadius {
    fn default() -> Self {
        Self(10.0) // Default corner radius
    }
}

/// System to spawn UI elements for primitive-specific controls
pub fn spawn_primitive_controls(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    spawn_rounded_rect_controls(commands, asset_server);
}

/// Spawn UI elements specific to the rounded rectangle tool
fn spawn_rounded_rect_controls(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    commands
        .spawn((
            RoundedRectSettingsPanel,
            Name::new("RoundedRectSettingsPanel"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(184.0), // Moved down a bit more from the submenu
                left: Val::Px(34.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                min_width: Val::Px(190.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            Visibility::Hidden, // Start hidden
            BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
            BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
        ))
        .with_children(|parent| {
            // Label for the input field
            parent.spawn((
                Text::new("Corner Radius:"),
                TextFont {
                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Text input field for corner radius
            parent
                .spawn((
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
                    BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                    Interaction::None,
                    CornerRadiusInput,
                ))
                .with_children(|input_parent| {
                    // Text value display (also serves as input field value)
                    input_parent.spawn((
                        Text::new("10.0"),
                        TextFont {
                            font: asset_server
                                .load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        RadiusValueText,
                    ));
                });

            // Help text
            parent.spawn((
                Text::new("Type a value and press Enter"),
                TextFont {
                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));
        });
}

/// Component to mark the text that displays the current radius value
#[derive(Component)]
pub struct RadiusValueText;

/// System to update the visibility of primitive-specific UI panels based on current tool
pub fn update_primitive_ui_visibility(
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut rounded_rect_panel: Query<
        &mut Visibility,
        With<RoundedRectSettingsPanel>,
    >,
) {
    // Only show panels when in primitives mode
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Primitives {
        for mut visibility in rounded_rect_panel.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    // Show/hide panels based on current primitive type
    match current_primitive_type.0 {
        PrimitiveType::RoundedRectangle => {
            for mut visibility in rounded_rect_panel.iter_mut() {
                *visibility = Visibility::Visible;
            }
        }
        _ => {
            for mut visibility in rounded_rect_panel.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// System to handle the corner radius text input
pub fn handle_radius_input(
    mut input_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<CornerRadiusInput>),
    >,
    panel_query: Query<&Node, With<RoundedRectSettingsPanel>>,
    mut windows: Query<&mut Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut current_radius: ResMut<CurrentCornerRadius>,
    mut radius_text: Query<&mut Text, With<RadiusValueText>>,
    mut input_state: Local<CornerRadiusInputState>,
    mut ui_state: ResMut<UiInteractionState>,
) {
    // Handle input field focus/click
    for (interaction, mut bg_color, mut border_color) in input_query.iter_mut()
    {
        match *interaction {
            Interaction::Hovered => {
                bg_color.0 = HOVERED_BUTTON;
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::Pressed => {
                // When clicked, focus the input and set background
                input_state.focused = true;
                bg_color.0 = PRESSED_BUTTON;
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::None => {
                if !input_state.focused {
                    bg_color.0 = NORMAL_BUTTON;
                    border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
                }
            }
        }
    }

    // Handle mouse clicks outside to unfocus
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window = windows.single_mut();

        // Check if click is over the panel
        if let (Some(cursor_pos), Ok(panel_node)) =
            (window.cursor_position(), panel_query.get_single())
        {
            let panel_left = 32.0;
            let panel_top = 200.0;
            let panel_width = match panel_node.min_width {
                Val::Px(width) => width,
                _ => 200.0,
            };
            let panel_height = 100.0; // Approximate height based on content

            // If input is focused and mouse is clicked outside the input area, unfocus
            let is_over_panel = cursor_pos.x >= panel_left
                && cursor_pos.x <= panel_left + panel_width
                && cursor_pos.y >= panel_top
                && cursor_pos.y <= panel_top + panel_height;

            // Set UI interaction state for drawing prevention
            ui_state.is_interacting_with_ui = is_over_panel;

            // Check if click is in the panel but not immediately after focusing
            if !is_over_panel && input_state.focused {
                // Apply the current value before unfocusing
                apply_input_value(
                    input_state.text.clone(),
                    &mut current_radius,
                    &mut radius_text,
                    &mut input_state,
                );
                input_state.focused = false;
            }
        }
    }

    // Handle keyboard input when focused
    if input_state.focused {
        // Handle character input
        for key in keyboard_input.get_pressed() {
            match key {
                // Only process keys when they are just pressed
                _ if !keyboard_input.just_pressed(*key) => continue,

                // Handle backspace
                KeyCode::Backspace => {
                    if !input_state.text.is_empty() {
                        input_state.text.pop();
                        update_text_display(
                            &input_state.text,
                            &mut radius_text,
                        );
                    }
                }

                // Handle Enter key to submit
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    apply_input_value(
                        input_state.text.clone(),
                        &mut current_radius,
                        &mut radius_text,
                        &mut input_state,
                    );
                    input_state.focused = false;
                }

                // Handle Escape key to cancel
                KeyCode::Escape => {
                    // Reset to current value
                    input_state.text = current_radius.0.to_string();
                    update_text_display(&input_state.text, &mut radius_text);
                    input_state.focused = false;
                }

                // Handle numeric input and decimal point
                KeyCode::Digit0 | KeyCode::Numpad0 => add_char_to_input(
                    '0',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit1 | KeyCode::Numpad1 => add_char_to_input(
                    '1',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit2 | KeyCode::Numpad2 => add_char_to_input(
                    '2',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit3 | KeyCode::Numpad3 => add_char_to_input(
                    '3',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit4 | KeyCode::Numpad4 => add_char_to_input(
                    '4',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit5 | KeyCode::Numpad5 => add_char_to_input(
                    '5',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit6 | KeyCode::Numpad6 => add_char_to_input(
                    '6',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit7 | KeyCode::Numpad7 => add_char_to_input(
                    '7',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit8 | KeyCode::Numpad8 => add_char_to_input(
                    '8',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Digit9 | KeyCode::Numpad9 => add_char_to_input(
                    '9',
                    &mut input_state.text,
                    &mut radius_text,
                ),
                KeyCode::Period | KeyCode::NumpadDecimal => add_char_to_input(
                    '.',
                    &mut input_state.text,
                    &mut radius_text,
                ),

                _ => {} // Ignore other keys
            }
        }
    }
}

// Helper to add a character to the input text
fn add_char_to_input(
    c: char,
    text: &mut String,
    text_query: &mut Query<&mut Text, With<RadiusValueText>>,
) {
    // For decimal point, only add if there isn't one already
    if c == '.' && text.contains('.') {
        return;
    }

    // Otherwise add the character
    text.push(c);
    update_text_display(text, text_query);
}

// Helper to update the text display
fn update_text_display(
    text: &str,
    text_query: &mut Query<&mut Text, With<RadiusValueText>>,
) {
    if let Ok(mut text_comp) = text_query.get_single_mut() {
        // Show cursor at end of text when editing
        *text_comp = Text::new(if text.is_empty() {
            "_".to_string()
        } else {
            format!("{}_", text)
        });
    }
}

// Helper to apply the input value
fn apply_input_value(
    text: String,
    current_radius: &mut ResMut<CurrentCornerRadius>,
    text_query: &mut Query<&mut Text, With<RadiusValueText>>,
    input_state: &mut CornerRadiusInputState,
) {
    // Parse the input text as a float
    if let Ok(value) = text.parse::<f32>() {
        // Clamp the value to reasonable limits (0-2048)
        let clamped_value = value.clamp(0.0, 2048.0);
        current_radius.0 = clamped_value;
        input_state.text = format!("{:.1}", clamped_value);

        // Update the display
        if let Ok(mut text_comp) = text_query.get_single_mut() {
            *text_comp = Text::new(format!("{:.1}", clamped_value));
        }

        info!("Corner radius set to: {:.1}", clamped_value);
    } else {
        // Invalid input, reset to current value
        input_state.text = format!("{:.1}", current_radius.0);
        if let Ok(mut text_comp) = text_query.get_single_mut() {
            *text_comp = Text::new(format!("{:.1}", current_radius.0));
        }
    }
}
