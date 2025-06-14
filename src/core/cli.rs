//! Command line interface for the application.
use clap::Parser;
use std::path::PathBuf;

/// A UFO and font design tool.
#[derive(Parser, Debug)]
#[clap(name = "bezy")]
pub struct CliArgs {
    /// The path to a UFO file to load.
    #[clap(long = "load-ufo")]
    pub ufo_path: Option<PathBuf>,
} 