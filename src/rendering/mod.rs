//! Rendering and Visualization
//!
//! This module contains all rendering and visualization functionality:
//! - Drawing systems for glyphs, paths, and UI elements
//! - Camera management for viewport control
//! - Background patterns and visual aids
//! - Debug visualization tools
//! - Selection visualization (marquee, selected points, handles)

#![allow(unused_imports)]

pub mod cameras;
pub mod checkerboard;
pub mod debug;
pub mod draw;
pub mod fontir_glyph_outline;
pub mod glyph_outline;
pub mod metrics;
pub mod point_backgrounds;
pub mod selection;
pub mod sort_renderer;
pub mod sort_visuals;

// Re-export commonly used items
pub use checkerboard::{CheckerboardEnabled, CheckerboardPlugin};
pub use point_backgrounds::PointBackgroundPlugin;
pub use selection::{
    render_all_point_entities, render_control_handles,
    render_selected_entities, render_selection_marquee,
};
