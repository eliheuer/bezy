//! Bevy Systems and Plugins
//!
//! This module contains Bevy-specific systems and plugin configurations:
//! - Plugin management and configuration
//! - Command handling for user actions
//! - UI interaction detection and processing
//! - Input consumer system

#![allow(unused_imports)]

pub mod commands;
pub mod fontir_lifecycle;
pub mod input_consumer;
pub mod lifecycle;
pub mod plugins;
pub mod sort_manager;
pub mod text_editor_sorts;
pub mod text_shaping;
pub mod ui_interaction;

// Re-export commonly used items
pub use commands::CommandsPlugin;
pub use fontir_lifecycle::{load_fontir_font, create_default_sort, center_camera_on_default_sort};
pub use input_consumer::InputConsumerPlugin;
pub use lifecycle::{exit_on_esc, load_ufo_font};
pub use plugins::{configure_default_plugins, BezySystems};
pub use text_shaping::TextShapingPlugin;
pub use ui_interaction::UiInteractionPlugin;
