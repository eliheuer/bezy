use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::data::AppState;

pub fn get_basic_font_info_from_state(app_state: &AppState) -> String {
    if app_state.workspace.font.ufo.font_info.is_some() {
        format!("UFO: {}", app_state.get_font_display_name())
    } else {
        "UFO: No font loaded".to_string()
    }
}

pub fn load_ufo_from_path(
    path: &str,
) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}

// System that initializes the font state
pub fn initialize_font_state(
    mut commands: Commands,
    cli_args: Res<crate::cli::CliArgs>,
) {
    // Check if a UFO path was provided via CLI
    if let Some(ufo_path) = &cli_args.ufo_path {
        // Load UFO file from the path provided via CLI
        match load_ufo_from_path(ufo_path.to_str().unwrap_or_default()) {
            Ok(ufo) => {
                let mut state = AppState::default();
                state.set_font(ufo, Some(ufo_path.clone()));
                let display_name = state.get_font_display_name();
                commands.insert_resource(state);
                info!("Loaded font: {}", display_name);
            }
            Err(e) => {
                error!("Failed to load UFO file: {}", e);
                commands.init_resource::<AppState>();
            }
        }
    } else {
        // No CLI argument provided, just initialize an empty state
        commands.init_resource::<AppState>();
    }
}
