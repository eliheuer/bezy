//! Rendering and Visualization
//!
//! This module contains all rendering and visualization functionality:
//! - Drawing systems for glyphs, paths, and UI elements
//! - Camera management for viewport control
//! - Background patterns and visual aids
//! - Debug visualization tools

#![allow(unused_imports)]

pub mod cameras;
pub mod checkerboard;
pub mod debug;
pub mod draw;
pub mod glyph_outline;
pub mod metrics;
pub mod sort_renderer;

// Re-export commonly used items
pub use checkerboard::CheckerboardPlugin;
