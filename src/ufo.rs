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

/// Find a glyph by its Unicode codepoint hex value (e.g., "0061")
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    // Parse the hex value
    let codepoint =
        match u32::from_str_radix(codepoint_hex.trim_start_matches("0x"), 16) {
            Ok(cp) => cp,
            Err(_) => return None,
        };

    // Try direct character name first (for basic Latin characters)
    if let Some(c) = char::from_u32(codepoint) {
        let char_name = c.to_string();
        let glyph_name = norad::GlyphName::from(char_name);

        if let Some(default_layer) = ufo.get_default_layer() {
            if default_layer.get_glyph(&glyph_name).is_some() {
                return Some(glyph_name.to_string());
            }
        }
    }

    // Try with "uni" prefix
    let uni_name = format!("uni{:04X}", codepoint);
    let glyph_name = norad::GlyphName::from(uni_name);

    if let Some(default_layer) = ufo.get_default_layer() {
        if default_layer.get_glyph(&glyph_name).is_some() {
            return Some(glyph_name.to_string());
        }
    }

    // Special cases for common characters
    let special_cases = [
        (0x0020, "space"),       // Space
        (0x002E, "period"),      // Period
        (0x002C, "comma"),       // Comma
        (0x0027, "quotesingle"), // Single quote
    ];

    for (cp, name) in special_cases.iter() {
        if *cp == codepoint {
            let glyph_name = norad::GlyphName::from(*name);
            if let Some(default_layer) = ufo.get_default_layer() {
                if default_layer.get_glyph(&glyph_name).is_some() {
                    return Some(glyph_name.to_string());
                }
            }
        }
    }

    None
}

/// Get all Unicode codepoints in the font, sorted in ascending order
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let mut codepoints = Vec::new();

    // For now, we'll use a simplified approach with some common codepoints
    // This can be expanded later to properly extract all codepoints from the UFO
    let common_codepoints = [
        "0020", // space
        "0021", // !
        "0041", // A
        "0042", // B
        "0043", // C
        "0061", // a
        "0062", // b
        "0063", // c
    ];

    for cp in common_codepoints.iter() {
        // Only add codepoints that exist in the font
        if find_glyph_by_unicode(ufo, cp).is_some() {
            codepoints.push(cp.to_string());
        }
    }

    // Sort codepoints
    codepoints.sort();
    codepoints
}

/// Find the next codepoint in the font (in ascending order)
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    if codepoints.is_empty() {
        return None;
    }

    // If current_hex is empty or not found, return the first codepoint
    if current_hex.is_empty() {
        return Some(codepoints[0].clone());
    }

    // Find the position of the current codepoint
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match current_idx {
        Some(idx) if idx < codepoints.len() - 1 => {
            Some(codepoints[idx + 1].clone())
        }
        Some(_) => Some(codepoints[0].clone()), // Wrap around to the first codepoint
        None => Some(codepoints[0].clone()), // Current not found, start from the beginning
    }
}

/// Find the previous codepoint in the font (in descending order)
pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    if codepoints.is_empty() {
        return None;
    }

    // If current_hex is empty or not found, return the last codepoint
    if current_hex.is_empty() {
        return Some(codepoints[codepoints.len() - 1].clone());
    }

    // Find the position of the current codepoint
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match current_idx {
        Some(0) => Some(codepoints[codepoints.len() - 1].clone()), // Wrap around to the last codepoint
        Some(idx) => Some(codepoints[idx - 1].clone()),
        None => Some(codepoints[codepoints.len() - 1].clone()), // Current not found, start from the end
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
