//! Core application functionality
//!
//! This module contains the core application logic, including:
//! - Application initialization and configuration
//! - State management
//! - Settings and CLI handling
//! - Cursor and coordinate management
//! - Input system

pub mod app;
pub mod cli;
pub mod cursor;
pub mod errors;
pub mod input;
pub mod settings;
pub mod state;

// Re-export commonly used items
pub use app::create_app;
pub use cli::CliArgs;
pub use cursor::{CursorInfo, CursorPlugin};
pub use input::{InputPlugin, InputState, InputEvent, InputMode, InputConsumer, helpers as input_helpers};
pub use settings::BezySettings;
pub use state::{AppState, GlyphNavigation, TextEditorState};