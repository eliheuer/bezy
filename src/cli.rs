use bevy::prelude::Resource;
use clap::{Parser, ValueEnum};
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
                match u32::from_str_radix(unicode_str.trim_start_matches("0x"), 16) {
                    Ok(codepoint) => {
                        // Convert to a char if it's a valid Unicode codepoint
                        match char::from_u32(codepoint) {
                            Some(c) => c.to_string(),
                            None => "H".to_string() // Default to 'H' if invalid
                        }
                    },
                    Err(_) => "H".to_string() // Default to 'H' if parsing fails
                }
            },
            None => "H".to_string() // Default to 'H' if not specified
        }
    }
} 