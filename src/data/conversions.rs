//! UFO format conversion utilities
//!
//! This module contains conversion logic between our internal thread-safe 
//! data structures and the norad UFO format. This is pure data transformation
//! logic - serialization and deserialization between equivalent representations.

use std::path::PathBuf;
use norad::Font;
use crate::core::state::{FontData, GlyphData, OutlineData, ContourData, PointData, PointTypeData, FontInfo};

impl GlyphData {
    /// Convert from norad glyph to our thread-safe version
    pub fn from_norad_glyph(norad_glyph: &norad::Glyph) -> Self {
        let outline = if !norad_glyph.contours.is_empty() {
            Some(OutlineData::from_norad_contours(&norad_glyph.contours))
        } else {
            None
        };

        Self {
            name: norad_glyph.name().to_string(),
            advance_width: norad_glyph.width as f64,
            advance_height: Some(norad_glyph.height as f64),
            unicode_values: norad_glyph.codepoints.iter().collect(),
            outline,
        }
    }

    /// Convert back to norad glyph
    pub fn to_norad_glyph(&self) -> norad::Glyph {
        let mut glyph = norad::Glyph::new(&self.name);
        glyph.width = self.advance_width;
        glyph.height = self.advance_height.unwrap_or(0.0);
        
        // Convert Vec<char> to Codepoints
        for &codepoint in &self.unicode_values {
            glyph.codepoints.insert(codepoint);
        }
        
        if let Some(outline_data) = &self.outline {
            glyph.contours = outline_data.to_norad_contours();
        }
        
        glyph
    }
}

impl OutlineData {
    pub fn from_norad_contours(norad_contours: &[norad::Contour]) -> Self {
        let contours = norad_contours.iter()
            .map(ContourData::from_norad_contour)
            .collect();
        
        Self { contours }
    }

    pub fn to_norad_contours(&self) -> Vec<norad::Contour> {
        self.contours.iter()
            .map(ContourData::to_norad_contour)
            .collect()
    }
}

impl ContourData {
    pub fn from_norad_contour(norad_contour: &norad::Contour) -> Self {
        let points = norad_contour.points.iter()
            .map(PointData::from_norad_point)
            .collect();
        
        Self { points }
    }

    pub fn to_norad_contour(&self) -> norad::Contour {
        let points = self.points.iter()
            .map(PointData::to_norad_point)
            .collect();
        
        // Use constructor with required arguments
        norad::Contour::new(points, None)
    }
}

impl PointData {
    pub fn from_norad_point(norad_point: &norad::ContourPoint) -> Self {
        Self {
            x: norad_point.x as f64,
            y: norad_point.y as f64,
            point_type: PointTypeData::from_norad_point_type(&norad_point.typ),
        }
    }

    pub fn to_norad_point(&self) -> norad::ContourPoint {
        // Use constructor with all 6 required arguments
        norad::ContourPoint::new(
            self.x, // f64 is expected
            self.y, // f64 is expected
            self.point_type.to_norad_point_type(),
            false, // smooth
            None,  // name
            None,  // identifier
        )
    }
}

impl PointTypeData {
    pub fn from_norad_point_type(norad_type: &norad::PointType) -> Self {
        match norad_type {
            norad::PointType::Move => PointTypeData::Move,
            norad::PointType::Line => PointTypeData::Line,
            norad::PointType::OffCurve => PointTypeData::OffCurve,
            norad::PointType::Curve => PointTypeData::Curve,
            norad::PointType::QCurve => PointTypeData::QCurve,
        }
    }

    pub fn to_norad_point_type(&self) -> norad::PointType {
        match self {
            PointTypeData::Move => norad::PointType::Move,
            PointTypeData::Line => norad::PointType::Line,
            PointTypeData::OffCurve => norad::PointType::OffCurve,
            PointTypeData::Curve => norad::PointType::Curve,
            PointTypeData::QCurve => norad::PointType::QCurve,
        }
    }
}

impl FontData {
    /// Extract font data from norad Font 
    pub fn from_norad_font(font: &Font, path: Option<PathBuf>) -> Self {
        let mut glyphs = std::collections::HashMap::new();
        
        // Extract all glyphs from the default layer
        let layer = font.default_layer();
        
        // Iterate over glyphs in the layer
        for glyph in layer.iter() {
            let glyph_data = GlyphData::from_norad_glyph(glyph);
            glyphs.insert(glyph.name().to_string(), glyph_data);
        }
        
        Self { glyphs, path }
    }

    /// Convert back to a complete norad Font
    pub fn to_norad_font(&self, info: &FontInfo) -> Font {
        let mut font = Font::new();
        
        // Set font info using our conversion method
        font.font_info = info.to_norad_font_info();
        
        // Add glyphs to the default layer
        let layer = font.default_layer_mut();
        for (_name, glyph_data) in &self.glyphs {
            let glyph = glyph_data.to_norad_glyph();
            layer.insert_glyph(glyph);
        }
        
        font
    }
} 