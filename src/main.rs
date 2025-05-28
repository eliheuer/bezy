//! A font editor made with Rust, the Bevy game engine, and some Linebender crates.

// "The enjoyment of one's tools is an essential ingredient of successful work."
// — Donald Knuth

mod core;
mod editing;
mod geometry;
mod io;
mod rendering;
mod systems;
mod ui;
mod utils;
use core::app::create_app;
use core::cli::CliArgs;

fn main() {
    let cli_args = CliArgs::parse_args();
    create_app(cli_args).run();
}
