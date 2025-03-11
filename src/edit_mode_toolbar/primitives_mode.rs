use crate::edit_mode_toolbar::EditModeSystem;
use crate::theme::*;
use bevy::prelude::*;

// Import primitive shapes modules directly
use crate::edit_mode_toolbar::primitives::base;

// An enum to track which primitive type is currently selected
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum PrimitiveType {
    #[default]
    Rectangle,
    Ellipse,
    // Future primitive types can be added here (stars, hexagons, etc.)
}

impl PrimitiveType {
    // Get the icon for each primitive type
    pub fn get_icon(&self) -> &'static str {
        match self {
            // Temporarily reusing the primitives icon for all types
            PrimitiveType::Rectangle => "\u{E016}",
            PrimitiveType::Ellipse => "\u{E016}",
        }
    }

    // Get the display name for each primitive type
    pub fn display_name(&self) -> &'static str {
        match self {
            PrimitiveType::Rectangle => "Rectangle",
            PrimitiveType::Ellipse => "Ellipse",
        }
    }
}

// Component to mark primitive sub-menu buttons
#[derive(Component)]
pub struct PrimitiveSubMenuButton;

// Component to associate a button with its primitive type
#[derive(Component)]
pub struct PrimitiveTypeButton(pub PrimitiveType);

// Resource to track the currently selected primitive type
#[derive(Resource, Default)]
pub struct CurrentPrimitiveType(pub PrimitiveType);

pub struct PrimitivesMode;

impl EditModeSystem for PrimitivesMode {
    fn update(&self, commands: &mut Commands) {
        // The actual implementation will need to access the current primitive type through a system parameter
        // For now, we'll just use the default rectangle
        let primitive_tool = base::get_primitive_tool(PrimitiveType::Rectangle);
        primitive_tool.update(commands);
    }

    fn on_enter(&self) {
        // Called when entering primitives mode - will show the sub-menu
        info!("Entered primitives mode");
    }

    fn on_exit(&self) {
        // Called when exiting primitives mode - will hide the sub-menu
        info!("Exited primitives mode");
    }
}

// Now add a system that will handle the active primitive tool
pub fn handle_active_primitive_tool(
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
) {
    // Only update when in primitives mode
    if current_mode.0 == crate::edit_mode_toolbar::EditMode::Primitives {
        let tool = base::get_primitive_tool(current_primitive_type.0);
        tool.update(&mut commands);
    }
}

// System to spawn the primitives sub-menu
pub fn spawn_primitives_submenu(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    // Create a submenu container that sits below the main toolbar
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(108.0), // Position below the main toolbar (32px + 64px + spacing)
                left: Val::Px(32.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Name::new("PrimitivesSubMenu"),
            // Start as hidden until primitives mode is selected
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            let primitive_types =
                [PrimitiveType::Rectangle, PrimitiveType::Ellipse];

            for primitive_type in primitive_types.iter() {
                spawn_primitive_button(parent, primitive_type, asset_server);
            }
        });
}

// Helper function to spawn a single primitive type button
fn spawn_primitive_button(
    parent: &mut ChildBuilder,
    primitive_type: &PrimitiveType,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        })
        .with_children(|button_container| {
            button_container
                .spawn((
                    Button,
                    PrimitiveSubMenuButton,
                    PrimitiveTypeButton(*primitive_type),
                    Name::new(format!(
                        "{}Button",
                        primitive_type.display_name()
                    )),
                    Node {
                        width: Val::Px(64.0),
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
                    // Add the icon
                    button.spawn((
                        Text::new(primitive_type.get_icon().to_string()),
                        TextFont {
                            font: asset_server
                                .load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Add a text label below the icon
                    button.spawn((
                        Text::new(primitive_type.display_name().to_string()),
                        TextFont {
                            font: asset_server
                                .load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(4.0),
                            ..default()
                        },
                    ));
                });
        });
}

// System to handle primitive type selection
pub fn handle_primitive_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &PrimitiveTypeButton,
            Entity,
        ),
        (Changed<Interaction>, With<PrimitiveSubMenuButton>),
    >,
    mut text_query: Query<(&Parent, &mut TextColor)>,
    mut current_type: ResMut<CurrentPrimitiveType>,
) {
    for (interaction, mut color, mut border_color, primitive_button, entity) in
        &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            // Update the current primitive type
            current_type.0 = primitive_button.0;
            info!("Selected primitive type: {:?}", current_type.0);
        }

        // Update button appearance based on current selection
        let is_current_type = current_type.0 == primitive_button.0;

        // Update button colors
        match (*interaction, is_current_type) {
            (Interaction::Pressed, _) | (_, true) => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::Hovered, false) => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::None, false) => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }

        // Update text color for this button
        for (parent, mut text_color) in &mut text_query {
            if parent.get() == entity {
                text_color.0 = if is_current_type {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
            }
        }
    }
}

// System to show/hide the primitive sub-menu based on the current edit mode
pub fn toggle_primitive_submenu_visibility(
    current_edit_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut submenu_query: Query<(&mut Visibility, &Name)>,
) {
    // Find the primitives submenu by name
    for (mut visibility, name) in submenu_query.iter_mut() {
        if name.as_str() == "PrimitivesSubMenu" {
            // Show the submenu only when in primitives mode
            *visibility = if current_edit_mode.0
                == crate::edit_mode_toolbar::EditMode::Primitives
            {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
