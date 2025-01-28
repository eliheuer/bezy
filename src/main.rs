//! A font editor made with Bevy, with inspiration from Runebender.

use bevy::prelude::*;
use norad::Font;
use anyhow::Result;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, load_ufo_font)
        .run();
}

fn load_ufo_font() {
    match try_load_ufo() {
        Ok(font) => {
            let family_name = font.font_info.family_name.unwrap_or_default();
            let style_name = font.font_info.style_name.unwrap_or_default();
            println!("Successfully loaded UFO font: {} {}", family_name, style_name);
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

fn try_load_ufo() -> Result<Font> {
    let path = "design-assets/test-fonts/test-font-001.ufo";
    let font = Font::load(path)?;
    Ok(font)
}