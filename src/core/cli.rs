use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

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

    /// Get the glyph name for the specified Unicode codepoint or a default
    ///
    /// This returns a string following standard glyph naming conventions.
    /// The actual lookup in the font is done by the ufo::find_glyph_by_unicode function.
    pub fn get_test_glyph(&self) -> String {
        match &self.test_unicode {
            Some(unicode_str) => {
                match u32::from_str_radix(
                    unicode_str.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(codepoint) => {
                        // Basic Latin characters use their direct character name
                        if (0x0061..=0x007A).contains(&codepoint) ||  // a-z
                           (0x0041..=0x005A).contains(&codepoint) ||  // A-Z
                           (0x0030..=0x0039).contains(&codepoint)
                        // 0-9
                        {
                            if let Some(c) = char::from_u32(codepoint) {
                                return c.to_string();
                            }
                        }

                        // Common special characters
                        match codepoint {
                            0x0020 => return "space".to_string(), // Space
                            0x002E => return "period".to_string(), // Period
                            0x002C => return "comma".to_string(), // Comma
                            0x0027 => return "quotesingle".to_string(), // Single quote
                            _ => {}
                        }

                        // Standard "uni" + hex code format for other Unicode points
                        format!("uni{:04X}", codepoint)
                    }
                    Err(_) => "H".to_string(), // Default to 'H' if parsing fails
                }
            }
            None => "H".to_string(), // Default to 'H' if not specified
        }
    }

    /// Find a glyph in the UFO by the current codepoint
    ///
    /// This is a helper that tries two approaches:
    /// 1. First, use the more sophisticated ufo::find_glyph_by_unicode
    /// 2. If that fails, fall back to the basic get_test_glyph
    ///
    /// Returns the glyph name if found
    pub fn find_glyph<'a>(
        &self,
        ufo: &'a norad::Ufo,
    ) -> Option<norad::GlyphName> {
        // Get the default layer
        let default_layer = ufo.get_default_layer()?;

        // Try to find the glyph by directly searching for Unicode value
        if let Some(codepoint) = &self.test_unicode {
            if let Some(glyph_name) =
                crate::io::ufo::find_glyph_by_unicode(ufo, codepoint)
            {
                let name = norad::GlyphName::from(glyph_name);
                if default_layer.get_glyph(&name).is_some() {
                    return Some(name);
                }
            }
        }

        // Fall back to get_test_glyph if needed
        let test_glyph = self.get_test_glyph();
        let glyph_name = norad::GlyphName::from(test_glyph);

        if default_layer.get_glyph(&glyph_name).is_some() {
            Some(glyph_name)
        } else {
            None
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
