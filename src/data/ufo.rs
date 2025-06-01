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

// Default font information values
const DEFAULT_FAMILY_NAME: &str = "Untitled";
const DEFAULT_STYLE_NAME: &str = "Regular";
const DEFAULT_UNITS_PER_EM: f64 = 1024.0;

// Common fallback glyphs to try when looking for test glyphs
const COMMON_TEST_GLYPHS: &[&str] = 
    &["H", "h", "A", "a", "n", "space", ".notdef"];

/// System resource to track the last printed codepoint for change detection
#[derive(Resource, Default)]
pub struct LastCodepointPrinted {
    pub codepoint: Option<String>,
}

/// Direction for finding adjacent codepoints in a list
enum Direction {
    Next,
    Previous,
}

// Glyph Finding Functions

/// Find a glyph by Unicode codepoint
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    let target_char = parse_codepoint(codepoint_hex)?;
    let default_layer = ufo.get_default_layer()?;

    if let Some(glyph_name) = 
        search_common_glyphs(&default_layer, target_char) 
    {
        return Some(glyph_name);
    }

    search_standard_glyph_names(&default_layer, target_char)
}

/// Get all available Unicode codepoints in the font
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let Some(default_layer) = ufo.get_default_layer() else {
        return Vec::new();
    };

    let unicode_ranges = get_common_unicode_ranges();
    let mut found_codepoints = Vec::new();

    for (start, end) in unicode_ranges {
        scan_unicode_range(
            default_layer, 
            start, 
            end, 
            &mut found_codepoints
        );
    }

    sort_and_deduplicate_codepoints(&mut found_codepoints);
    
    debug!("Found {} codepoints in font", found_codepoints.len());
    found_codepoints
}

/// Find the next codepoint in the font (wraps around to beginning)
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    find_adjacent_codepoint(&codepoints, current_hex, Direction::Next)
}

/// Find the previous codepoint in the font (wraps around to end)
pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    let codepoints = get_all_codepoints(ufo);
    find_adjacent_codepoint(&codepoints, current_hex, Direction::Previous)
}

// Font Information Functions

/// Get basic font information as a formatted string
pub fn get_basic_font_info_from_ufo(ufo: &Ufo) -> String {
    let Some(font_info) = &ufo.font_info else {
        return "No font info available".to_string();
    };

    let family_name = extract_family_name(font_info);
    let style_name = extract_style_name(font_info);
    let units_per_em = extract_units_per_em(font_info);

    format!("{} {} ({}upm)", family_name, style_name, units_per_em)
}

/// Get basic font information from app state
pub fn get_basic_font_info_from_state(app_state: &AppState) -> String {
    get_basic_font_info_from_ufo(&app_state.workspace.font.ufo)
}

// File Loading Functions

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
        load_font_from_cli_path(&mut commands, ufo_path);
    } else {
        commands.init_resource::<AppState>();
    }
}

// Console Output Functions

/// System to print font info and current codepoint when it changes
pub fn print_font_info_to_terminal(
    app_state: Res<AppState>,
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let current_codepoint = glyph_navigation.current_codepoint.clone();

    if last_printed.codepoint == current_codepoint {
        return;
    }

    print_font_information(&app_state);
    print_codepoint_information(&glyph_navigation);

    last_printed.codepoint = current_codepoint;
}

// Helper Functions

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
    let test_names = generate_glyph_name_variants(target_char);

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
    match &glyph.codepoints {
        Some(codepoints) => codepoints.contains(&target_char),
        None => false,
    }
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
            Direction::Next => Some(codepoints.first()?.clone()),
            Direction::Previous => Some(codepoints.last()?.clone()),
        };
    }

    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match direction {
        Direction::Next => find_next_in_list(codepoints, current_idx),
        Direction::Previous => find_previous_in_list(codepoints, current_idx),
    }
}

/// Find the next codepoint in the list (with wraparound)
fn find_next_in_list(
    codepoints: &[String], 
    current_idx: Option<usize>
) -> Option<String> {
    match current_idx {
        Some(idx) if idx < codepoints.len() - 1 => {
            Some(codepoints[idx + 1].clone())
        }
        Some(_) => {
            Some(codepoints.first()?.clone())
        }
        None => {
            Some(codepoints.first()?.clone())
        }
    }
}

/// Find the previous codepoint in the list (with wraparound)
fn find_previous_in_list(
    codepoints: &[String], 
    current_idx: Option<usize>
) -> Option<String> {
    match current_idx {
        Some(0) => {
            Some(codepoints.last()?.clone())
        }
        Some(idx) => {
            Some(codepoints[idx - 1].clone())
        }
        None => {
            Some(codepoints.last()?.clone())
        }
    }
}

/// Extract family name from font info with fallback
fn extract_family_name(font_info: &norad::FontInfo) -> &str {
    font_info
        .family_name
        .as_deref()
        .unwrap_or(DEFAULT_FAMILY_NAME)
}

/// Extract style name from font info with fallback
fn extract_style_name(font_info: &norad::FontInfo) -> &str {
    font_info
        .style_name
        .as_deref()
        .unwrap_or(DEFAULT_STYLE_NAME)
}

/// Extract units per em from font info with fallback
fn extract_units_per_em(font_info: &norad::FontInfo) -> f64 {
    font_info
        .units_per_em
        .map(|v| v.get() as f64)
        .unwrap_or(DEFAULT_UNITS_PER_EM)
}

/// Print basic font information to the console
fn print_font_information(app_state: &AppState) {
    let font_info = get_basic_font_info_from_state(app_state);
    info!("{}", font_info);
}

/// Print current codepoint information to the console
fn print_codepoint_information(
    glyph_navigation: &crate::core::state::GlyphNavigation
) {
    let Some(codepoint) = &glyph_navigation.current_codepoint else {
        info!("No specific codepoint selected");
        return;
    };

    if codepoint.is_empty() {
        info!("No specific codepoint selected");
        return;
    }

    match parse_codepoint_for_display(codepoint) {
        Ok(character) => print_character_info(codepoint, character),
        Err(_) => info!("Current codepoint: {} (invalid hex)", codepoint),
    }
}

/// Parse a codepoint string for display purposes
fn parse_codepoint_for_display(
    codepoint: &str
) -> Result<Option<char>, std::num::ParseIntError> {
    let code_val = u32::from_str_radix(codepoint, 16)?;
    let character = char::from_u32(code_val);
    Ok(character)
}

/// Print information about a Unicode character
fn print_character_info(codepoint: &str, character: Option<char>) {
    match character {
        Some(ch) if ch.is_control() => {
            info!("Current codepoint: U+{} (control character)", codepoint);
        }
        Some(ch) => {
            info!("Current codepoint: U+{} ('{}')", codepoint, ch);
        }
        None => {
            info!("Current codepoint: U+{} (invalid Unicode)", codepoint);
        }
    }
}

/// Load a font from the CLI-provided path
fn load_font_from_cli_path(commands: &mut Commands, ufo_path: &PathBuf) {
    let path_str = ufo_path.to_str().unwrap_or_default();
    
    match load_ufo_from_path(path_str) {
        Ok(ufo) => {
            create_and_insert_font_state(commands, ufo, ufo_path);
        }
        Err(e) => {
            error!("Failed to load UFO file: {}", e);
            commands.init_resource::<AppState>();
        }
    }
}

/// Create AppState with the loaded font and insert it as a resource
fn create_and_insert_font_state(
    commands: &mut Commands, 
    ufo: Ufo, 
    ufo_path: &PathBuf
) {
    let mut state = AppState::default();
    state.set_font(ufo, Some(ufo_path.clone()));
    let display_name = state.get_font_display_name();
    commands.insert_resource(state);
    info!("Loaded font: {}", display_name);
}

