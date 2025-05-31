//! Settings: User editable settings for the application
//!
//! This module contains all non-visual settings for the application.
//! For visual settings, see the UI/theme.rs file.

// Snap to Grid
pub const SNAP_TO_GRID_ENABLED: bool = true;
pub const SNAP_TO_GRID_VALUE: f32 = 2.0;

// Nudge Settings
pub const NUDGE_AMOUNT: f32 = 2.0;
pub const SHIFT_NUDGE_AMOUNT: f32 = 8.0;
pub const CMD_NUDGE_AMOUNT: f32 = 32.0;

// Camera Zoom Settings
pub const KEYBOARD_ZOOM_STEP: f32 = 0.8;
pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.01;
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 10.0;