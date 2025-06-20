//! Settings: User editable settings for the application
//!
//! This module contains all non-visual settings for the application.
//! For visual settings, see the UI/theme.rs file.

use bevy::prelude::*;

// Default UFO File
pub const DEFAULT_UFO_PATH: &str = "assets/fonts/bezy-grotesk-regular.ufo";

// Snap to Grid
pub const SNAP_TO_GRID_ENABLED: bool = true;
pub const SNAP_TO_GRID_VALUE: f32 = 2.0;

// Sort Snap to Grid (for text mode sort placement)
// Sorts snap to a coarser grid than they represent entire glyphs
pub const SORT_SNAP_MULTIPLIER: f32 = 8.0;

// Nudge Settings
pub const NUDGE_AMOUNT: f32 = 2.0;
pub const SHIFT_NUDGE_AMOUNT: f32 = 8.0;
pub const CMD_NUDGE_AMOUNT: f32 = 32.0;

// Camera Zoom Settings
pub const KEYBOARD_ZOOM_STEP: f32 = 0.8;
pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.01;
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 10.0;

/// Apply sort-specific grid snapping to a position.
/// Sorts use a coarser grid than regular points for better placement.
pub fn apply_sort_grid_snap(position: Vec2) -> Vec2 {
    if SNAP_TO_GRID_ENABLED {
        let sort_grid_value = SNAP_TO_GRID_VALUE * SORT_SNAP_MULTIPLIER;
        Vec2::new(
            (position.x / sort_grid_value).round() * sort_grid_value,
            (position.y / sort_grid_value).round() * sort_grid_value,
        )
    } else {
        position
    }
}
