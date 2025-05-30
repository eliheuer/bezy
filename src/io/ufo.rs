//! UFO file I/O operations
//!
//! This module provides functions for working with UFO files and glyphs,
//! with a focus on Unicode codepoint mapping and font information display.

use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::core::data::AppState;

/// System resource to track the last printed codepoint for change detection
#[derive(Resource, Default)]
pub struct LastCodepointPrinted {
    pub codepoint: Option<String>,
}

/// Find a glyph by Unicode codepoint using norad's built-in codepoints field
/// 
/// This is much simpler than the previous approach - norad already handles
/// the Unicode mapping for us through the <unicode hex="..."/> elements in the UFO.
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    // Parse the hex codepoint string to a char
    let codepoint = u32::from_str_radix(codepoint_hex.trim_start_matches("0x"), 16)
        .ok()
        .and_then(char::from_u32)?;

    // Get the default layer and search through all glyphs
    let default_layer = ufo.get_default_layer()?;
    
    // Try common glyph names first as a fallback
    let common_glyphs = ["H", "h", "A", "a", "n", "space", ".notdef"];
    for glyph_name_str in common_glyphs.iter() {
        let name = norad::GlyphName::from(*glyph_name_str);
        if let Some(glyph) = default_layer.get_glyph(&name) {
            // Check if this glyph contains our target codepoint
            if let Some(ref codepoints) = glyph.codepoints {
                if codepoints.contains(&codepoint) {
                    return Some(glyph.name.to_string());
                }
            }
        }
    }
    
    // Try the standard naming patterns
    let test_names = [
        codepoint.to_string(),
        format!("uni{:04X}", codepoint as u32),
        format!("u{:04X}", codepoint as u32),
    ];
    
    for name_str in test_names.iter() {
        let glyph_name = norad::GlyphName::from(name_str.clone());
        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
            // Check if this glyph contains our target codepoint
            if let Some(ref codepoints) = glyph.codepoints {
                if codepoints.contains(&codepoint) {
                    return Some(glyph.name.to_string());
                }
            }
        }
    }
    
    None
}

/// Get all Unicode codepoints available in the font
/// 
/// This replaces the complex scanning logic with a simple approach using norad's codepoints.
/// Note: This is a simplified version due to norad 0.3.1 API limitations.
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let mut codepoints = Vec::new();
    
    if let Some(default_layer) = ufo.get_default_layer() {
        // Try common glyphs to see what's available
        // This is limited by norad 0.3.1's API - we can't iterate all glyphs
        let ranges = [
            (0x0020, 0x007F), // Basic Latin
            (0x00A0, 0x00FF), // Latin-1 Supplement
            (0x0100, 0x017F), // Latin Extended-A
            (0x0180, 0x024F), // Latin Extended-B
            (0x0400, 0x04FF), // Cyrillic
            (0x0600, 0x06FF), // Arabic
        ];
        
        for (start, end) in ranges.iter() {
            for code in *start..=*end {
                if let Some(target_char) = char::from_u32(code) {
                    // Check a few common naming patterns
                    let test_names = [
                        target_char.to_string(),
                        format!("uni{:04X}", code),
                        format!("u{:04X}", code),
                    ];
                    
                    for name_str in test_names.iter() {
                        let glyph_name = norad::GlyphName::from(name_str.clone());
                        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                            if let Some(ref glyph_codepoints) = glyph.codepoints {
                                if glyph_codepoints.contains(&target_char) {
                                    let cp_hex = format!("{:04X}", code);
                                    if !codepoints.contains(&cp_hex) {
                                        codepoints.push(cp_hex);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Sort and deduplicate
    codepoints.sort_by(|a, b| {
        let a_val = u32::from_str_radix(a, 16).unwrap_or(0);
        let b_val = u32::from_str_radix(b, 16).unwrap_or(0);
        a_val.cmp(&b_val)
    });
    codepoints.dedup();
    
    info!("Found {} codepoints in font", codepoints.len());
    codepoints
}

/// Find the next codepoint in the font (in ascending order)
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    
    if codepoints.is_empty() {
        return None;
    }
    
    if current_hex.is_empty() {
        return Some(codepoints[0].clone());
    }
    
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);
    
    match current_idx {
        Some(idx) if idx < codepoints.len() - 1 => Some(codepoints[idx + 1].clone()),
        Some(_) => Some(codepoints[0].clone()), // wrap around
        None => Some(codepoints[0].clone()),    // not found, start from beginning
    }
}

/// Find the previous codepoint in the font (in descending order)
pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    
    if codepoints.is_empty() {
        return None;
    }
    
    if current_hex.is_empty() {
        return Some(codepoints[codepoints.len() - 1].clone());
    }
    
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);
    
    match current_idx {
        Some(0) => Some(codepoints[codepoints.len() - 1].clone()), // wrap around to end
        Some(idx) => Some(codepoints[idx - 1].clone()),
        None => Some(codepoints[codepoints.len() - 1].clone()),    // not found, start from end
    }
}

/// Get basic font information as a formatted string
pub fn get_basic_font_info_from_ufo(ufo: &Ufo) -> String {
    let font_info = match &ufo.font_info {
        Some(info) => info,
        None => return "No font info available".to_string(),
    };

    let family_name = font_info
        .family_name
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Unknown");

    let style_name = font_info
        .style_name
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Regular");

    let units_per_em = font_info
        .units_per_em
        .map(|v| v.get() as f64)
        .unwrap_or(1000.0);

    format!("{} {} ({}upm)", family_name, style_name, units_per_em)
}

/// Get basic font information from app state
pub fn get_basic_font_info_from_state(app_state: &AppState) -> String {
    get_basic_font_info_from_ufo(&app_state.workspace.font.ufo)
}

/// System to print font info and current codepoint to terminal when it changes
pub fn print_font_info_to_terminal(
    app_state: Res<AppState>,
    glyph_navigation: Res<crate::core::data::GlyphNavigation>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let font_info = get_basic_font_info_from_state(&app_state);
    let current_codepoint = glyph_navigation.current_codepoint.clone();

    // Check if we need to print (startup or codepoint changed)
    let should_print = last_printed.codepoint != current_codepoint;

    if should_print {
        // Log the basic font info
        info!("{}", font_info);

        // Add codepoint info if present
        if let Some(codepoint) = &glyph_navigation.current_codepoint {
            if !codepoint.is_empty() {
                // Try to get a readable character representation
                if let Ok(code_val) = u32::from_str_radix(codepoint, 16) {
                    if let Some(character) = char::from_u32(code_val) {
                        if character.is_control() {
                            info!("Current codepoint: U+{} (control character)", codepoint);
                        } else {
                            info!("Current codepoint: U+{} ('{}')", codepoint, character);
                        }
                    } else {
                        info!("Current codepoint: U+{} (invalid Unicode)", codepoint);
                    }
                } else {
                    info!("Current codepoint: {} (invalid hex)", codepoint);
                }
            }
        } else {
            info!("No specific codepoint selected");
        }

        // Update the last printed state
        last_printed.codepoint = current_codepoint;
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
    cli_args: Res<crate::core::cli::CliArgs>,
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

/// Directly scan the font for all available glyph names
/// This is useful for debugging to understand the naming conventions used in the font
pub fn dump_all_glyph_names(ufo: &Ufo) -> Vec<String> {
    let mut glyph_names = Vec::new();

    if let Some(default_layer) = ufo.get_default_layer() {
        // Since we don't have Layer::iter(), use the knowledge of the UFO format
        // UFO glyphs are stored in a map/dictionary, but we can't directly iterate it
        // Try common glyphs to see what naming pattern the font uses

        // Try basic Latin ranges - these are almost always present
        for cp in 0x0041..=0x005A {
            // A-Z
            let char_name = char::from_u32(cp).unwrap().to_string();
            let glyph_name = norad::GlyphName::from(char_name);
            if default_layer.get_glyph(&glyph_name).is_some() {
                glyph_names.push(glyph_name.to_string());
            }
        }

        for cp in 0x0061..=0x007A {
            // a-z
            let char_name = char::from_u32(cp).unwrap().to_string();
            let glyph_name = norad::GlyphName::from(char_name);
            if default_layer.get_glyph(&glyph_name).is_some() {
                glyph_names.push(glyph_name.to_string());
            }
        }

        // Try Arabic range
        for cp in 0x0600..=0x0650 {
            // Sample of Arabic
            // Try uni format
            let uni_name = format!("uni{:04X}", cp);
            let glyph_name = norad::GlyphName::from(uni_name);
            if default_layer.get_glyph(&glyph_name).is_some() {
                glyph_names.push(glyph_name.to_string());
            }
        }

        // Sort names for easier reading
        glyph_names.sort();

        // Log the names
        info!("Found {} glyph names in font", glyph_names.len());

        // Look for Arabic-related names
        let mut arabic_names = Vec::new();
        for name in &glyph_names {
            if name.contains("arab")
                || name.contains("alef")
                || name.contains("beh")
                || name.starts_with("uni06")
            {
                arabic_names.push(name.clone());
            }
        }

        if !arabic_names.is_empty() {
            info!("Found {} Arabic-related glyph names:", arabic_names.len());
            for name in &arabic_names {
                info!("  - {}", name);
            }
        } else {
            info!("No Arabic-related glyph names found in font");
        }
    }

    glyph_names
}
