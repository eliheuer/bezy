//! UI Panes
//!
//! This module contains the various pane components that make up the main
//! editing interface of the Bezy font editor:
//! - Coordinate pane for precise positioning
//! - Glyph pane for glyph visualization and editing
//! - Design space for overall font design management

pub mod coord_pane;
pub mod design_space;
pub mod glyph_pane;

// Re-export plugins for easy access
pub use design_space::DesignSpacePlugin;
