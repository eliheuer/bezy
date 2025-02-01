//! A font editor made with Bevy, with inspiration from Runebender.

use bevy::prelude::*;
use norad::Font as Ufo;
use anyhow::Result;

/// Main entry point for the Bezy font editor application.
/// Sets up the window and initializes the Bevy app with required plugins and systems.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy".into(),
                resolution: (1024., 768.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1))) // Darker gray background
        .add_systems(Startup, (setup, spawn_grid))
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
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(32.0),
            left: Val::Px(32.0),
            ..default()
        },
    ));
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
            format!("Font: {} - {}", family_name, style_name)
        }
        Err(e) => format!("Error loading font: {:?}", e)
    }
}

/// Spawns a grid centered in the window.
/// Creates both vertical and horizontal lines with semi-transparent gray color.
fn spawn_grid(mut commands: Commands) {
    // Calculate grid position to center it
    let grid_size = 256.0;
    let grid_position = Vec2::new(0.0, 0.0); // Center of the window
    
    // Create vertical lines
    for i in 0..=256 {
        let x = grid_position.x - (grid_size / 2.0) + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                custom_size: Some(Vec2::new(1.0, grid_size)),
                ..default()
            },
            Transform::from_xyz(x * 4.0, grid_position.y, 0.0),
        ));
    }

    // Create horizontal lines
    for i in 0..=256 {
        let y = grid_position.y - (grid_size / 2.0) + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                custom_size: Some(Vec2::new(grid_size * 4.0, 1.0)),
                ..default()
            },
            Transform::from_xyz(grid_position.x, y * 4.0, 0.0),
        ));
    }
}