//! Bevy Systems and Plugins
//!
//! This module contains Bevy-specific systems and plugin configurations:
//! - Plugin management and configuration
//! - Command handling for user actions
//! - UI interaction detection and processing

pub mod commands;
pub mod debug;
pub mod metrics;
pub mod plugins;
pub mod sort_interaction;
pub mod sort_manager;
pub mod text_editor_sorts;
pub mod ui_interaction;

// Re-export commonly used items
pub use commands::CommandsPlugin;
pub use plugins::{BezySystems, configure_default_plugins};
pub use ui_interaction::UiInteractionPlugin; 