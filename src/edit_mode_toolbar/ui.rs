use crate::edit_mode_toolbar::*;
use crate::theme::*;

#[derive(Component)]
pub struct EditModeToolbarButton;

#[derive(Component)]
pub struct TextColor(pub Color);

#[derive(Component)]
pub struct ButtonName(pub String);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EditMode {
    #[default]
    Select,
    Pen,
    Hyper,
    Knife,
    Pan,
    Measure,
    Primitives,
    Text,
}

impl EditMode {
    pub fn get_system(&self) -> Box<dyn EditModeSystem> {
        match self {
            EditMode::Select => Box::new(SelectMode),
            EditMode::Pen => Box::new(PenMode),
            EditMode::Hyper => Box::new(HyperMode),
            EditMode::Knife => Box::new(KnifeMode),
            EditMode::Pan => Box::new(PanMode),
            EditMode::Measure => Box::new(MeasureMode),
            EditMode::Primitives => Box::new(PrimitivesMode),
            EditMode::Text => Box::new(TextMode),
        }
    }

    /// Returns the Unicode PUA icon for each edit mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            EditMode::Select => "\u{E010}",
            EditMode::Pen => "\u{E011}",
            EditMode::Hyper => "\u{E012}",
            EditMode::Knife => "\u{E013}",
            EditMode::Pan => "\u{E014}",
            EditMode::Measure => "\u{E015}",
            EditMode::Primitives => "\u{E016}",
            EditMode::Text => "\u{E017}",
        }
    }

    /// Returns a user-friendly display name for each edit mode
    pub fn display_name(&self) -> &'static str {
        match self {
            EditMode::Select => "Select",
            EditMode::Pen => "Pen",
            EditMode::Hyper => "Hyper",
            EditMode::Knife => "Knife",
            EditMode::Pan => "Pan",
            EditMode::Measure => "Measure",
            EditMode::Primitives => "Primitives",
            EditMode::Text => "Text",
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentEditMode(pub EditMode);

pub fn spawn_edit_mode_toolbar(
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    // Spawn a container for the edit mode toolbar buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            // Create a button for each edit mode type
            let edit_modes = [
                EditMode::Select,
                EditMode::Pen,
                EditMode::Hyper,
                EditMode::Knife,
                EditMode::Pan,
                EditMode::Measure,
                EditMode::Primitives,
                EditMode::Text,
            ];

            for edit_mode in edit_modes.iter() {
                spawn_mode_button(parent, edit_mode, asset_server);
            }
        });

    // Also spawn the primitives sub-menu (it will start hidden)
    crate::edit_mode_toolbar::spawn_primitives_submenu(commands, asset_server);
}

/// Helper function to spawn a single mode button
fn spawn_mode_button(
    parent: &mut ChildBuilder,
    edit_mode: &EditMode,
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
                    EditModeToolbarButton,
                    ButtonName(edit_mode.display_name().to_string()),
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
                    // Add the icon using the EditMode method
                    button.spawn((
                        Text::new(edit_mode.get_icon().to_string()),
                        TextFont {
                            font: asset_server
                                .load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 48.0, // Consistent size for all icons
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn handle_toolbar_mode_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ButtonName,
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    mut text_query: Query<(&Parent, &mut TextColor)>,
    mut current_mode: ResMut<CurrentEditMode>,
) {
    // First handle any new interactions
    for (interaction, _color, _border_color, button_name, _entity) in
        &mut interaction_query
    {
        if *interaction == Interaction::Pressed {
            // Parse the button name to an EditMode
            let new_mode = parse_edit_mode_from_button_name(&button_name.0);

            // Only process if the mode is actually changing
            if current_mode.0 != new_mode {
                // Get the old mode's system and call on_exit
                let old_system = current_mode.0.get_system();
                old_system.on_exit();

                // Call on_enter for the new mode
                let new_system = new_mode.get_system();
                new_system.on_enter();

                // Save the new mode
                current_mode.0 = new_mode;

                // Log only when mode actually changes
                info!("Switched edit mode to: {:?}", new_mode);
            }
        }
    }

    // Then update all button appearances based on the current mode
    for (interaction, mut color, mut border_color, button_name, entity) in
        &mut interaction_query
    {
        let is_current_mode = match button_name.0.as_str() {
            "Select" => current_mode.0 == EditMode::Select,
            "Pen" => current_mode.0 == EditMode::Pen,
            "Hyper" => current_mode.0 == EditMode::Hyper,
            "Knife" => current_mode.0 == EditMode::Knife,
            "Pan" => current_mode.0 == EditMode::Pan,
            "Measure" => current_mode.0 == EditMode::Measure,
            "Primitives" => current_mode.0 == EditMode::Primitives,
            "Text" => current_mode.0 == EditMode::Text,
            _ => false,
        };

        // Update button colors
        match (*interaction, is_current_mode) {
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
                text_color.0 = if is_current_mode {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
            }
        }
    }
}

/// Helper function to parse a button name into an EditMode
fn parse_edit_mode_from_button_name(button_name: &str) -> EditMode {
    match button_name {
        "Select" => EditMode::Select,
        "Pen" => EditMode::Pen,
        "Hyper" => EditMode::Hyper,
        "Knife" => EditMode::Knife,
        "Pan" => EditMode::Pan,
        "Measure" => EditMode::Measure,
        "Primitives" => EditMode::Primitives,
        "Text" => EditMode::Text,
        _ => {
            warn!(
                "Unknown edit mode button: {}, defaulting to Select",
                button_name
            );
            EditMode::Select
        }
    }
}

pub fn update_current_edit_mode(
    mut commands: Commands,
    current_mode: Res<CurrentEditMode>,
) {
    let system = current_mode.0.get_system();
    system.update(&mut commands);
}
