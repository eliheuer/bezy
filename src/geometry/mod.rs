//! Geometric Primitives and Operations
//!
//! This module contains the geometric building blocks of the font editor:
//! - Point definitions and operations
//! - Path representations and manipulations
//! - Component handling for composite glyphs
//! - Quadrant-based spatial organization

pub mod component;
pub mod path;
pub mod point;
pub mod point_list;
pub mod quadrant;

// Re-export commonly used items
pub use point::EditPoint;
pub use point_list::PointList; 