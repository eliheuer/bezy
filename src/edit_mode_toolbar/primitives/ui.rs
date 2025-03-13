use crate::edit_mode_toolbar::{CurrentPrimitiveType, PrimitiveType};
use crate::theme::*;
use bevy::prelude::*;

/// Component for the panel containing rounded rectangle specific settings
#[derive(Component)]
pub struct RoundedRectSettingsPanel;

/// Component for the corner radius slider
#[derive(Component)]
pub struct CornerRadiusSlider;

/// Resource to store the current corner radius
#[derive(Resource)]
pub struct CurrentCornerRadius(pub f32);

/// Resource to track when UI elements are being interacted with
#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub is_interacting_with_ui: bool,
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
                top: Val::Px(180.0), // Position below the primitives submenu
                left: Val::Px(32.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                min_width: Val::Px(200.0),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            Visibility::Hidden, // Start hidden
            BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
        ))
        .with_children(|parent| {
            // Label for the slider
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

            // Slider container
            parent
                .spawn((Node {
                    width: Val::Px(180.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|slider_parent| {
                    // Background track
                    slider_parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ));

                    // Slider handle (draggable)
                    slider_parent.spawn((
                        CornerRadiusSlider,
                        Button,
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(90.0), // Initial position at 50%
                            top: Val::Px(-10.0), // Center vertically (24px height - 4px track height) / 2
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                        BorderColor(Color::srgb(0.7, 0.7, 0.7)), // Add a border for better visibility
                    ));
                });

            // Current value display
            parent.spawn((
                Text::new("10.0"),
                TextFont {
                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
                RadiusValueText,
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

/// System to handle dragging the corner radius slider
pub fn handle_radius_slider(
    mut params: ParamSet<(
        Query<
            (&Interaction, &mut Node, &mut BackgroundColor),
            With<CornerRadiusSlider>,
        >,
        Query<&Node, With<RoundedRectSettingsPanel>>,
    )>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: Local<Option<()>>,
    mut windows: Query<&mut Window>,
    mut current_radius: ResMut<CurrentCornerRadius>,
    mut radius_text: Query<&mut Text, With<RadiusValueText>>,
    mut ui_state: ResMut<UiInteractionState>,
    slider_interaction: Query<&Interaction, With<CornerRadiusSlider>>,
) {
    // Check if mouse is hovering over slider (even without interaction change)
    let is_hovering = slider_interaction
        .iter()
        .any(|interaction| *interaction == Interaction::Hovered);

    // Update slider appearance when interaction state changes
    {
        let mut slider_query = params.p0();
        for (interaction, _, mut background_color) in slider_query.iter_mut() {
            match *interaction {
                Interaction::Hovered => {
                    // Highlight when hovered
                    background_color.0 = Color::srgb(0.9, 0.9, 1.0);
                }
                Interaction::Pressed => {
                    // Darker when pressed
                    background_color.0 = Color::srgb(0.7, 0.7, 0.8);
                }
                Interaction::None => {
                    // Normal state
                    background_color.0 = Color::WHITE;
                }
            }
        }
    }

    // Check if the mouse is over the panel - consider the panel dimensions
    let window = windows.single_mut();
    let is_over_panel = if let Some(cursor_pos) = window.cursor_position() {
        // Panel query must be accessed through the params
        let panel_query = params.p1();
        if let Ok(panel_node) = panel_query.get_single() {
            // Panel position and dimensions
            let panel_left = 32.0;
            let panel_top = 180.0;
            let panel_width = match panel_node.min_width {
                Val::Px(width) => width,
                _ => 200.0, // Default width if not a pixel value
            };
            let panel_height = 100.0; // Approximate height based on content

            // Check if cursor is inside panel bounds
            cursor_pos.x >= panel_left
                && cursor_pos.x <= panel_left + panel_width
                && cursor_pos.y >= panel_top
                && cursor_pos.y <= panel_top + panel_height
        } else {
            false
        }
    } else {
        false
    };

    // Set the interaction state based on hovering, dragging, or being over the panel
    ui_state.is_interacting_with_ui =
        is_hovering || drag_state.is_some() || is_over_panel;

    // Update drag state
    {
        let slider_query = params.p0();
        if mouse_button.just_pressed(MouseButton::Left) {
            for (interaction, _, _) in slider_query.iter() {
                if *interaction == Interaction::Hovered {
                    *drag_state = Some(());
                    ui_state.is_interacting_with_ui = true;
                }
            }
        } else if mouse_button.just_released(MouseButton::Left) {
            *drag_state = None;
        }
    }

    // Handle dragging
    if drag_state.is_some() {
        if let Some(cursor_position) = window.cursor_position() {
            // Calculate slider position based on mouse X position
            let mut slider_query = params.p0();
            if let Ok((_, mut style, _)) = slider_query.get_single_mut() {
                // Get bounds of slider (hardcoded to match the slider track width)
                let min_x = 0.0;
                let max_x = 180.0;
                let parent_left = 32.0 + 10.0; // Adjust based on panel position and padding

                // Calculate relative position
                let rel_x =
                    (cursor_position.x - parent_left).clamp(min_x, max_x);
                let percentage = rel_x / max_x;

                // Update slider position
                style.left = Val::Px(rel_x);

                // Map to corner radius range (0.0 to 50.0)
                let new_radius = percentage * 50.0;
                current_radius.0 = new_radius;

                // Update the value text
                if let Ok(mut text) = radius_text.get_single_mut() {
                    // Update text with the new radius value
                    *text = Text::new(format!("{:.1}", new_radius));
                }
            }
        }
    }
}
