//! A font editor made with Bevy, with inspiration from Runebender.

use bevy::prelude::*;
use norad::Font as Ufo;
use anyhow::Result;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_ufo();
    // UI camera
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new(get_basic_font_info()),
        TextFont {
            font: asset_server.load("fonts/SkynetGrotesk-RegularDisplay.ttf"),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

/// Loads the UFO font
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

/// Tries to load the UFO font 
fn try_load_ufo() -> Result<Ufo> {
    let path = "design-assets/test-fonts/test-font-001.ufo";
    let ufo = Ufo::load(path)?;
    Ok(ufo)
}

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