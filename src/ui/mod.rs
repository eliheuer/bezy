//! User Interface Components
//!
//! This module contains all user interface related functionality:
//! - Toolbars for different editing modes and access controls
//! - Panes for displaying and editing glyph information
//! - HUD elements and overlays
//! - Theme and styling definitions
//! - Text editing interfaces

pub mod hud;
pub mod panes;
pub mod text_editor;
pub mod theme;
pub mod toolbars;

// Re-export commonly used items
pub use theme::BACKGROUND_COLOR;
pub use hud::*;
pub use text_editor::TextEditorPlugin; 