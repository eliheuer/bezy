use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::env;
use std::path::PathBuf;

use crate::data::AppState;

pub fn load_ufo() {
    match load_ufo_from_args() {
        Ok(ufo) => {
            let family_name = ufo.font_info.as_ref()
                .and_then(|info| info.family_name.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_else(String::default);
            let style_name = ufo.font_info.as_ref()
                .and_then(|info| info.style_name.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_else(String::default);
            println!(
                "Successfully loaded UFO font: {} {}",
                family_name, style_name
            );
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

pub fn load_ufo_from_args() -> Result<Ufo, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err("Usage: program <path-to-ufo-file>".into());
    }

    let font_path = PathBuf::from(&args[1]);
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}

pub fn get_basic_font_info() -> String {
    match load_ufo_from_args() {
        Ok(ufo) => {
            let family_name = ufo.font_info.as_ref()
                .and_then(|info| info.family_name.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_else(String::default);
            let style_name = ufo.font_info.as_ref()
                .and_then(|info| info.style_name.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_else(String::default);
            format!("UFO: {} {}", family_name, style_name)
        }
        Err(e) => format!("UFO: Error loading font: {:?}", e),
    }
}

pub fn load_ufo_from_path(path: &str) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}

#[allow(dead_code)]
fn load_and_set_font(state: &mut AppState, arg: &str) {
    let path = PathBuf::from(arg);
    if let Ok(ufo) = load_ufo_from_path(arg) {
        state.set_font(ufo, Some(path));
        info!("Loaded font: {}", state.get_font_display_name());
    } else {
        error!("Failed to load UFO file at {}", arg);
    }
}

#[allow(dead_code)]
pub fn try_load_ufo_from_args(mut state: ResMut<AppState>) {
    if let Some(arg) = env::args().nth(1) {
        if let Ok(ufo) = load_ufo_from_path(&arg) {
            state.set_font(ufo, Some(PathBuf::from(&arg)));
            info!("Loaded font: {}", state.get_font_display_name());
        } else {
            error!("Failed to load UFO file at {}", arg);
        }
    }
}

// System that initializes the font state
pub fn initialize_font_state(mut commands: Commands, cli_args: Res<crate::cli::CliArgs>) {
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
        // No CLI argument, check environment args for backward compatibility
        if let Some(arg) = env::args().nth(1) {
            match load_ufo_from_path(&arg) {
                Ok(ufo) => {
                    let mut state = AppState::default();
                    state.set_font(ufo, Some(PathBuf::from(arg)));
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
            commands.init_resource::<AppState>();
        }
    }
}
