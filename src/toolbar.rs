use crate::main_toolbar::*;
use crate::theme::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainToolbarButton;

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
}

#[derive(Resource, Default)]
pub struct CurrentEditMode(pub EditMode);

pub fn spawn_main_toolbar(commands: &mut Commands, asset_server: &AssetServer) {
    // Spawn a container for the main toolbar buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            for (_index, button_name) in [
                "Select",
                "Pen",
                "Hyper",
                "Knife",
                "Pan",
                "Measure",
                "Primitives",
                "Text",
            ]
            .iter()
            .enumerate()
            {
                parent
                    .spawn(Node {
                        margin: UiRect::all(Val::Px(4.0)),
                        ..default()
                    })
                    .with_children(|button_container| {
                        button_container
                            .spawn((
                                Button,
                                MainToolbarButton,
                                ButtonName(button_name.to_string()),
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
                                // Add the icon for the button
                                let icon = match button_name.to_string().as_str() {
                                    "Select" => "\u{E010}",
                                    "Pen" => "\u{E011}",
                                    "Hyper" => "\u{E012}",
                                    "Knife" => "\u{E013}",
                                    "Pan" => "\u{E014}",
                                    "Measure" => "\u{E015}",
                                    "Primitives" => "\u{E016}",
                                    "Text" => "\u{E017}",
                                    _ => "",
                                }
                                .to_string();

                                button.spawn((
                                    Text::new(icon),
                                    TextFont {
                                        font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                        font_size: 48.0, // Consistent size for all icons
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
            }
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
        With<MainToolbarButton>,
    >,
    mut text_query: Query<(&Parent, &mut TextColor)>,
    mut current_mode: ResMut<CurrentEditMode>,
) {
    // First handle any new interactions
    for (interaction, _color, _border_color, button_name, _entity) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Get the old mode's system and call on_exit
            let old_system = current_mode.0.get_system();
            old_system.on_exit();

            // Update the current edit mode based on the button pressed
            let new_mode = match button_name.0.as_str() {
                "Select" => EditMode::Select,
                "Pen" => EditMode::Pen,
                "Hyper" => EditMode::Hyper,
                "Knife" => EditMode::Knife,
                "Pan" => EditMode::Pan,
                "Measure" => EditMode::Measure,
                "Primitives" => EditMode::Primitives,
                "Text" => EditMode::Text,
                _ => EditMode::Select,
            };

            // Get the new mode's system and call on_enter
            let new_system = new_mode.get_system();
            new_system.on_enter();

            current_mode.0 = new_mode;
        }
    }

    // Then update all button appearances based on the current mode
    for (interaction, mut color, mut border_color, button_name, entity) in &mut interaction_query {
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

pub fn update_current_edit_mode(mut commands: Commands, current_mode: Res<CurrentEditMode>) {
    let system = current_mode.0.get_system();
    system.update(&mut commands);
}
