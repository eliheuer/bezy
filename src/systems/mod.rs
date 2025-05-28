//! Bevy Systems and Plugins
//!
//! This module contains Bevy-specific systems and plugin configurations:
//! - Plugin management and configuration
//! - Command handling for user actions
//! - UI interaction detection and processing

pub mod commands;
pub mod plugins;
pub mod ui_interaction;

// Re-export commonly used items
pub use commands::CommandsPlugin;
pub use ui_interaction::UiInteractionPlugin;
