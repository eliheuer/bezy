//! UFO file I/O operations
//!
//! This module handles reading UFO font files and finding glyphs by their Unicode values.
//! UFO fonts store glyphs with names like "A", "uni0041", or "u0041" for the same character.

use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::core::state::AppState;
use crate::data::unicode::{
    parse_codepoint, 
    generate_glyph_name_variants, 
    sort_and_deduplicate_codepoints
};

/// Find a glyph by its Unicode codepoint (like "0041" for letter A)
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    // Convert hex string like "0041" to actual character 'A'
    let character = parse_codepoint(codepoint_hex)?;
    let layer = ufo.get_default_layer()?;

    // Try different naming patterns: "A", "uni0041", "u0041", etc.
    let possible_names = generate_glyph_name_variants(character);

    for glyph_name in possible_names {
        if let Some(glyph) = layer.get_glyph(&norad::GlyphName::from(glyph_name)) {
            if glyph_has_unicode_value(glyph, character) {
                return Some(glyph.name.to_string());
            }
        }
    }
    
    None
}

/// Get all Unicode codepoints that have glyphs in this font
/// 
/// This function efficiently finds codepoints by iterating through existing glyphs
/// instead of scanning Unicode ranges. This is much faster because:
/// - Only checks glyphs that actually exist in the font
/// - No need to scan thousands of potential Unicode codepoints
/// - Direct access to glyph codepoint data
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let Some(layer) = ufo.get_default_layer() else {
        return Vec::new();
    };

    let mut found_codepoints = Vec::new();

    // Iterate through actual glyphs in the font (much more efficient!)
    for glyph in layer.iter_contents() {
        if let Some(unicode_values) = &glyph.codepoints {
            for &unicode_char in unicode_values {
                let unicode_number = unicode_char as u32;
                let codepoint_hex = format!("{:04X}", unicode_number);
                found_codepoints.push(codepoint_hex);
            }
        }
    }

    // Remove duplicates and sort the list
    sort_and_deduplicate_codepoints(&mut found_codepoints);
    
    debug!("Found {} codepoints in font", found_codepoints.len());
    found_codepoints
}

/// Direction for cycling through codepoints
#[derive(Debug, Clone, Copy)]
pub enum CycleDirection {
    Next,
    Previous,
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
        return match direction {
            CycleDirection::Next => available_codepoints.first().cloned(),
            CycleDirection::Previous => available_codepoints.last().cloned(),
        };
    }

    // Find where we are in the list
    if let Some(current_position) = find_codepoint_position(available_codepoints, current_codepoint) {
        match direction {
            CycleDirection::Next => {
                let next_position = current_position + 1;
                if next_position < available_codepoints.len() {
                    // Move to next item
                    Some(available_codepoints[next_position].clone())
                } else {
                    // Wrap around to beginning
                    available_codepoints.first().cloned()
                }
            }
            CycleDirection::Previous => {
                if current_position > 0 {
                    // Move to previous item
                    Some(available_codepoints[current_position - 1].clone())
                } else {
                    // Wrap around to end
                    available_codepoints.last().cloned()
                }
            }
        }
    } else {
        // Current codepoint not found, start from appropriate end based on direction
        match direction {
            CycleDirection::Next => available_codepoints.first().cloned(),
            CycleDirection::Previous => available_codepoints.last().cloned(),
        }
    }
}

/// Move to the next codepoint in the font (wraps to beginning if at end)
pub fn find_next_codepoint_in_list(
    available_codepoints: &[String], 
    current_codepoint: &str
) -> Option<String> {
    cycle_codepoint_in_list(available_codepoints, current_codepoint, CycleDirection::Next)
}

/// Move to the previous codepoint in the font (wraps to end if at beginning)
pub fn find_previous_codepoint_in_list(
    available_codepoints: &[String], 
    current_codepoint: &str
) -> Option<String> {
    cycle_codepoint_in_list(available_codepoints, current_codepoint, CycleDirection::Previous)
}

/// Load a UFO font file from disk
pub fn load_ufo_from_path(path: &str) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    
    if !font_path.exists() {
        let error_msg = format!("File not found: {}", font_path.display());
        return Err(error_msg.into());
    }

    let ufo = Ufo::load(font_path)?;
    Ok(ufo)
}

/// Set up the font when the app starts (called by Bevy)
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

// Helper Functions (the building blocks used above) ---------------

/// Check if a glyph contains a specific Unicode character
fn glyph_has_unicode_value(glyph: &norad::Glyph, character: char) -> bool {
    match &glyph.codepoints {
        Some(unicode_values) => unicode_values.contains(&character),
        None => false,
    }
}

/// Find the position of a codepoint in the list
fn find_codepoint_position(codepoints: &[String], target: &str) -> Option<usize> {
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