//! Font metrics and measurement data
//!
//! This module contains structures for font metrics like ascender,
//! descender, x-height, and other measurement information.

use bevy::prelude::*;
use norad::Font;

/// Font information
#[derive(Clone, Default)]
pub struct FontInfo {
    /// Family name
    pub family_name: String,
    /// Style name
    pub style_name: String,
    /// Units per em
    pub units_per_em: f64,
    /// Font metrics for spacing and positioning
    pub metrics: FontMetrics,
    /// Ascender value
    pub ascender: Option<f64>,
    /// Descender value
    pub descender: Option<f64>,
    /// x-height value
    pub x_height: Option<f64>,
    /// Cap height value
    pub cap_height: Option<f64>,
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
    #[allow(dead_code)]
    pub line_height: f64,
}

/// Create resource-compatible FontMetrics for rendering
#[derive(Resource, Debug, Clone)]
pub struct FontMetricsResource {
    #[allow(dead_code)]
    pub units_per_em: f64,
    #[allow(dead_code)]
    pub ascender: f64,
    #[allow(dead_code)]
    pub descender: f64,
    #[allow(dead_code)]
    pub line_height: f64,
    #[allow(dead_code)]
    pub x_height: Option<f64>,
    #[allow(dead_code)]
    pub cap_height: Option<f64>,
}

impl FontInfo {
    /// Extract font info from norad Font
    pub fn from_norad_font(font: &Font) -> Self {
        let units_per_em = font.font_info.units_per_em
            .map(|v| v.to_string().parse().unwrap_or(1024.0))
            .unwrap_or(1024.0);
        let ascender = font.font_info.ascender.map(|v| v as f64);
        let descender = font.font_info.descender.map(|v| v as f64);
        let x_height = font.font_info.x_height.map(|v| v as f64);
        let cap_height = font.font_info.cap_height.map(|v| v as f64);
        let _italic_angle = font.font_info.italic_angle.map(|v| v as f64);
        
        let metrics = FontMetrics::from_ufo(font);
        
        Self {
            family_name: Self::extract_string_field(&font.font_info, |info| &info.family_name, "Untitled"),
            style_name: Self::extract_string_field(&font.font_info, |info| &info.style_name, "Regular"),
            units_per_em,
            metrics,
            ascender,
            descender,
            x_height,
            cap_height,
        }
    }
    
    /// Helper to extract string fields with defaults
    fn extract_string_field<F>(
        font_info: &norad::FontInfo,
        getter: F,
        default: &str,
    ) -> String
    where
        F: Fn(&norad::FontInfo) -> &Option<String>,
    {
        getter(font_info)
            .as_ref()
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
    
    /// Get a display name combining family and style names
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
    
    /// Convert back to norad FontInfo
    pub fn to_norad_font_info(&self) -> norad::FontInfo {
        let mut info = norad::FontInfo::default();
        
        // Set family and style names
        if !self.family_name.is_empty() {
            info.family_name = Some(self.family_name.clone());
        }
        if !self.style_name.is_empty() {
            info.style_name = Some(self.style_name.clone());
        }
        
        // Set numeric values
        if let Some(units_per_em) = norad::fontinfo::NonNegativeIntegerOrFloat::new(self.units_per_em) {
            info.units_per_em = Some(units_per_em);
        }
        info.ascender = self.ascender;
        info.descender = self.descender;
        info.x_height = self.x_height;
        info.cap_height = self.cap_height;
        info
    }
    
    /// Get metrics for rendering
    #[allow(dead_code)]
    pub fn get_metrics(&self) -> &FontMetrics {
        &self.metrics
    }
}

impl FontMetrics {
    /// Extract metrics from a UFO
    pub fn from_ufo(ufo: &Font) -> Self {
        let font_info = &ufo.font_info;

        let units_per_em = font_info
            .units_per_em.map(|v| v.to_string().parse().unwrap_or(1024.0))
            .unwrap_or(1024.0);
        
        // Load metrics from UFO, using reasonable defaults based on units_per_em if missing
        let ascender = font_info.ascender.map(|v| v as f64)
            .or_else(|| Some(units_per_em * 0.8)); // 80% of UPM
        let descender = font_info.descender.map(|v| v as f64)
            .or_else(|| Some(-(units_per_em * 0.2))); // -20% of UPM
        let x_height = font_info.x_height.map(|v| v as f64);
        let cap_height = font_info.cap_height.map(|v| v as f64);
        let _italic_angle = font_info.italic_angle.map(|v| v as f64);
        
        let line_height = ascender.unwrap() - descender.unwrap();

        Self {
            units_per_em,
            descender,
            x_height,
            cap_height,
            ascender,
            italic_angle: None,
            line_height,
        }
    }
}

impl From<&FontInfo> for FontMetricsResource {
    fn from(info: &FontInfo) -> Self {
        Self {
            units_per_em: info.units_per_em,
            ascender: info.ascender.unwrap_or(800.0),
            descender: info.descender.unwrap_or(-200.0),
            line_height: info.ascender.unwrap_or(800.0) - info.descender.unwrap_or(-200.0),
            x_height: info.x_height,
            cap_height: info.cap_height,
        }
    }
} 