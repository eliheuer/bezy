//! Application settings and configuration
//!
//! This module contains all configurable settings for the Bezy font editor.
//! Settings are organized by category and use const values for performance.
//! For visual/UI settings, see ui/theme.rs

use crate::ui::themes::ThemeVariant;
use bevy::prelude::*;

// FILE PATHS

/// Default UFO file to load if none is specified
pub const DEFAULT_UFO_PATH: &str = "assets/fonts/bezy-grotesk.designspace";

// WINDOW SETTINGS

/// Default window title
pub const WINDOW_TITLE: &str = "Bezy";

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


// GLOBAL SETTINGS RESOURCE

/// Main settings resource containing all configuration
///
/// This is a Bevy resource that can be accessed from any system.
/// Settings are loaded at startup and can be modified at runtime.
#[derive(Resource, Debug, Clone, Default)]
pub struct BezySettings {
    pub grid: GridSettings,
    /// Current theme variant
    pub theme: ThemeVariant,
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
pub const NUDGE_AMOUNT: f32 = 2.0;
pub const SHIFT_NUDGE_AMOUNT: f32 = 8.0;
pub const CMD_NUDGE_AMOUNT: f32 = 32.0;

