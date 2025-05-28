//! Editing Functionality
//!
//! This module contains all editing-related functionality:
//! - Edit sessions for managing editing state
//! - Edit types and mode definitions
//! - Selection management for points, paths, and objects
//! - Undo/redo system for reversible operations

pub mod edit_session;
pub mod edit_type;
pub mod selection;
pub mod undo;
pub mod undo_plugin;

// Re-export commonly used items
pub use edit_session::EditSessionPlugin;
pub use selection::SelectionPlugin;
pub use undo_plugin::UndoPlugin;
