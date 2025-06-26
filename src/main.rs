//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
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
use std::process;

fn main() {

    // Initialize logging first so we can see error messages
    env_logger::init();
    
    // Parse command line arguments
    let cli_args = core::cli::CliArgs::parse();
    
    // Create and run the application
    match core::app::create_app(cli_args) {
        Ok(mut app) => {
            app.run();
        }
        Err(error) => {
            eprintln!();
            eprintln!("Error starting Bezy:");
            eprintln!("{}", error);
            eprintln!();
            eprintln!("Try running with --help for usage information.");
            eprintln!("Or visit: https://bezy.org");
            process::exit(1);
        }
    }
} 
