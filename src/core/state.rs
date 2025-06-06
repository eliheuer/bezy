//! Application state management
//!
//! This module contains the core state structures that track what the user
//! is currently working on - which font is loaded, which glyph is selected,
//! and the overall editing session state.

use crate::data::ufo::find_glyph_by_unicode;
use crate::editing::selection::components::GlyphPointReference;
use bevy::prelude::*;
use norad::glyph::ContourPoint;
use norad::{GlyphName, Ufo};
use std::path::PathBuf;

/// Tracks which glyph the user is currently viewing
///
/// This is separate from the CLI args.
/// It changes as the user navigates between different glyphs in the font.
#[derive(Resource, Default, Clone)]
pub struct GlyphNavigation {
    /// The current Unicode codepoint being viewed (like "0061" for 'a')
    pub current_codepoint: Option<String>,
    /// Whether we found this codepoint in the loaded font
    pub codepoint_found: bool,
}

impl GlyphNavigation {
    /// Create a new navigation state with a starting codepoint
    pub fn new(initial_codepoint: Option<String>) -> Self {
        Self {
            current_codepoint: initial_codepoint,
            codepoint_found: false,
        }
    }

    /// Change to a different codepoint
    pub fn set_codepoint(&mut self, new_codepoint: String) {
        self.current_codepoint = Some(new_codepoint);
        self.codepoint_found = false; // We'll need to check if this exists
    }

    /// Get the current codepoint as a string for display
    pub fn get_codepoint_string(&self) -> String {
        self.current_codepoint.clone().unwrap_or_default()
    }

    /// Find the glyph name for the current codepoint
    pub fn find_glyph(&self, ufo: &Ufo) -> Option<GlyphName> {
        self.current_codepoint
            .as_ref()
            .and_then(|codepoint| find_glyph_by_unicode(ufo, codepoint))
            .map(GlyphName::from)
    }
}

/// The main application state
///
/// This holds everything the app needs to know about the current editing session.
#[derive(Resource, Default, Clone)]
pub struct AppState {
    /// The current font editing workspace
    pub workspace: Workspace,
}

impl AppState {
    /// Load a new font into the editor
    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.workspace.set_font(ufo, path);
    }

    /// Get a display name for the current font
    pub fn get_font_display_name(&self) -> String {
        self.workspace.get_font_name()
    }

    /// Get a mutable reference to a point in the font data
    pub fn get_point_mut(
        &mut self,
        point_ref: &GlyphPointReference,
    ) -> Option<&mut ContourPoint> {
        let glyph_name = GlyphName::from(&*point_ref.glyph_name);

        self.workspace
            .font
            .ufo
            .get_default_layer_mut()?
            .get_glyph_mut(&glyph_name)?
            .outline
            .as_mut()?
            .contours
            .get_mut(point_ref.contour_index)?
            .points
            .get_mut(point_ref.point_index)
    }
}

/// Represents a font editing session
///
/// A workspace contains one font and all the information
/// about how we're editing it.
#[derive(Clone)]
pub struct Workspace {
    /// The font we're editing
    pub font: FontObject,
    /// Information about the font (name, metrics, etc.)
    pub info: FontInfo,
    /// The currently selected glyph (if any)
    pub selected: Option<GlyphName>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            font: FontObject::default(),
            info: FontInfo::default(),
            selected: None,
        }
    }
}

impl Workspace {
    /// Load a new font into this workspace
    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.font = FontObject { ufo, path };
        self.info = FontInfo::from_ufo(&self.font.ufo);
    }

    /// Get a display name for the font
    pub fn get_font_name(&self) -> String {
        let parts: Vec<&str> = [&self.info.family_name, &self.info.style_name]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect();

        if parts.is_empty() {
            "Untitled Font".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// Save the font to its file path
    pub fn save(&mut self) -> Result<(), String> {
        let path = self
            .font
            .path
            .clone()
            .ok_or("No file path set - use Save As first")?;

        // Update the UFO with current info before saving
        self.update_ufo_info();

        // Save the UFO
        self.font
            .ufo
            .save(&path)
            .map_err(|e| format!("Failed to save: {}", e))?;

        info!("Saved font to {:?}", path);
        Ok(())
    }

    /// Update the UFO's info from our FontInfo
    fn update_ufo_info(&mut self) {
        let font_info =
            self.font.ufo.font_info.get_or_insert_with(Default::default);

        if !self.info.family_name.is_empty() {
            font_info.family_name = Some(self.info.family_name.clone());
        }
        if !self.info.style_name.is_empty() {
            font_info.style_name = Some(self.info.style_name.clone());
        }
    }

    /// Get a mutable reference to the font object
    pub fn font_mut(&mut self) -> &mut FontObject {
        &mut self.font
    }
}

/// A font file with its path on disk
#[derive(Clone)]
pub struct FontObject {
    /// The actual font data
    pub ufo: Ufo,
    /// Where this font is stored on disk (if anywhere)
    pub path: Option<PathBuf>,
}

impl Default for FontObject {
    fn default() -> Self {
        let mut ufo = Ufo::new();
        // Set up a basic default font
        let mut font_info = norad::FontInfo::default();
        font_info.family_name = Some("Untitled".to_string());
        font_info.style_name = Some("Regular".to_string());
        ufo.font_info = Some(font_info);

        Self { ufo, path: None }
    }
}

/// Basic information about a font
///
/// This extracts the most important info from the UFO for easy access.
#[derive(Clone, Default)]
pub struct FontInfo {
    pub family_name: String,
    pub style_name: String,
    #[allow(dead_code)]
    pub units_per_em: f64,
    pub metrics: FontMetrics,
}

impl FontInfo {
    /// Extract font info from a UFO
    pub fn from_ufo(ufo: &Ufo) -> Self {
        let font_info = ufo.font_info.as_ref();

        Self {
            family_name: Self::extract_string_field(
                font_info,
                |info| &info.family_name,
                "Untitled",
            ),
            style_name: Self::extract_string_field(
                font_info,
                |info| &info.style_name,
                "Regular",
            ),
            units_per_em: font_info
                .and_then(|info| info.units_per_em.map(|v| v.get() as f64))
                .unwrap_or(1024.0),
            metrics: FontMetrics::from_ufo(ufo),
        }
    }

    /// Helper to extract string fields with defaults
    fn extract_string_field<F>(
        font_info: Option<&norad::FontInfo>,
        getter: F,
        default: &str,
    ) -> String
    where
        F: Fn(&norad::FontInfo) -> &Option<String>,
    {
        font_info
            .and_then(|info| getter(info).as_ref())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Get a display name combining family and style names
    #[allow(dead_code)]
    pub fn get_display_name(&self) -> String {
        let parts: Vec<&str> = [&self.family_name, &self.style_name]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect();

        if parts.is_empty() {
            "Untitled Font".to_string()
        } else {
            parts.join(" ")
        }
    }
}

/// Font metrics for spacing and positioning
#[derive(Clone, Default)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub descender: Option<f64>,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
    pub ascender: Option<f64>,
    #[allow(dead_code)]
    pub italic_angle: Option<f64>,
}

impl FontMetrics {
    /// Extract metrics from a UFO
    pub fn from_ufo(ufo: &Ufo) -> Self {
        let font_info = ufo.font_info.as_ref();

        // Helper closure to extract optional f64 values for regular metrics
        let extract_metric =
            |getter: fn(&norad::FontInfo) -> Option<norad::IntegerOrFloat>| {
                font_info.and_then(|info| getter(info).map(|v| v.get() as f64))
            };

        Self {
            units_per_em: font_info
                .and_then(|info| info.units_per_em.map(|v| v.get() as f64))
                .unwrap_or(1024.0),
            descender: extract_metric(|info| info.descender),
            x_height: extract_metric(|info| info.x_height),
            cap_height: extract_metric(|info| info.cap_height),
            ascender: extract_metric(|info| info.ascender),
            italic_angle: extract_metric(|info| info.italic_angle),
        }
    }
}

// Legacy type alias for compatibility
pub type SimpleFontInfo = FontInfo;
