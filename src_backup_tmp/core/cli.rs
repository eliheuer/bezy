//! Command line arguments for the application

use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// command line arguments for font loading and unicode default setting
#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// path to a ufo font file to load with a default source file
    #[arg(
        long = "load-ufo",
        default_value = "assets/fonts/bezy-grotesk-regular.ufo"
    )]
    pub ufo_path: Option<PathBuf>,

    /// what unicode codepoint to start the editor viewing
    #[arg(long = "load-unicode", default_value = "0061")]
    pub load_unicode: String,

    /// display debug information
    #[arg(long, default_value_t = false)]
    pub debug: bool,
}
