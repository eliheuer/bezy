//! Virtual workspace management
//! 
//! This module provides workspace management functionality that was part of the
//! original application design. It's adapted to work with our current AppState structure.

use bevy::prelude::*;
use norad::{Font, Glyph};
use std::collections::HashMap;
use std::path::PathBuf;
// Arc not needed for current implementation

use crate::core::state::{FontInfo, FontMetrics};

/// Legacy workspace resource for compatibility with older UI code
/// 
/// This provides a simplified interface that some UI components expect,
/// while delegating to the main AppState for actual data storage.
/// Note: This struct contains norad::Font which is not Send/Sync,
/// so it cannot be used as a Bevy Resource. It's kept for compatibility
/// with legacy UI code that expects this interface.
#[derive(Default)]
pub struct Workspace {
    /// The UFO font data (delegated to AppState in practice)
    #[allow(dead_code)]
    pub ufo: Option<Font>,
    /// File path of the current font
    #[allow(dead_code)]
    pub path: Option<PathBuf>,
    /// Currently selected glyph name
    #[allow(dead_code)]
    pub selected_glyph: Option<String>,
    /// Map of open glyph entities
    #[allow(dead_code)]
    pub open_glyphs: HashMap<String, Entity>,
    /// Font information and metrics
    #[allow(dead_code)]
    pub info: FontInfo,
}

impl Workspace {
    /// Create a new empty workspace
    #[allow(dead_code)]
    pub fn new() -> Self {
        Workspace {
            ufo: None,
            path: None,
            selected_glyph: None,
            open_glyphs: HashMap::new(),
            info: FontInfo::default(),
        }
    }

    /// Set the font data for this workspace
    #[allow(dead_code)]
    pub fn set_font(&mut self, ufo: Font, path: Option<PathBuf>) {
        self.ufo = Some(ufo);
        self.path = path;
        self.selected_glyph = None;
        self.open_glyphs.clear();
        self.info = FontInfo::default();
    }

    /// Get a glyph by name
    #[allow(dead_code)]
    pub fn get_glyph(&self, name: &str) -> Option<&Glyph> {
        self.ufo.as_ref()?.default_layer().get_glyph(name)
    }

    /// Get font metrics
    #[allow(dead_code)]
    pub fn get_metrics(&self) -> FontMetrics {
        self.info.metrics.clone()
    }

    /// Get display name for the font
    #[allow(dead_code)]
    pub fn get_display_name(&self) -> String {
        self.info.get_display_name()
    }
}

/// Simplified font info type for compatibility
#[allow(dead_code)]
pub type SimpleFontInfo = FontInfo; 