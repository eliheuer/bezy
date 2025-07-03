//! User interface modules for the Bezy font editor
//!
//! This module contains all UI-related components, systems, and plugins:
//! - Theme and styling definitions
//! - UI panes (design space, glyph navigation, etc.)
//! - Toolbars and controls
//! - Text editing components
//! - Head-up display (HUD) management

pub mod hud;
pub mod panes;
pub mod text_editor;
pub mod theme;
pub mod toolbars;

// Re-export commonly used items
#[allow(unused_imports)]
pub use hud::{HudPlugin, spawn_hud};
#[allow(unused_imports)]
pub use text_editor::TextEditorPlugin;
