//! Application settings and configuration
//!
//! This module contains all configurable settings for the Bezy font editor.
//! Settings are organized by category and use const values for performance.
//! For visual/UI settings, see ui/theme.rs

use crate::ui::themes::ThemeVariant;
use bevy::prelude::*;

// FILE PATHS

/// Default UFO file to load if none is specified
pub const DEFAULT_UFO_PATH: &str = "assets/fonts/bezy-grotesk-regular.ufo";

// WINDOW SETTINGS

/// Default window title
pub const WINDOW_TITLE: &str = "Bezy Font Editor";

/// Default window resolution
pub const DEFAULT_WINDOW_SIZE: (f32, f32) = (1024.0, 768.0);

// GRID AND SNAPPING

/// Configuration for grid snapping behavior
#[derive(Debug, Clone, Copy)]
pub struct GridSettings {
    /// Whether grid snapping is enabled by default
    pub enabled: bool,
    /// Size of the grid in font units
    pub unit_size: f32,
    /// Multiplier for sort placement grid (coarser than point grid)
    pub sort_multiplier: f32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            unit_size: 2.0,
            sort_multiplier: 8.0,
        }
    }
}

// GLYPH GRID SETTINGS

/// Configuration for the glyph grid feature
#[derive(Debug, Clone, Copy)]
pub struct GlyphGridSettings {
    /// Whether to create a glyph grid at startup
    pub enabled: bool,
    /// Number of glyphs per row in the grid
    pub glyphs_per_row: usize,
    /// Grid size for snapping (uses checkerboard default)
    pub grid_size: f32,
}

impl Default for GlyphGridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            glyphs_per_row: 32,
            grid_size: 32.0, // Uses CHECKERBOARD_DEFAULT_UNIT_SIZE
        }
    }
}

// KEYBOARD CONTROLS

/// Configuration for keyboard-based editing
#[derive(Debug, Clone, Copy)]
pub struct KeyboardSettings {
    /// Small nudge amount (arrow keys)
    #[allow(dead_code)]
    pub nudge_small: f32,
    /// Medium nudge amount (Shift + arrow keys)
    #[allow(dead_code)]
    pub nudge_medium: f32,
    /// Large nudge amount (Cmd/Ctrl + arrow keys)
    #[allow(dead_code)]
    pub nudge_large: f32,
    /// Zoom step for keyboard zoom in/out
    #[allow(dead_code)]
    pub zoom_step: f32,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            nudge_small: 2.0,
            nudge_medium: 8.0,
            nudge_large: 64.0,
            zoom_step: 0.8,
        }
    }
}

// CAMERA AND VIEWPORT

/// Configuration for camera behavior
#[derive(Debug, Clone, Copy)]
pub struct CameraSettings {
    /// Minimum allowed zoom level (higher = more zoomed out)
    #[allow(dead_code)]
    pub min_zoom_scale: f32,
    /// Maximum allowed zoom level (lower = more zoomed in)
    #[allow(dead_code)]
    pub max_zoom_scale: f32,
    /// Default zoom level
    #[allow(dead_code)]
    pub default_zoom_scale: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            min_zoom_scale: 0.01,
            max_zoom_scale: 10.0,
            default_zoom_scale: 1.0,
        }
    }
}

// GLOBAL SETTINGS RESOURCE

/// Main settings resource containing all configuration
///
/// This is a Bevy resource that can be accessed from any system.
/// Settings are loaded at startup and can be modified at runtime.
#[derive(Resource, Debug, Clone)]
pub struct BezySettings {
    pub grid: GridSettings,
    pub glyph_grid: GlyphGridSettings,
    #[allow(dead_code)]
    pub keyboard: KeyboardSettings,
    #[allow(dead_code)]
    pub camera: CameraSettings,
    /// Current theme variant
    pub theme: ThemeVariant,
}

impl Default for BezySettings {
    fn default() -> Self {
        Self {
            grid: GridSettings::default(),
            glyph_grid: GlyphGridSettings::default(),
            keyboard: KeyboardSettings::default(),
            camera: CameraSettings::default(),
            theme: ThemeVariant::default(),
        }
    }
}

// HELPER FUNCTIONS

impl BezySettings {
    /// Apply grid snapping to a position based on current settings
    ///
    /// This is the standard grid snapping for point editing.
    pub fn apply_grid_snap(&self, position: Vec2) -> Vec2 {
        if self.grid.enabled {
            Vec2::new(
                (position.x / self.grid.unit_size).round()
                    * self.grid.unit_size,
                (position.y / self.grid.unit_size).round()
                    * self.grid.unit_size,
            )
        } else {
            position
        }
    }

    /// Apply sort-specific grid snapping to a position
    ///
    /// Sorts use a coarser grid than regular points for better placement.
    /// This makes it easier to align entire glyphs.
    #[deprecated(
        note = "Use checkerboard dynamic grid size for sort snapping instead"
    )]
    pub fn apply_sort_grid_snap(&self, position: Vec2) -> Vec2 {
        if self.grid.enabled {
            let sort_grid_size =
                self.grid.unit_size * self.grid.sort_multiplier;
            Vec2::new(
                (position.x / sort_grid_size).round() * sort_grid_size,
                (position.y / sort_grid_size).round() * sort_grid_size,
            )
        } else {
            position
        }
    }

    /// Get the nudge amount based on modifier keys
    ///
    /// Returns the appropriate nudge distance based on which modifier
    /// keys are currently pressed.
    #[allow(dead_code)]
    pub fn get_nudge_amount(
        &self,
        shift_pressed: bool,
        cmd_pressed: bool,
    ) -> f32 {
        if cmd_pressed {
            self.keyboard.nudge_large
        } else if shift_pressed {
            self.keyboard.nudge_medium
        } else {
            self.keyboard.nudge_small
        }
    }

    /// Clamp zoom scale to allowed range
    #[allow(dead_code)]
    pub fn clamp_zoom_scale(&self, scale: f32) -> f32 {
        scale.clamp(self.camera.min_zoom_scale, self.camera.max_zoom_scale)
    }

    /// Set the theme variant
    pub fn set_theme(&mut self, theme: ThemeVariant) {
        self.theme = theme;
    }

    /// Get the current theme variant
    pub fn get_theme(&self) -> ThemeVariant {
        self.theme.clone()
    }
}

// LEGACY CONSTANTS (for backward compatibility)

// These constants are kept for backward compatibility but should be replaced
// with the structured settings above in new code.

pub const SNAP_TO_GRID_ENABLED: bool = true;
pub const SNAP_TO_GRID_VALUE: f32 = 2.0;
#[allow(dead_code)]
pub const SORT_SNAP_MULTIPLIER: f32 = 8.0;
pub const NUDGE_AMOUNT: f32 = 2.0;
pub const SHIFT_NUDGE_AMOUNT: f32 = 8.0;
pub const CMD_NUDGE_AMOUNT: f32 = 32.0;
#[allow(dead_code)]
pub const KEYBOARD_ZOOM_STEP: f32 = 0.8;
#[allow(dead_code)]
pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.01;
#[allow(dead_code)]
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 10.0;

/// Legacy function for backward compatibility
///
/// New code should use BezySettings::apply_sort_grid_snap instead
#[deprecated(note = "Use BezySettings::apply_sort_grid_snap instead")]
#[allow(dead_code)]
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
