use bevy::prelude::Resource;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

/// Bezy font editor command line interface
#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a UFO font file to load
    #[arg(long = "load-ufo")]
    pub ufo_path: Option<PathBuf>,

    /// Unicode codepoint to display for testing (in hexadecimal, e.g., 0048 for 'H')
    #[arg(long = "test-unicode")]
    pub test_unicode: Option<String>,

    /// Display debug information
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    /// Internal flag to track if the requested codepoint was found
    #[clap(skip)]
    pub codepoint_found: bool,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get the glyph name for the specified Unicode codepoint or a default
    pub fn get_test_glyph(&self) -> String {
        match &self.test_unicode {
            Some(unicode_str) => {
                // Try to parse the hex string
                match u32::from_str_radix(
                    unicode_str.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(codepoint) => {
                        // Use a lookup table for common special characters
                        let mut special_chars = HashMap::new();
                        special_chars.insert(0x0020, "space"); // Space
                        special_chars.insert(0x002E, "period"); // Period
                        special_chars.insert(0x002C, "comma"); // Comma
                        special_chars.insert(0x0027, "quotesingle"); // Single quote

                        // Check special characters first
                        if let Some(glyph_name) = special_chars.get(&codepoint)
                        {
                            return glyph_name.to_string();
                        }

                        // For basic Latin characters (a-z, A-Z, 0-9), use the character directly
                        if (0x0061..=0x007A).contains(&codepoint) ||  // a-z (lowercase)
                           (0x0041..=0x005A).contains(&codepoint) ||  // A-Z (uppercase)
                           (0x0030..=0x0039).contains(&codepoint)
                        {
                            // 0-9 (digits)
                            let c = char::from_u32(codepoint).unwrap_or('H');
                            return c.to_string();
                        }

                        // Try both naming conventions
                        // First, the literal name if it's a printable character
                        if let Some(c) = char::from_u32(codepoint) {
                            if c.is_ascii_graphic() {
                                return format!("{}|uni{:04X}", c, codepoint);
                            }
                        }

                        // For other Unicode codepoints, use the "uni" + codepoint naming convention
                        format!("uni{:04X}", codepoint)
                    }
                    Err(_) => "H".to_string(), // Default to 'H' if parsing fails
                }
            }
            None => "H".to_string(), // Default to 'H' if not specified
        }
    }

    /// Get the original codepoint string for display purposes
    pub fn get_codepoint_string(&self) -> String {
        match &self.test_unicode {
            Some(unicode_str) => unicode_str.clone(),
            None => "".to_string(),
        }
    }

    /// Set the unicode codepoint to a new value
    pub fn set_codepoint(&mut self, new_codepoint: String) {
        self.test_unicode = Some(new_codepoint);
        self.codepoint_found = false; // Reset so the glyph can be checked
    }
}
