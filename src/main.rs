//! A font editor built with Rust, the Bevy game engine, and Linebender crates.

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

fn main() {
    let cli_args = core::cli::CliArgs::parse_args();
    core::app::create_app(cli_args).run();
}

