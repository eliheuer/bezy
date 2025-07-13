//! Core application functionality
//!
//! This module contains the core application logic, including:
//! - Application initialization and configuration
//! - State management
//! - Settings and CLI handling
//! - Pointer and coordinate management
//! - Input system

pub mod app;
pub mod cli;
pub mod errors;
pub mod input;
pub mod pointer;
pub mod settings;
pub mod state;

// Re-export commonly used items
pub use app::create_app;
pub use cli::CliArgs;
pub use input::{helpers, InputEvent, InputState};
pub use pointer::{PointerInfo, PointerPlugin};
pub use settings::BezySettings;
pub use state::{AppState, GlyphNavigation};
