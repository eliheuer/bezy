use crate::theme::*;
use anyhow::Result;
use bevy::prelude::*;
use norad::Font as Ufo;
use std::path::PathBuf;
use crate::hud::PressedButtonText;

/// Loads and validates the UFO font file, printing status to console.
/// Currently loads a test font from the design-assets directory.
fn load_ufo() {
    match try_load_ufo() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            println!(
                "Successfully loaded UFO font: {} {}",
                family_name, style_name
            );
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

/// Attempts to load the UFO font file and returns a Result.
/// Returns Ok(Ufo) if successful, or an Error if loading fails.
fn try_load_ufo() -> Result<Ufo> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let font_path = manifest_dir.join("assets/fonts/bezy-grotesk-regular.ufo");
    let ufo = Ufo::load(font_path)?;
    Ok(ufo)
}

/// Retrieves basic font information as a formatted string.
/// Returns a string containing the font family name and style,
/// or an error message if the font cannot be loaded.
fn get_basic_font_info() -> String {
    match try_load_ufo() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            format!("{} {}", family_name, style_name)
        }
        Err(e) => format!("Error loading font: {:?}", e),
    }
}

/// Initial setup system that runs on startup.
/// Spawns the UI camera and creates the font info text display.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_ufo();

    // Spawn UI camera
    commands.spawn(Camera2d);

    // Spawn your font info text (unchanged)
    commands.spawn((
        Text::new(get_basic_font_info()),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(32.0),
            ..default()
        },
    ));

    // Spawn a container for the buttons in the upper left corner.
    // We set its flex direction to Row so its children are arranged horizontally.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(32.0),
            // Ensure horizontal layout
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            // Create 8 buttons showing the tool names
            for tool in ["Select", "Pen", "Hyper", "Knife", "Pan", "Measure", "Rectangle", "Oval"] {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BorderRadius::all(Val::Px(0.0)),
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_child((
                        Text::new(tool),
                        TextFont {
                            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
            }
        });

    // Add center text display
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            // Center the text
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            // Negative margins of half the text size to center it
            margin: UiRect::new(Val::Px(-200.0), Val::Px(0.0), Val::Px(-32.0), Val::Px(0.0)),
            ..default()
        },
        PressedButtonText,
    ));
}
