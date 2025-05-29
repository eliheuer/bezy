use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

// Unicode character ranges as constants for readability
const LOWERCASE_A_TO_Z: std::ops::RangeInclusive<u32> = 0x0061..=0x007A; // a-z
const UPPERCASE_A_TO_Z: std::ops::RangeInclusive<u32> = 0x0041..=0x005A; // A-Z
const DIGITS_0_TO_9: std::ops::RangeInclusive<u32> = 0x0030..=0x0039;    // 0-9

// Common special character codes
const SPACE_CHAR: u32 = 0x0020;
const PERIOD_CHAR: u32 = 0x002E;
const COMMA_CHAR: u32 = 0x002C;
const QUOTE_CHAR: u32 = 0x0027;

/// Bezy font editor command line interface
#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a UFO font file to load
    #[arg(
        long = "load-ufo",
        default_value = "assets/fonts/bezy-grotesk-regular.ufo"
    )]
    pub ufo_path: Option<PathBuf>,

    /// Unicode codepoint to display for testing (in hexadecimal, e.g., 0048 for 'H')
    #[arg(long = "test-unicode", default_value = "0061")]
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

    /// Get the glyph name for the specified Unicode codepoint
    pub fn get_test_glyph(&self) -> String {
        match &self.test_unicode {
            Some(unicode_str) => self.convert_unicode_to_glyph_name(unicode_str),
            None => "H".to_string(),
        }
    }

    /// Convert a Unicode string to a proper glyph name
    fn convert_unicode_to_glyph_name(&self, unicode_str: &str) -> String {
        match self.parse_unicode_codepoint(unicode_str) {
            Ok(codepoint) => self.codepoint_to_glyph_name(codepoint),
            Err(_) => "H".to_string(), // Fallback to 'H' if parsing fails
        }
    }

    /// Parse Unicode string to codepoint number
    fn parse_unicode_codepoint(&self, unicode_str: &str) -> Result<u32, std::num::ParseIntError> {
        u32::from_str_radix(unicode_str.trim_start_matches("0x"), 16)
    }

    /// Convert a Unicode codepoint to its corresponding glyph name
    fn codepoint_to_glyph_name(&self, codepoint: u32) -> String {
        // Check if it's a basic Latin character that uses direct naming
        if self.is_basic_latin_character(codepoint) {
            if let Some(c) = char::from_u32(codepoint) {
                return c.to_string();
            }
        }

        // Handle special characters with custom names
        if let Some(special_name) = self.get_special_character_name(codepoint) {
            return special_name;
        }

        // Use standard "uni" + hex format for other characters
        format!("uni{:04X}", codepoint)
    }

    /// Check if codepoint is a basic Latin character (a-z, A-Z, 0-9)
    fn is_basic_latin_character(&self, codepoint: u32) -> bool {
        LOWERCASE_A_TO_Z.contains(&codepoint) ||
        UPPERCASE_A_TO_Z.contains(&codepoint) ||
        DIGITS_0_TO_9.contains(&codepoint)
    }

    /// Get special character names for common punctuation
    fn get_special_character_name(&self, codepoint: u32) -> Option<String> {
        match codepoint {
            SPACE_CHAR => Some("space".to_string()),
            PERIOD_CHAR => Some("period".to_string()),
            COMMA_CHAR => Some("comma".to_string()),
            QUOTE_CHAR => Some("quotesingle".to_string()),
            _ => None,
        }
    }

    /// Find a glyph in the UFO by the current codepoint
    pub fn find_glyph<'a>(&self, ufo: &'a norad::Ufo) -> Option<norad::GlyphName> {
        let default_layer = ufo.get_default_layer()?;

        // Try Unicode-based lookup first
        if let Some(glyph_name) = self.find_glyph_by_unicode(ufo) {
            if default_layer.get_glyph(&glyph_name).is_some() {
                return Some(glyph_name);
            }
        }

        // Fall back to glyph name lookup
        self.find_glyph_by_name(&default_layer)
    }

    /// Try to find glyph using Unicode codepoint lookup
    fn find_glyph_by_unicode(&self, ufo: &norad::Ufo) -> Option<norad::GlyphName> {
        let codepoint = self.test_unicode.as_ref()?;
        let glyph_name = crate::io::ufo::find_glyph_by_unicode(ufo, codepoint)?;
        Some(norad::GlyphName::from(glyph_name))
    }

    /// Try to find glyph using generated glyph name
    fn find_glyph_by_name(&self, layer: &norad::Layer) -> Option<norad::GlyphName> {
        let test_glyph = self.get_test_glyph();
        let glyph_name = norad::GlyphName::from(test_glyph);
        
        if layer.get_glyph(&glyph_name).is_some() {
            Some(glyph_name)
        } else {
            None
        }
    }

    /// Get the original codepoint string for display purposes
    pub fn get_codepoint_string(&self) -> String {
        self.test_unicode.clone().unwrap_or_default()
    }

    /// Set the unicode codepoint to a new value
    pub fn set_codepoint(&mut self, new_codepoint: String) {
        self.test_unicode = Some(new_codepoint);
        self.codepoint_found = false; // Reset so the glyph can be checked
    }
}
