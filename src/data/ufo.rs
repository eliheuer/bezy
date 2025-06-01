//! UFO file I/O operations
//!
//! This module provides functions for working with UFO files and glyphs,
//! with a focus on Unicode codepoint mapping and font information display.

use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::core::state::AppState;

// Constants for Unicode ranges to scan
const BASIC_LATIN_START: u32 = 0x0020;
const BASIC_LATIN_END: u32 = 0x007F;
const LATIN_1_SUPPLEMENT_START: u32 = 0x00A0;
const LATIN_1_SUPPLEMENT_END: u32 = 0x00FF;
const LATIN_EXTENDED_A_START: u32 = 0x0100;
const LATIN_EXTENDED_A_END: u32 = 0x017F;
const LATIN_EXTENDED_B_START: u32 = 0x0180;
const LATIN_EXTENDED_B_END: u32 = 0x024F;
const CYRILLIC_START: u32 = 0x0400;
const CYRILLIC_END: u32 = 0x04FF;
const ARABIC_START: u32 = 0x0600;
const ARABIC_END: u32 = 0x06FF;

// Common fallback glyphs to try when looking for test glyphs
const COMMON_TEST_GLYPHS: &[&str] = 
    &["H", "h", "A", "a", "n", "space", ".notdef"];

/// System resource to track the last printed codepoint for change detection
#[derive(Resource, Default)]
pub struct LastCodepointPrinted {
    pub codepoint: Option<String>,
}

/// Find a glyph by Unicode codepoint using norad's built-in codepoints field
///
/// This function searches for a glyph that contains the specified Unicode
/// codepoint. It first tries common glyph names, then standard naming patterns.
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    let target_char = parse_codepoint(codepoint_hex)?;
    let default_layer = ufo.get_default_layer()?;

    // Try common glyphs first (faster than searching all possibilities)
    if let Some(glyph_name) = 
        search_common_glyphs(&default_layer, target_char) 
    {
        return Some(glyph_name);
    }

    // Try standard naming patterns
    search_standard_glyph_names(&default_layer, target_char)
}

/// Parse a hex codepoint string to a Unicode character
fn parse_codepoint(codepoint_hex: &str) -> Option<char> {
    u32::from_str_radix(codepoint_hex.trim_start_matches("0x"), 16)
        .ok()
        .and_then(char::from_u32)
}

/// Search for the target character in common glyph names
fn search_common_glyphs(
    default_layer: &norad::Layer, 
    target_char: char
) -> Option<String> {
    for glyph_name_str in COMMON_TEST_GLYPHS {
        let name = norad::GlyphName::from(*glyph_name_str);
        
        if let Some(glyph) = default_layer.get_glyph(&name) {
            if glyph_contains_codepoint(&glyph, target_char) {
                return Some(glyph.name.to_string());
            }
        }
    }
    None
}

/// Search for the target character using standard glyph naming patterns
fn search_standard_glyph_names(
    default_layer: &norad::Layer, 
    target_char: char
) -> Option<String> {
    let codepoint_value = target_char as u32;
    let test_names = [
        target_char.to_string(),
        format!("uni{:04X}", codepoint_value),
        format!("u{:04X}", codepoint_value),
    ];

    for name_str in test_names.iter() {
        let glyph_name = norad::GlyphName::from(name_str.clone());
        
        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
            if glyph_contains_codepoint(&glyph, target_char) {
                return Some(glyph.name.to_string());
            }
        }
    }
    None
}

/// Check if a glyph contains the specified Unicode codepoint
fn glyph_contains_codepoint(glyph: &norad::Glyph, target_char: char) -> bool {
    glyph.codepoints
        .as_ref()
        .map(|codepoints| codepoints.contains(&target_char))
        .unwrap_or(false)
}

/// Get all Unicode codepoints available in the font
///
/// This scans common Unicode ranges to find which codepoints have glyphs.
/// Note: Limited by norad 0.3.1's API - we can't iterate all glyphs directly.
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let Some(default_layer) = ufo.get_default_layer() else {
        return Vec::new();
    };

    let unicode_ranges = get_unicode_ranges_to_scan();
    let mut found_codepoints = Vec::new();

    for (start, end) in unicode_ranges {
        scan_unicode_range(default_layer, start, end, &mut found_codepoints);
    }

    sort_and_deduplicate_codepoints(&mut found_codepoints);
    
    debug!("Found {} codepoints in font", found_codepoints.len());
    found_codepoints
}

/// Get the list of Unicode ranges to scan for available glyphs
fn get_unicode_ranges_to_scan() -> Vec<(u32, u32)> {
    vec![
        (BASIC_LATIN_START, BASIC_LATIN_END),
        (LATIN_1_SUPPLEMENT_START, LATIN_1_SUPPLEMENT_END),
        (LATIN_EXTENDED_A_START, LATIN_EXTENDED_A_END),
        (LATIN_EXTENDED_B_START, LATIN_EXTENDED_B_END),
        (CYRILLIC_START, CYRILLIC_END),
        (ARABIC_START, ARABIC_END),
    ]
}

/// Scan a specific Unicode range for available glyphs
fn scan_unicode_range(
    default_layer: &norad::Layer,
    start: u32,
    end: u32,
    found_codepoints: &mut Vec<String>,
) {
    for code in start..=end {
        if let Some(target_char) = char::from_u32(code) {
            if check_glyph_exists_for_codepoint(default_layer, target_char) {
                let codepoint_hex = format!("{:04X}", code);
                if !found_codepoints.contains(&codepoint_hex) {
                    found_codepoints.push(codepoint_hex);
                }
            }
        }
    }
}

/// Check if any glyph exists for the given Unicode codepoint
fn check_glyph_exists_for_codepoint(
    default_layer: &norad::Layer, 
    target_char: char
) -> bool {
    let test_names = generate_glyph_name_variants(target_char);

    for name_str in test_names.iter() {
        let glyph_name = norad::GlyphName::from(name_str.clone());
        
        if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
            if glyph_contains_codepoint(&glyph, target_char) {
                return true;
            }
        }
    }
    false
}

/// Generate common glyph name variants for a Unicode character
fn generate_glyph_name_variants(target_char: char) -> [String; 3] {
    let code = target_char as u32;
    [
        target_char.to_string(),
        format!("uni{:04X}", code),
        format!("u{:04X}", code),
    ]
}

/// Sort codepoints numerically and remove duplicates
fn sort_and_deduplicate_codepoints(codepoints: &mut Vec<String>) {
    codepoints.sort_by(|a, b| {
        let a_val = u32::from_str_radix(a, 16).unwrap_or(0);
        let b_val = u32::from_str_radix(b, 16).unwrap_or(0);
        a_val.cmp(&b_val)
    });
    codepoints.dedup();
}

/// Find the next codepoint in the font (in ascending order)
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    find_adjacent_codepoint(&codepoints, current_hex, Direction::Next)
}

/// Find the previous codepoint in the font (in descending order)
pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    find_adjacent_codepoint(&codepoints, current_hex, Direction::Previous)
}

/// Direction for finding adjacent codepoints
enum Direction {
    Next,
    Previous,
}

/// Find the next or previous codepoint in a sorted list
fn find_adjacent_codepoint(
    codepoints: &[String], 
    current_hex: &str, 
    direction: Direction
) -> Option<String> {
    if codepoints.is_empty() {
        return None;
    }

    if current_hex.is_empty() {
        return match direction {
            Direction::Next => Some(codepoints[0].clone()),
            Direction::Previous => Some(codepoints[codepoints.len() - 1].clone()),
        };
    }

    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match (current_idx, direction) {
        (Some(idx), Direction::Next) if idx < codepoints.len() - 1 => {
            Some(codepoints[idx + 1].clone())
        }
        (Some(_), Direction::Next) => {
            Some(codepoints[0].clone()) // wrap around
        }
        (Some(0), Direction::Previous) => {
            Some(codepoints[codepoints.len() - 1].clone()) // wrap to end
        }
        (Some(idx), Direction::Previous) => {
            Some(codepoints[idx - 1].clone())
        }
        (None, Direction::Next) => {
            Some(codepoints[0].clone()) // not found, start from beginning
        }
        (None, Direction::Previous) => {
            Some(codepoints[codepoints.len() - 1].clone()) // not found, start from end
        }
    }
}

/// Get basic font information as a formatted string
pub fn get_basic_font_info_from_ufo(ufo: &Ufo) -> String {
    let Some(font_info) = &ufo.font_info else {
        return "No font info available".to_string();
    };

    let family_name = font_info
        .family_name
        .as_deref()
        .unwrap_or("Unknown");

    let style_name = font_info
        .style_name
        .as_deref()
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
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let current_codepoint = glyph_navigation.current_codepoint.clone();

    // Only print if the codepoint has changed
    if last_printed.codepoint == current_codepoint {
        return;
    }

    print_font_information(&app_state);
    print_codepoint_information(&glyph_navigation);

    // Update the last printed state
    last_printed.codepoint = current_codepoint;
}

/// Print basic font information to the console
fn print_font_information(app_state: &AppState) {
    let font_info = get_basic_font_info_from_state(app_state);
    info!("{}", font_info);
}

/// Print current codepoint information to the console
fn print_codepoint_information(glyph_navigation: &crate::core::state::GlyphNavigation) {
    let Some(codepoint) = &glyph_navigation.current_codepoint else {
        info!("No specific codepoint selected");
        return;
    };

    if codepoint.is_empty() {
        info!("No specific codepoint selected");
        return;
    }

    match parse_codepoint_for_display(codepoint) {
        Ok((code_val, character)) => {
            print_character_info(codepoint, code_val, character);
        }
        Err(_) => {
            info!("Current codepoint: {} (invalid hex)", codepoint);
        }
    }
}

/// Parse a codepoint string for display purposes
fn parse_codepoint_for_display(codepoint: &str) -> Result<(u32, Option<char>), std::num::ParseIntError> {
    let code_val = u32::from_str_radix(codepoint, 16)?;
    let character = char::from_u32(code_val);
    Ok((code_val, character))
}

/// Print information about a Unicode character
fn print_character_info(codepoint: &str, code_val: u32, character: Option<char>) {
    match character {
        Some(ch) if ch.is_control() => {
            info!(
                "Current codepoint: U+{} (control character)",
                codepoint
            );
        }
        Some(ch) => {
            info!("Current codepoint: U+{} ('{}')", codepoint, ch);
        }
        None => {
            info!(
                "Current codepoint: U+{} (invalid Unicode)",
                codepoint
            );
        }
    }
}

/// Load a UFO file from the specified path
pub fn load_ufo_from_path(path: &str) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}

/// System that initializes the font state from CLI arguments
pub fn initialize_font_state(
    mut commands: Commands,
    cli_args: Res<crate::core::cli::CliArgs>,
) {
    if let Some(ufo_path) = &cli_args.ufo_path {
        load_font_from_cli_path(&mut commands, ufo_path);
    } else {
        // No CLI argument provided, initialize empty state
        commands.init_resource::<AppState>();
    }
}

/// Load a font from the CLI-provided path
fn load_font_from_cli_path(commands: &mut Commands, ufo_path: &PathBuf) {
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
}

/// Scan the font for all available glyph names (for debugging)
/// 
/// This function helps understand the naming conventions used in a font
/// by trying to find glyphs in common Unicode ranges.
#[allow(dead_code)]
pub fn dump_all_glyph_names(ufo: &Ufo) -> Vec<String> {
    let Some(default_layer) = ufo.get_default_layer() else {
        return Vec::new();
    };

    let mut glyph_names = Vec::new();

    // Scan basic Latin ranges (most commonly present)
    scan_latin_glyphs(default_layer, &mut glyph_names);
    
    // Scan Arabic range with uni naming pattern
    scan_arabic_glyphs(default_layer, &mut glyph_names);

    // Sort and analyze the results
    glyph_names.sort();
    log_glyph_analysis(&glyph_names);

    glyph_names
}

/// Scan for Latin alphabet glyphs (A-Z, a-z)
fn scan_latin_glyphs(default_layer: &norad::Layer, glyph_names: &mut Vec<String>) {
    // Uppercase A-Z
    for cp in 0x0041..=0x005A {
        check_and_add_glyph(default_layer, cp, glyph_names);
    }

    // Lowercase a-z  
    for cp in 0x0061..=0x007A {
        check_and_add_glyph(default_layer, cp, glyph_names);
    }
}

/// Scan for Arabic glyphs using uni naming pattern
fn scan_arabic_glyphs(default_layer: &norad::Layer, glyph_names: &mut Vec<String>) {
    for cp in 0x0600..=0x0650 {
        let uni_name = format!("uni{:04X}", cp);
        let glyph_name = norad::GlyphName::from(uni_name);
        
        if default_layer.get_glyph(&glyph_name).is_some() {
            glyph_names.push(glyph_name.to_string());
        }
    }
}

/// Check if a glyph exists for a codepoint and add it to the list
fn check_and_add_glyph(
    default_layer: &norad::Layer, 
    codepoint: u32, 
    glyph_names: &mut Vec<String>
) {
    if let Some(character) = char::from_u32(codepoint) {
        let char_name = character.to_string();
        let glyph_name = norad::GlyphName::from(char_name);
        
        if default_layer.get_glyph(&glyph_name).is_some() {
            glyph_names.push(glyph_name.to_string());
        }
    }
}

/// Log analysis of found glyph names
fn log_glyph_analysis(glyph_names: &[String]) {
    info!("Found {} glyph names in font", glyph_names.len());

    // Look for Arabic-related names
    let arabic_names: Vec<_> = glyph_names
        .iter()
        .filter(|name| {
            name.contains("arab") 
                || name.contains("alef") 
                || name.contains("beh")
                || name.starts_with("uni06")
        })
        .collect();

    if arabic_names.is_empty() {
        info!("No Arabic-related glyph names found in font");
    } else {
        info!("Found {} Arabic-related glyph names:", arabic_names.len());
        for name in arabic_names {
            info!("  - {}", name);
        }
    }
}
