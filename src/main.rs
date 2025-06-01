//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! "The enjoyment of one's tools is an essential ingredient of successful work."
//! — Donald Knuth

mod core;
mod data;
mod editing;
mod geometry;
mod rendering;
mod systems;
mod ui;
mod utils;
use clap::Parser;

fn main() {
    let cli_args = core::cli::CliArgs::parse();
    core::app::create_app(cli_args).run();
}
