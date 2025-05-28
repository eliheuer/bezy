//! Core application functionality
//!
//! This module contains the fundamental components of the Bezy font editor:
//! - Application initialization and configuration
//! - Core data structures and state management
//! - Settings and configuration management
//! - Command-line interface handling

pub mod app;
pub mod cli;
pub mod data;
pub mod settings;

// Re-export commonly used items for convenience
pub use app::create_app;
pub use cli::CliArgs;
pub use data::AppState; 