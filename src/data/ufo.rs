//! UFO file I/O operations

use anyhow::Result;
use norad::Font;
use std::path::Path;
use std::collections::HashMap;
use crate::data::unicode::sort_and_deduplicate_codepoints;

/// Load a UFO font file from disk
pub fn load_ufo_from_path(path: impl AsRef<Path>) -> Result<Font> {
    let font = Font::load(path)?;
    Ok(font)
}

// Unicode Codepoint Mapping --------------------------------------------------

/// Convert a Unicode character to its hex codepoint string
fn char_to_hex_codepoint(unicode_char: char) -> String {
    format!("{:04X}", unicode_char as u32)
}

/// Add all codepoints from a glyph to the mapping
fn add_glyph_codepoints_to_map(
    map: &mut HashMap<String, String>,
    glyph: &norad::Glyph,
) {
    for &unicode_char in &glyph.codepoints {
        let codepoint_hex = char_to_hex_codepoint(unicode_char);
        map.insert(codepoint_hex, glyph.name().to_string());
    }
}

/// Build a map of Unicode codepoints to glyph names for efficient lookups
fn build_codepoint_glyph_map(font: &Font) -> HashMap<String, String> {
    let mut codepoint_to_glyph = HashMap::new();

    let layer = font.default_layer();

    for glyph in layer.iter() {
        add_glyph_codepoints_to_map(&mut codepoint_to_glyph, glyph);
    }
    codepoint_to_glyph
}

// Public Glyph Lookup API ---------------------------------------------------

/// Find a glyph by its Unicode codepoint (like "0041" for letter A)
pub fn find_glyph_by_unicode(
    font: &Font,
    codepoint_hex: &str
) -> Option<String> {
    build_codepoint_glyph_map(font).get(codepoint_hex).cloned()
}

/// Get all Unicode codepoints that have glyphs in this font
pub fn get_all_codepoints(font: &Font) -> Vec<String> {
    let map = build_codepoint_glyph_map(font);
    let mut codepoints: Vec<String> = map.keys().cloned().collect();
    sort_and_deduplicate_codepoints(&mut codepoints);

    codepoints
}

// Codepoint Navigation ------------------------------------------------------

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

/// Find the position of a codepoint in the list
fn find_codepoint_position(
    codepoints: &[String],
    target: &str
) -> Option<usize> {
    codepoints.iter().position(|codepoint| codepoint == target)
}

/// Cycle to the next or previous codepoint in the font
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
        // Current codepoint not found, start from the end based on direction
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

/// Move to the next codepoint in the font
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

/// Move to the previous codepoint in the font
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