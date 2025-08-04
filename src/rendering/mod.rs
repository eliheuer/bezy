//! Rendering and Visualization
//!
//! This module contains all rendering and visualization functionality:
//! - Drawing systems for glyphs, paths, and UI elements
//! - Camera management for viewport control
//! - Background patterns and visual aids
//! - Debug visualization tools
//! - Selection visualization (marquee, selected points, handles)

#![allow(unused_imports)]

pub mod camera_responsive;
pub mod cameras;
pub mod checkerboard;
pub mod debug;
pub mod draw;
pub mod entity_pools;
pub mod mesh_cache;
// Removed: gizmo-based outline modules (fontir_glyph_outline, glyph_outline)
// All outline rendering is now handled by mesh_glyph_outline
pub mod mesh_glyph_outline;
pub mod metrics;
pub mod outline_coordination;
pub mod outline_elements;
pub mod points;
pub mod selection;
pub mod sort_renderer;
pub mod sort_visuals;
pub mod unified_glyph_editing;

// Re-export commonly used items
pub use camera_responsive::{CameraResponsivePlugin, CameraResponsiveScale};
pub use checkerboard::{CheckerboardEnabled, CheckerboardPlugin};
pub use entity_pools::EntityPoolingPlugin;
pub use mesh_cache::MeshCachingPlugin;
pub use mesh_glyph_outline::MeshGlyphOutlinePlugin;
pub use metrics::MetricsRenderingPlugin;
pub use outline_coordination::OutlineCoordinationPlugin;
pub use outline_elements::OutlineElementsPlugin;
pub use points::PointRenderingPlugin;
pub use selection::{
    render_all_point_entities, render_selected_entities,
    render_selection_marquee,
};
pub use sort_visuals::SortHandleRenderingPlugin;
pub use unified_glyph_editing::UnifiedGlyphEditingPlugin;
