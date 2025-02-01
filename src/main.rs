// A font editor made with the Bevy game engine.

use bevy::prelude::*;
use bevy::color::palettes::basic::*;
use bevy::winit::WinitSettings;
use norad::Font as Ufo;
use anyhow::Result;

// Constants, think of this like the "settings" for the UI.
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

/// Main entry point for the Bezy font editor application.
/// Sets up the window and initializes the Bevy app with required plugins and systems.
fn main() {
    App::new()
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy".into(),
                resolution: (1024., 768.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Darker gray background
        .add_systems(Startup, (setup, spawn_grid))
        .add_systems(Update, button_system)
        .run();
}

/// Initial setup system that runs on startup.
/// Spawns the UI camera and creates the font info text display.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_ufo();
    // UI camera
    commands.spawn(Camera2d);

    // Text
    commands.spawn((
        Text::new(get_basic_font_info()),
        TextFont {
            font: asset_server.load("fonts/SkynetGrotesk-RegularDisplay.ttf"),
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
    // Spawn a button
    commands
    .spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(126.0),
                    height: Val::Px(62.0),
                    
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::srgb(0.2, 0.2, 0.2)),
                BorderRadius::all(Val::Px(0.0)),
                BackgroundColor(NORMAL_BUTTON),
            ))
            .with_child((
                Text::new("Button"),
                TextFont {
                    font: asset_server.load("fonts/SkynetGrotesk-RegularDisplay.ttf"),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
    });
}

/// Loads and validates the UFO font file, printing status to console.
/// Currently loads a test font from the design-assets directory.
fn load_ufo() {
    match try_load_ufo() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            println!("Successfully loaded UFO font: {} {}", family_name, style_name);
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

/// Attempts to load the UFO font file and returns a Result.
/// Returns Ok(Ufo) if successful, or an Error if loading fails.
fn try_load_ufo() -> Result<Ufo> {
    let path = "design-assets/test-fonts/test-font-001.ufo";
    let ufo = Ufo::load(path)?;
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
            format!("Font: {} {}", family_name, style_name)
        }
        Err(e) => format!("Error loading font: {:?}", e)
    }
}

/// Spawns a grid centered in the window.
/// Creates both vertical and horizontal lines with semi-transparent gray color.
fn spawn_grid(mut commands: Commands) {
    // Get window dimensions (using a larger value to ensure coverage)
    let window_width = 2048.0;  // Doubled from window width
    let window_height = 1536.0; // Doubled from window height
    let grid_position = Vec2::new(0.0, 0.0); // Center of the window
    
    // Create vertical lines
    for i in -512..=512 {  // Increased range
        let x = grid_position.x + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.2),
                custom_size: Some(Vec2::new(1.0, window_height)),
                ..default()
            },
            Transform::from_xyz(x * 32.0, grid_position.y, 0.0),
        ));
    }

    // Create horizontal lines
    for i in -512..=512 {  // Increased range
        let y = grid_position.y + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.2),
                custom_size: Some(Vec2::new(window_width, 1.0)),
                ..default()
            },
            Transform::from_xyz(grid_position.x, y * 32.0, 0.0),
        ));
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
