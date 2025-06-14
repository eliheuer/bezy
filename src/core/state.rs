//! Application state management.

use norad::Font;

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
