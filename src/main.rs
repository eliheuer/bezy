//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bezy::core;
use clap::Parser;
use std::process;
use log::info;

fn main() {
    // Initialize logging first so we can see error messages
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    info!("Bezy starting up...");

    // Parse command line arguments - only on desktop
    #[cfg(not(target_arch = "wasm32"))]
    {
        let cli_args = core::cli::CliArgs::parse();
        info!("CLI args parsed successfully");

        // Create and run the application
        match core::app::create_app(cli_args) {
            Ok(mut app) => {
                info!("App created successfully, starting to run...");
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

    // For WASM, use default arguments
    #[cfg(target_arch = "wasm32")]
    {
        let cli_args = core::cli::CliArgs::default_for_web();

        // Create and run the application
        match core::app::create_app(cli_args) {
            Ok(mut app) => {
                app.run();
            }
            Err(error) => {
                web_sys::console::error_1(&format!("Error starting Bezy: {}", error).into());
            }
        }
    }
} 
