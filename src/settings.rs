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

// More non-visual settings can be added here as needed
