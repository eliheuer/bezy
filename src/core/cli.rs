use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Command line arguments for font loading and glyph selection by Unicode codepoint
#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a UFO font file to load with a default source file
    #[arg(
        long = "load-ufo",
        default_value = "assets/fonts/bezy-grotesk-regular.ufo"
    )]
    pub ufo_path: Option<PathBuf>,

    /// What unicode codepoint to start the editor viewing 
    #[arg(long = "load-unicode", default_value = "0061")]
    pub load_unicode: Option<String>,

    /// Display debug information
    #[arg(long, default_value_t = false)]
    pub debug: bool,
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
