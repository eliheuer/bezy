//! Command line interface for the application.

use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

use crate::core::settings::DEFAULT_UFO_PATH;

/// A UFO and font design tool.
#[derive(Parser, Debug, Resource)]
#[clap(name = "bezy")]
pub struct CliArgs {
    /// The path to a UFO file to load.
    #[clap(long = "load-ufo", default_value = DEFAULT_UFO_PATH)]
    pub ufo_path: Option<PathBuf>,
} 