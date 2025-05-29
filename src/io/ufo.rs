use anyhow::Result;
use bevy::prelude::*;
use norad::Ufo;
use std::path::PathBuf;

use crate::core::data::AppState;

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
    cli_args: Res<crate::core::cli::CliArgs>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let font_info = get_basic_font_info_from_state(&app_state);
    let current_codepoint = cli_args.load_unicode.clone();

    // Check if we need to print (startup or codepoint changed)
    let should_print = last_printed.codepoint != current_codepoint;

    if should_print {
        // Log the basic font info
        info!("{}", font_info);

        // Add codepoint info if present
        if let Some(codepoint) = &cli_args.load_unicode {
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

                // Log the codepoint info separately
                info!("Codepoint: {} {}", codepoint, char_display);

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

        // Update last printed codepoint
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

    // STRATEGY 1: Try common naming conventions for glyphs

    // Get the default layer
    if let Some(default_layer) = ufo.get_default_layer() {
        // 1. Try direct character name (works for Latin and some other scripts)
        let char_name = target_char.to_string();
        let glyph_name = norad::GlyphName::from(char_name.clone());
        if let Some(_glyph) = default_layer.get_glyph(&glyph_name) {
            return Some(glyph_name.to_string());
        }

        // 2. Try standard "uni<CODE>" format (used by many fonts)
        let uni_name = format!("uni{:04X}", codepoint);
        let glyph_name = norad::GlyphName::from(uni_name.clone());
        if let Some(_) = default_layer.get_glyph(&glyph_name) {
            return Some(glyph_name.to_string());
        }

        // 3. Try "u<CODE>" format (alternative naming used in some fonts)
        let u_name = format!("u{:04X}", codepoint);
        let glyph_name = norad::GlyphName::from(u_name.clone());
        if let Some(_) = default_layer.get_glyph(&glyph_name) {
            return Some(glyph_name.to_string());
        }

        // 4. Try Arabic-specific patterns for common characters
        if (0x0600..=0x06FF).contains(&codepoint) {
            // Arabic basic glyph names (without diacritics)
            let arabic_common_names = match codepoint {
                0x0627 => vec!["alef", "arabic.alef"],
                0x0628 => vec!["beh", "arabic.beh"],
                0x062A => vec!["teh", "arabic.teh"],
                0x062B => vec!["theh", "arabic.theh"],
                0x062C => vec!["jeem", "arabic.jeem"],
                0x062D => vec!["hah", "arabic.hah"],
                0x062E => vec!["khah", "arabic.khah"],
                0x062F => vec!["dal", "arabic.dal"],
                0x0630 => vec!["thal", "arabic.thal"],
                0x0631 => vec!["reh", "arabic.reh"],
                0x0632 => vec!["zain", "arabic.zain"],
                0x0633 => vec!["seen", "arabic.seen"],
                0x0634 => vec!["sheen", "arabic.sheen"],
                0x0635 => vec!["sad", "arabic.sad"],
                0x0636 => vec!["dad", "arabic.dad"],
                0x0637 => vec!["tah", "arabic.tah"],
                0x0638 => vec!["zah", "arabic.zah"],
                0x0639 => vec!["ain", "arabic.ain"],
                0x063A => vec!["ghain", "arabic.ghain"],
                0x0641 => vec!["feh", "arabic.feh"],
                0x0642 => vec!["qaf", "arabic.qaf"],
                0x0643 => vec!["kaf", "arabic.kaf"],
                0x0644 => vec!["lam", "arabic.lam"],
                0x0645 => vec!["meem", "arabic.meem"],
                0x0646 => vec!["noon", "arabic.noon"],
                0x0647 => vec!["heh", "arabic.heh"],
                0x0648 => vec!["waw", "arabic.waw"],
                0x0649 => vec!["alefMaksura", "arabic.alefMaksura"],
                0x064A => vec!["yeh", "arabic.yeh"],
                _ => vec![],
            };

            for name in arabic_common_names {
                let glyph_name = norad::GlyphName::from(name);
                if let Some(_) = default_layer.get_glyph(&glyph_name) {
                    return Some(glyph_name.to_string());
                }
            }
        }

        // 5. Special cases for common characters
        let special_cases = [
            (0x0020, "space"),
            (0x002E, "period"),
            (0x002C, "comma"),
            (0x0027, "quotesingle"),
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

/// Get ALL Unicode codepoints in the font, sorted in numeric order
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    let mut codepoints = Vec::new();

    // Get the default layer
    if let Some(_default_layer) = ufo.get_default_layer() {
        info!("Extracting all codepoints from font...");

        // APPROACH 1: First check glyphs using direct character names
        // This works for most Latin and other scripts where glyph names match characters

        // Check common Unicode ranges
        let ranges = [
            // Basic Latin (ASCII)
            (0x0020, 0x007E),
            // Latin-1 Supplement
            (0x00A0, 0x00FF),
            // Latin Extended-A
            (0x0100, 0x017F),
            // Latin Extended-B
            (0x0180, 0x024F),
            // IPA Extensions
            (0x0250, 0x02AF),
            // Greek and Coptic
            (0x0370, 0x03FF),
            // Cyrillic
            (0x0400, 0x04FF),
            // Arabic - explicitly check a wider range
            (0x0600, 0x077F),
            // Common symbols and punctuation
            (0x2000, 0x206F), // General Punctuation
            (0x2070, 0x209F), // Superscripts and Subscripts
            (0x20A0, 0x20CF), // Currency Symbols
            (0x2100, 0x214F), // Letterlike Symbols
            (0x2150, 0x218F), // Number Forms
            (0x2190, 0x21FF), // Arrows
            (0x2200, 0x22FF), // Mathematical Operators
        ];

        // Search for codepoints in each range
        for (start, end) in ranges.iter() {
            info!("Checking range U+{:04X} to U+{:04X}", start, end);
            for code in *start..=*end {
                check_and_add_codepoint(ufo, code, &mut codepoints);
            }
        }

        // APPROACH 2: Check for 'uni' prefixed glyph names
        // Many fonts use naming pattern like uni0041 for Unicode code points
        if let Some(default_layer) = ufo.get_default_layer() {
            // Try the most common uni-prefixed scheme
            for code in 0x0000..=0xFFFF {
                let uni_name = format!("uni{:04X}", code);
                let glyph_name = norad::GlyphName::from(uni_name);
                if default_layer.get_glyph(&glyph_name).is_some() {
                    let cp_hex = format!("{:04X}", code);
                    if !codepoints.contains(&cp_hex) {
                        codepoints.push(cp_hex);
                    }
                }
            }

            // Try the 'u' prefix scheme (also common)
            for code in 0x0000..=0xFFFF {
                let u_name = format!("u{:04X}", code);
                let glyph_name = norad::GlyphName::from(u_name);
                if default_layer.get_glyph(&glyph_name).is_some() {
                    let cp_hex = format!("{:04X}", code);
                    if !codepoints.contains(&cp_hex) {
                        codepoints.push(cp_hex);
                    }
                }
            }
        }
    }

    // Sort codepoints numerically
    codepoints.sort_by(|a, b| {
        let a_val = u32::from_str_radix(a, 16).unwrap_or(0);
        let b_val = u32::from_str_radix(b, 16).unwrap_or(0);
        a_val.cmp(&b_val)
    });

    // Remove duplicates (just in case)
    codepoints.dedup();

    // Log what we found
    info!("Found {} codepoints in font", codepoints.len());

    if !codepoints.is_empty() {
        // Show a sample of what we found
        let sample_size = std::cmp::min(10, codepoints.len());
        let mut sample = String::new();

        // First few
        for i in 0..sample_size / 2 {
            sample.push_str(&format!("U+{} ", codepoints[i]));
        }

        // Middle
        if codepoints.len() > sample_size {
            sample.push_str("... ");
        }

        // Last few
        for i in
            codepoints.len().saturating_sub(sample_size / 2)..codepoints.len()
        {
            sample.push_str(&format!("U+{} ", codepoints[i]));
        }

        info!("Codepoint sample: {}", sample);
    }

    codepoints
}

/// Helper function to check for a codepoint and add it if found
fn check_and_add_codepoint(ufo: &Ufo, code: u32, codepoints: &mut Vec<String>) {
    let cp_hex = format!("{:04X}", code);
    if find_glyph_by_unicode(ufo, &cp_hex).is_some() {
        codepoints.push(cp_hex);
    }
}

/// Find the next codepoint in the font (in ascending order)
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    // Get all available codepoints
    let codepoints = get_all_codepoints(ufo);

    // Early exit if no codepoints
    if codepoints.is_empty() {
        info!("No codepoints found in font");
        return None;
    }

    // If current codepoint is empty, start with the first one
    if current_hex.is_empty() {
        info!(
            "Starting with first codepoint: U+{} (decimal: {})",
            codepoints[0],
            u32::from_str_radix(&codepoints[0], 16).unwrap_or(0)
        );
        return Some(codepoints[0].clone());
    }

    // Find current codepoint position
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match current_idx {
        // Found current codepoint - return next one (or first if at end)
        Some(idx) if idx < codepoints.len() - 1 => {
            let next = &codepoints[idx + 1];
            info!(
                "Moving from codepoint U+{} to U+{} (decimal: {})",
                current_hex,
                next,
                u32::from_str_radix(next, 16).unwrap_or(0)
            );
            Some(next.clone())
        }
        // At the end - wrap around to first
        Some(_) => {
            info!(
                "Wrapping around from codepoint U+{} to U+{} (decimal: {})",
                current_hex,
                codepoints[0],
                u32::from_str_radix(&codepoints[0], 16).unwrap_or(0)
            );
            Some(codepoints[0].clone())
        }
        // Current codepoint not found - start from first
        None => {
            info!("Current codepoint U+{} not found, starting from U+{} (decimal: {})", 
                  current_hex, codepoints[0],
                  u32::from_str_radix(&codepoints[0], 16).unwrap_or(0));
            Some(codepoints[0].clone())
        }
    }
}

/// Find the previous codepoint in the font (in descending order)
pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    // Get all available codepoints
    let codepoints = get_all_codepoints(ufo);

    // Early exit if no codepoints
    if codepoints.is_empty() {
        info!("No codepoints found in font");
        return None;
    }

    // If current codepoint is empty, start with the last one
    if current_hex.is_empty() {
        let last_idx = codepoints.len() - 1;
        info!(
            "Starting with last codepoint: U+{} (decimal: {})",
            codepoints[last_idx],
            u32::from_str_radix(&codepoints[last_idx], 16).unwrap_or(0)
        );
        return Some(codepoints[last_idx].clone());
    }

    // Find current codepoint position
    let current_idx = codepoints.iter().position(|cp| cp == current_hex);

    match current_idx {
        // Found current codepoint - return previous one (or last if at beginning)
        Some(0) => {
            // At the beginning - wrap around to last
            let last_idx = codepoints.len() - 1;
            info!(
                "Wrapping around from codepoint U+{} to U+{} (decimal: {})",
                current_hex,
                codepoints[last_idx],
                u32::from_str_radix(&codepoints[last_idx], 16).unwrap_or(0)
            );
            Some(codepoints[last_idx].clone())
        }
        Some(idx) => {
            // Move to previous
            let prev = &codepoints[idx - 1];
            info!(
                "Moving from codepoint U+{} to U+{} (decimal: {})",
                current_hex,
                prev,
                u32::from_str_radix(prev, 16).unwrap_or(0)
            );
            Some(prev.clone())
        }
        // Current codepoint not found - start from last
        None => {
            let last_idx = codepoints.len() - 1;
            info!("Current codepoint U+{} not found, starting from U+{} (decimal: {})", 
                  current_hex, codepoints[last_idx],
                  u32::from_str_radix(&codepoints[last_idx], 16).unwrap_or(0));
            Some(codepoints[last_idx].clone())
        }
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
