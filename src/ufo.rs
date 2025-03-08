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
    
    // Get the default layer
    if let Some(_default_layer) = ufo.get_default_layer() {
        // We don't have direct access to a list of all glyphs in norad 0.3.1
        // Instead, we'll try comprehensive Unicode ranges and check for their existence
        
        // Comprehensive list of Unicode ranges covering major scripts
        let ranges = [
            // Basic Latin (ASCII)
            (0x0020, 0x007F),
            // Latin-1 Supplement
            (0x00A0, 0x00FF),
            // Latin Extended-A
            (0x0100, 0x017F),
            // Latin Extended-B
            (0x0180, 0x024F),
            // IPA Extensions
            (0x0250, 0x02AF),
            // Spacing Modifier Letters
            (0x02B0, 0x02FF),
            // Combining Diacritical Marks
            (0x0300, 0x036F),
            // Greek and Coptic
            (0x0370, 0x03FF),
            // Cyrillic
            (0x0400, 0x04FF),
            // Cyrillic Supplement
            (0x0500, 0x052F),
            // Armenian
            (0x0530, 0x058F),
            // Hebrew
            (0x0590, 0x05FF),
            // Arabic
            (0x0600, 0x06FF),
            // Arabic Supplement
            (0x0750, 0x077F),
            // Syriac
            (0x0700, 0x074F),
            // Thaana
            (0x0780, 0x07BF),
            // NKo
            (0x07C0, 0x07FF),
            // Samaritan
            (0x0800, 0x083F),
            // Mandaic
            (0x0840, 0x085F),
            // Devanagari
            (0x0900, 0x097F),
            // Bengali
            (0x0980, 0x09FF),
            // Gurmukhi
            (0x0A00, 0x0A7F),
            // Gujarati
            (0x0A80, 0x0AFF),
            // Oriya
            (0x0B00, 0x0B7F),
            // Tamil
            (0x0B80, 0x0BFF),
            // Telugu
            (0x0C00, 0x0C7F),
            // Kannada
            (0x0C80, 0x0CFF),
            // Malayalam
            (0x0D00, 0x0D7F),
            // Sinhala
            (0x0D80, 0x0DFF),
            // Thai
            (0x0E00, 0x0E7F),
            // Lao
            (0x0E80, 0x0EFF),
            // Tibetan
            (0x0F00, 0x0FFF),
            // Myanmar
            (0x1000, 0x109F),
            // Georgian
            (0x10A0, 0x10FF),
            // Hangul Jamo
            (0x1100, 0x11FF),
            // Ethiopic
            (0x1200, 0x137F),
            // Cherokee
            (0x13A0, 0x13FF),
            // Unified Canadian Aboriginal Syllabics
            (0x1400, 0x167F),
            // Ogham
            (0x1680, 0x169F),
            // Runic
            (0x16A0, 0x16FF),
            // General Punctuation
            (0x2000, 0x206F),
            // Superscripts and Subscripts
            (0x2070, 0x209F),
            // Currency Symbols
            (0x20A0, 0x20CF),
            // Letterlike Symbols
            (0x2100, 0x214F),
            // Number Forms
            (0x2150, 0x218F),
            // Arrows
            (0x2190, 0x21FF),
            // Mathematical Operators
            (0x2200, 0x22FF),
            // Miscellaneous Technical
            (0x2300, 0x23FF),
            // Box Drawing
            (0x2500, 0x257F),
            // Block Elements
            (0x2580, 0x259F),
            // Geometric Shapes
            (0x25A0, 0x25FF),
            // Miscellaneous Symbols
            (0x2600, 0x26FF),
            // Dingbats
            (0x2700, 0x27BF),
            // CJK Symbols and Punctuation
            (0x3000, 0x303F),
            // Hiragana
            (0x3040, 0x309F),
            // Katakana
            (0x30A0, 0x30FF),
            // Bopomofo
            (0x3100, 0x312F),
            // Hangul Compatibility Jamo
            (0x3130, 0x318F),
            // CJK Unified Ideographs (Basic sample range)
            (0x4E00, 0x4FFF),
            // Geometric Shapes Extended
            (0x1F780, 0x1F7FF),
            // Emoji-related ranges
            (0x1F300, 0x1F5FF), // Miscellaneous Symbols and Pictographs
            (0x1F600, 0x1F64F), // Emoticons
            (0x1F680, 0x1F6FF), // Transport and Map Symbols
            // Private Use Area (sample range for custom icons)
            (0xE000, 0xE0FF),
        ];
        
        // Check each range efficiently
        for (start, end) in ranges.iter() {
            // To keep compilation fast, only check a sample of each range first
            let step = (*end - *start) / 10 + 1; // Sample ~10 codepoints from each range
            for codepoint in (*start..=*end).step_by(step as usize) {
                let cp_hex = format!("{:04X}", codepoint);
                if find_glyph_by_unicode(ufo, &cp_hex).is_some() {
                    // If we find a match in the range, check every codepoint in that range
                    // This optimizes for fonts that include only certain script ranges
                    for detailed_cp in *start..=*end {
                        let detailed_hex = format!("{:04X}", detailed_cp);
                        if find_glyph_by_unicode(ufo, &detailed_hex).is_some() {
                            codepoints.push(detailed_hex);
                        }
                    }
                    break; // Once we've checked all codepoints in this range, move to next range
                }
            }
        }
        
        // Add some specific common characters that might be outside the ranges
        let specific_chars = [
            0x2122, // ™
            0x2713, // ✓
            0x2022, // •
            0x20AC, // €
            0x20BF, // ₿
            0x261D, // ☝
            0x270C, // ✌
            0x1F44D, // 👍
        ];
        
        for &codepoint in specific_chars.iter() {
            let cp_hex = format!("{:04X}", codepoint);
            if find_glyph_by_unicode(ufo, &cp_hex).is_some() {
                codepoints.push(cp_hex);
            }
        }
    }
    
    // If there are no codepoints found, fall back to a minimal set
    if codepoints.is_empty() {
        let fallback_codepoints = [
            "0020", // space
            "0041", // A
            "0061", // a
        ];
        
        for cp in fallback_codepoints.iter() {
            // Only add if the glyph exists
            if find_glyph_by_unicode(ufo, cp).is_some() {
                codepoints.push(cp.to_string());
            }
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
