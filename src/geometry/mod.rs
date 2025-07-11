//! Geometric Primitives and Operations

pub mod component;
pub mod design_space;
pub mod indices;
pub mod path;
pub mod point;
pub mod point_list;
pub mod quadrant;

// Re-export commonly used items
pub use design_space::{DPoint, DVec2}; 