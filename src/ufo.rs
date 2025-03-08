use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::data::AppState;

// Resource to track the last printed codepoint information
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct LastCodepointPrinted {
    pub codepoint: Option<String>,
}

pub fn get_basic_font_info_from_state(app_state: &AppState) -> String {
    if app_state.workspace.font.ufo.font_info.is_some() {
        format!("UFO: {}", app_state.get_font_display_name())
    } else {
        "UFO: No font loaded".to_string()
    }
}

// System to print font info and codepoint to terminal
pub fn print_font_info_to_terminal(
    app_state: Res<AppState>,
    cli_args: Res<crate::cli::CliArgs>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let font_info = get_basic_font_info_from_state(&app_state);
    let mut display_text = font_info;
    let current_codepoint = cli_args.test_unicode.clone();

    // Check if we need to print (startup or codepoint changed)
    let should_print = last_printed.codepoint != current_codepoint;

    if should_print {
        // Add codepoint info if present
        if let Some(codepoint) = &cli_args.test_unicode {
            if !codepoint.is_empty() {
                // Try to get a readable character representation
                let cp_value = match u32::from_str_radix(
                    codepoint.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(value) => value,
                    Err(_) => 0,
                };

                let char_display = match char::from_u32(cp_value) {
                    Some(c) if c.is_control() => format!("<control>"),
                    Some(c) => format!("'{}'", c),
                    None => format!("<invalid>"),
                };

                display_text = format!(
                    "{}\nCodepoint: {} {}",
                    display_text, codepoint, char_display
                );

                // Verify codepoint exists in the font directly
                let codepoint_exists =
                    if app_state.workspace.font.ufo.font_info.is_some() {
                        find_glyph_by_unicode(
                            &app_state.workspace.font.ufo,
                            codepoint,
                        )
                        .is_some()
                    } else {
                        false
                    };

                if !codepoint_exists {
                    error!("Codepoint {} not found in UFO source", codepoint);
                } else {
                    info!("Codepoint {} found in font", codepoint);
                }
            }
        }

        // Print to terminal
        info!("{}", display_text);

        // Update last printed state
        last_printed.codepoint = current_codepoint;
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

    // Convert the U32 codepoint to a Rust char
    let target_char = match char::from_u32(codepoint) {
        Some(c) => c,
        None => return None,
    };

    // Get the default layer
    if let Some(default_layer) = ufo.get_default_layer() {
        // For Latin lowercase a-z (0061-007A), uppercase A-Z (0041-005A),
        // and common punctuation, try using the character itself as name
        if (0x0061..=0x007A).contains(&codepoint) || // a-z
           (0x0041..=0x005A).contains(&codepoint)
        {
            // A-Z
            // Try the character name
            let glyph_name = norad::GlyphName::from(target_char.to_string());
            if let Some(_glyph) = default_layer.get_glyph(&glyph_name) {
                // Found a match!
                return Some(glyph_name.to_string());
            }
        }

        // Try conventional format "uni<CODE>"
        let uni_name = format!("uni{:04X}", codepoint);
        let glyph_name = norad::GlyphName::from(uni_name);
        if let Some(_) = default_layer.get_glyph(&glyph_name) {
            return Some(glyph_name.to_string());
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
                if let Some(_) = default_layer.get_glyph(&glyph_name) {
                    return Some(glyph_name.to_string());
                }
            }
        }
    }

    // If we got here, we didn't find a matching glyph
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
