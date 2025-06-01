//! UFO file I/O operations
//!
//! This module provides functions for working with UFO files and glyphs,
//! with a focus on Unicode codepoint mapping and font information display.

use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::core::state::AppState;
use crate::data::unicode::{
    get_common_unicode_ranges, 
    parse_codepoint, 
    generate_glyph_name_variants, 
    sort_and_deduplicate_codepoints
};

// Glyph Finding Functions ------------------------------------

/// Find a glyph by Unicode codepoint
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    let target_char = parse_codepoint(codepoint_hex)?;
    let default_layer = ufo.get_default_layer()?;

    // Search using standard glyph naming patterns
    let test_names = generate_glyph_name_variants(target_char);

    for name_str in test_names.iter() {
        let glyph_name = norad::GlyphName::from(name_str.clone());
        
        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
            if let Some(codepoints) = &glyph.codepoints {
                if codepoints.contains(&target_char) {
                    return Some(glyph.name.to_string());
                }
            }
        }
    }
    None
}

/// Get all available Unicode codepoints in the font
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let Some(default_layer) = ufo.get_default_layer() else {
        return Vec::new();
    };

    let unicode_ranges = get_common_unicode_ranges();
    let mut found_codepoints = Vec::new();

    for (start, end) in unicode_ranges {
        for code in start..=end {
            if let Some(target_char) = char::from_u32(code) {
                let test_names = generate_glyph_name_variants(target_char);

                // Check if any glyph exists for this codepoint
                let exists = test_names.iter().any(|name_str| {
                    let glyph_name = norad::GlyphName::from(name_str.clone());
                    if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                        if let Some(codepoints) = &glyph.codepoints {
                            return codepoints.contains(&target_char);
                        }
                    }
                    false
                });

                if exists {
                    let codepoint_hex = format!("{:04X}", code);
                    if !found_codepoints.contains(&codepoint_hex) {
                        found_codepoints.push(codepoint_hex);
                    }
                }
            }
        }
    }

    sort_and_deduplicate_codepoints(&mut found_codepoints);
    
    debug!("Found {} codepoints in font", found_codepoints.len());
    found_codepoints
}

/// Find the next codepoint in a pre-computed list (wraps around to beginning)
pub fn find_next_codepoint_in_list(
    codepoints: &[String], 
    current_hex: &str
) -> Option<String> {
    if codepoints.is_empty() {
        return None;
    }

    if current_hex.is_empty() {
        return Some(codepoints.first()?.clone());
    }

    if let Some(current_idx) = codepoints.iter().position(|cp| cp == current_hex) {
        if current_idx < codepoints.len() - 1 {
            Some(codepoints[current_idx + 1].clone())
        } else {
            Some(codepoints.first()?.clone()) // wrap around
        }
    } else {
        Some(codepoints.first()?.clone()) // not found, start from beginning
    }
}

/// Find the previous codepoint in a pre-computed list (wraps around to end)
pub fn find_previous_codepoint_in_list(
    codepoints: &[String], 
    current_hex: &str
) -> Option<String> {
    if codepoints.is_empty() {
        return None;
    }

    if current_hex.is_empty() {
        return Some(codepoints.last()?.clone());
    }

    if let Some(current_idx) = codepoints.iter().position(|cp| cp == current_hex) {
        if current_idx > 0 {
            Some(codepoints[current_idx - 1].clone())
        } else {
            Some(codepoints.last()?.clone()) // wrap around
        }
    } else {
        Some(codepoints.last()?.clone()) // not found, start from end
    }
}

// File Loading Functions -------------------------------------

/// Load a UFO file from the specified path
pub fn load_ufo_from_path(path: &str) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    
    if !font_path.exists() {
        let error_msg = format!("File not found: {}", font_path.display());
        return Err(error_msg.into());
    }

    let ufo = Ufo::load(font_path)?;
    Ok(ufo)
}

/// System that initializes the font state from CLI arguments
pub fn initialize_font_state(
    mut commands: Commands,
    cli_args: Res<crate::core::cli::CliArgs>,
) {
    if let Some(ufo_path) = &cli_args.ufo_path {
        let path_str = ufo_path.to_str().unwrap_or_default();
        
        match load_ufo_from_path(path_str) {
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
        commands.init_resource::<AppState>();
    }
}

