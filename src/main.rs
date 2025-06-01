//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! "The enjoyment of one's tools is an essential ingredient of successful work."
//! — Donald Knuth

mod core;
mod editing;
mod rendering;
mod ui;
mod geometry;
mod systems;
mod utils;
mod data;
use clap::Parser;

fn main() {
    let cli_args = core::cli::CliArgs::parse();
    core::app::create_app(cli_args).run();
}