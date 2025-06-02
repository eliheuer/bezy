//! UFO file I/O operations

use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::core::state::AppState;
use crate::data::unicode::sort_and_deduplicate_codepoints;

/// Convert a Unicode character to its hex codepoint string (like 'A' -> "0041")
fn char_to_hex_codepoint(unicode_char: char) -> String {
    format!("{:04X}", unicode_char as u32)
}

/// Add all codepoints from a glyph to the mapping
fn add_glyph_codepoints_to_map(
    map: &mut HashMap<String, String>, 
    glyph: &std::sync::Arc<norad::Glyph>
) {
    let Some(unicode_values) = &glyph.codepoints else {
        return;
    };

    for &unicode_char in unicode_values {
        let codepoint_hex = char_to_hex_codepoint(unicode_char);
        map.insert(codepoint_hex, glyph.name.to_string());
    }
}

/// Build a map of Unicode codepoints to glyph names for efficient lookups
fn build_codepoint_glyph_map(ufo: &Ufo) -> HashMap<String, String> {
    let mut codepoint_to_glyph = HashMap::new();
    
    let Some(layer) = ufo.get_default_layer() else {
        return codepoint_to_glyph;
    };

    for glyph in layer.iter_contents() {
        add_glyph_codepoints_to_map(&mut codepoint_to_glyph, &glyph);
    }
    codepoint_to_glyph
}

/// Find a glyph by its Unicode codepoint (like "0041" for letter A)
pub fn find_glyph_by_unicode(
    ufo: &Ufo, 
    codepoint_hex: &str
) -> Option<String> {
    build_codepoint_glyph_map(ufo).get(codepoint_hex).cloned()
}

/// Get all Unicode codepoints that have glyphs in this font
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let map = build_codepoint_glyph_map(ufo);
    let mut codepoints: Vec<String> = map.keys().cloned().collect();
    sort_and_deduplicate_codepoints(&mut codepoints);
    
    debug!("Found {} codepoints in font", codepoints.len());
    codepoints
}

/// Direction for cycling through codepoints
#[derive(Debug, Clone, Copy)]
pub enum CycleDirection {
    Next,
    Previous,
}

/// Get the appropriate starting position based on direction
fn get_direction_default(
    codepoints: &[String], 
    direction: CycleDirection
) -> Option<String> {
    match direction {
        CycleDirection::Next => codepoints.first().cloned(),
        CycleDirection::Previous => codepoints.last().cloned(),
    }
}

/// Cycle to the next or previous codepoint in the font (wraps around at boundaries)
pub fn cycle_codepoint_in_list(
    available_codepoints: &[String], 
    current_codepoint: &str,
    direction: CycleDirection,
) -> Option<String> {
    if available_codepoints.is_empty() {
        return None;
    }

    // If no current codepoint, start at appropriate end based on direction
    if current_codepoint.is_empty() {
        return get_direction_default(available_codepoints, direction);
    }

    // Find where we are in the list
    let current_position = find_codepoint_position(
        available_codepoints, 
        current_codepoint
    );
    
    let Some(current_position) = current_position else {
        // Current codepoint not found, start from appropriate end based on direction
        return get_direction_default(available_codepoints, direction);
    };

    match direction {
        CycleDirection::Next => {
            let next_position = current_position + 1;
            if next_position < available_codepoints.len() {
                Some(available_codepoints[next_position].clone())
            } else {
                // Wrap around to beginning
                available_codepoints.first().cloned()
            }
        }
        CycleDirection::Previous => {
            if current_position > 0 {
                Some(available_codepoints[current_position - 1].clone())
            } else {
                // Wrap around to end
                available_codepoints.last().cloned()
            }
        }
    }
}

/// Move to the next codepoint in the font (wraps to beginning if at end)
pub fn find_next_codepoint_in_list(
    available_codepoints: &[String], 
    current_codepoint: &str
) -> Option<String> {
    cycle_codepoint_in_list(
        available_codepoints, 
        current_codepoint, 
        CycleDirection::Next
    )
}

/// Move to the previous codepoint in the font (wraps to end if at beginning)
pub fn find_previous_codepoint_in_list(
    available_codepoints: &[String], 
    current_codepoint: &str
) -> Option<String> {
    cycle_codepoint_in_list(
        available_codepoints, 
        current_codepoint, 
        CycleDirection::Previous
    )
}

/// Load a UFO font file from disk
pub fn load_ufo_from_path(
    path: &str
) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    
    if !font_path.exists() {
        let error_msg = format!("File not found: {}", font_path.display());
        return Err(error_msg.into());
    }

    let ufo = Ufo::load(font_path)?;
    Ok(ufo)
}

/// Set up the font when the app starts
pub fn initialize_font_state(
    mut commands: Commands,
    cli_args: Res<crate::core::cli::CliArgs>,
) {
    if let Some(font_path) = &cli_args.ufo_path {
        load_font_at_startup(&mut commands, font_path);
    } else {
        // No font specified, start with empty state
        commands.init_resource::<AppState>();
    }
}

/// Find the position of a codepoint in the list
fn find_codepoint_position(
    codepoints: &[String], 
    target: &str
) -> Option<usize> {
    codepoints.iter().position(|codepoint| codepoint == target)
}

/// Load and set up a font when the app starts
fn load_font_at_startup(commands: &mut Commands, font_path: &PathBuf) {
    let path_string = font_path.to_str().unwrap_or_default();
    
    match load_ufo_from_path(path_string) {
        Ok(ufo) => {
            // Successfully loaded font
            let mut app_state = AppState::default();
            app_state.set_font(ufo, Some(font_path.clone()));
            let font_name = app_state.get_font_display_name();
            commands.insert_resource(app_state);
            info!("Loaded font: {}", font_name);
        }
        Err(error) => {
            // Failed to load font
            error!("Failed to load UFO file: {}", error);
            commands.init_resource::<AppState>();
        }
    }
}