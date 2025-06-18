//! Application state management.

use norad::{Font, fontinfo::NonNegativeIntegerOrFloat};

/// The main application state.
///
/// This holds everything the app needs to know about the current editing session.
#[derive(Default)]
pub struct AppState {
    /// The current font we are editing.
    pub font: Option<Font>,
    /// The metrics for the current font.
    pub metrics: Option<FontMetrics>,
}

/// A collection of important font-wide metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub ascender: Option<f64>,
    pub descender: Option<f64>,
}

impl AppState {
    /// Update the state when a font is loaded
    pub fn set_font(&mut self, font: Font) {
        // Extract metrics from font
        let font_info = &font.font_info;
        let units_per_em = font_info.units_per_em.unwrap_or(NonNegativeIntegerOrFloat::new(1000.0).unwrap());
        let ascender = font_info.ascender;
        let descender = font_info.descender;
        
        let metrics = FontMetrics {
            units_per_em: (*units_per_em).into(),
            ascender,
            descender,
        };
        
        // Update main state
        self.font = Some(font);
        self.metrics = Some(metrics);
    }
}
