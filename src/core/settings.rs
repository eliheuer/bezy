// Settings ///////////////////////////////////////////////////////////////////
// This module contains all the settings for the app.

// Snap to Grid ///////////////////////////////////////////////////////////////

// Control whether grid snapping is enabled
pub const SNAP_TO_GRID_ENABLED: bool = true;
// The size of the grid to snap to
pub const SNAP_TO_GRID_VALUE: f32 = 2.0;

// Nudge Settings /////////////////////////////////////////////////////////////

/// The amount to nudge by in each direction (in design units)
pub const NUDGE_AMOUNT: f32 = 2.0;
/// The amount to nudge when shift is held (for larger movements)
pub const SHIFT_NUDGE_AMOUNT: f32 = 8.0;
/// The amount to nudge when command/ctrl is held (for even larger movements)
pub const CMD_NUDGE_AMOUNT: f32 = 32.0;

// Camera Zoom Settings ///////////////////////////////////////////////////////

/// The step multiplier for zooming when using keyboard shortcuts (Cmd++ / Cmd+-)
/// Values closer to 1.0 produce smaller zoom steps, values further from 1.0 produce larger steps
pub const KEYBOARD_ZOOM_STEP: f32 = 0.8; // 0.8 means zoom in by 20% or out by 25% per keystroke

/// Minimum allowed camera scale (maximum zoom in)
pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.01;

/// Maximum allowed camera scale (maximum zoom out)
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 10.0;

// More non-visual settings can be added here as needed
