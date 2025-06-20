//! Application state management.
//!
//! This module defines thread-safe data structures optimized for our font editor.
//! We use norad only for loading/saving UFO files, not as runtime storage.

use bevy::prelude::*;
use norad::Font;
use std::collections::HashMap;
use std::path::PathBuf;

/// The main application state - thread-safe for Bevy
#[derive(Resource, Default, Clone)]
pub struct AppState {
    /// The current font editing workspace
    pub workspace: Workspace,
}

/// Represents a font editing session with thread-safe data
#[derive(Clone, Default)]
pub struct Workspace {
    /// Thread-safe font data extracted from norad
    pub font: FontData,
    /// Information about the font (name, metrics, etc.)
    pub info: FontInfo,
    /// The currently selected glyph (if any)
    pub selected: Option<String>,
}

/// Thread-safe font data structure
#[derive(Clone, Default)]
pub struct FontData {
    /// All glyph data extracted from norad and stored thread-safely
    pub glyphs: HashMap<String, GlyphData>,
    /// Path to the UFO file (for saving)
    pub path: Option<PathBuf>,
}

/// Thread-safe glyph data
#[derive(Clone, Debug)]
pub struct GlyphData {
    /// Glyph name
    pub name: String,
    /// Advance width
    pub advance_width: f64,
    /// Advance height (optional)
    pub advance_height: Option<f64>,
    /// Unicode codepoints for this glyph
    pub unicode_values: Vec<char>,
    /// Glyph outline data
    pub outline: Option<OutlineData>,
}

/// Thread-safe outline data
#[derive(Clone, Debug)]
pub struct OutlineData {
    /// Contour data
    pub contours: Vec<ContourData>,
}

/// Thread-safe contour data
#[derive(Clone, Debug)]
pub struct ContourData {
    /// Points in this contour
    pub points: Vec<PointData>,
}

/// Thread-safe point data
#[derive(Clone, Debug)]
pub struct PointData {
    /// X coordinate
    pub x: f64,
    /// Y coordinate  
    pub y: f64,
    /// Point type
    pub point_type: PointTypeData,
}

/// Thread-safe point type
#[derive(Clone, Debug)]
pub enum PointTypeData {
    Move,
    Line, 
    OffCurve,
    Curve,
    QCurve,
}

/// Font information
#[derive(Clone, Default)]
pub struct FontInfo {
    /// Family name
    pub family_name: String,
    /// Style name
    pub style_name: String,
    /// Units per em
    pub units_per_em: f64,
    /// Ascender value
    pub ascender: Option<f64>,
    /// Descender value
    pub descender: Option<f64>,
    /// x-height value
    pub x_height: Option<f64>,
    /// Cap height value
    pub cap_height: Option<f64>,
}

impl AppState {
    /// Load a font from a UFO file path
    pub fn load_font_from_path(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Load the font using norad
        let font = Font::load(&path)?;
        
        // Extract data into our thread-safe structures
        self.workspace.font = FontData::from_norad_font(&font, Some(path));
        self.workspace.info = FontInfo::from_norad_font(&font);
        
        Ok(())
    }

    /// Save the current font to its file path
    pub fn save_font(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.workspace.font.path.clone()
            .ok_or("No file path set - use Save As first")?;
        
        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(&path)?;
        
        info!("Saved font to {:?}", path);
        Ok(())
    }

    /// Save the font to a new path
    pub fn save_font_as(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(&path)?;
        
        // Update our stored path
        self.workspace.font.path = Some(path.clone());
        
        info!("Saved font to {:?}", path);
        Ok(())
    }

    /// Get a display name for the current font
    pub fn get_font_display_name(&self) -> String {
        self.workspace.get_font_display_name()
    }

    /// Get a mutable reference to a specific point in a glyph
    pub fn get_point_mut(&mut self, glyph_name: &str, contour_idx: usize, point_idx: usize) 
        -> Option<&mut PointData> {
        self.workspace.font.glyphs
            .get_mut(glyph_name)?
            .outline.as_mut()?
            .contours.get_mut(contour_idx)?
            .points.get_mut(point_idx)
    }

    /// Update a specific point in a glyph
    pub fn update_point(&mut self, glyph_name: &str, contour_idx: usize, point_idx: usize, new_point: PointData) -> bool {
        if let Some(point) = self.get_point_mut(glyph_name, contour_idx, point_idx) {
            *point = new_point;
            true
        } else {
            false
        }
    }

    /// Get a point by reference (read-only)
    pub fn get_point(&self, glyph_name: &str, contour_idx: usize, point_idx: usize) 
        -> Option<&PointData> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()?
            .contours.get(contour_idx)?
            .points.get(point_idx)
    }

    /// Move a point by a delta amount
    pub fn move_point(&mut self, glyph_name: &str, contour_idx: usize, point_idx: usize, delta_x: f64, delta_y: f64) -> bool {
        if let Some(point) = self.get_point_mut(glyph_name, contour_idx, point_idx) {
            point.x += delta_x;
            point.y += delta_y;
            true
        } else {
            false
        }
    }

    /// Set the position of a point
    pub fn set_point_position(&mut self, glyph_name: &str, contour_idx: usize, point_idx: usize, x: f64, y: f64) -> bool {
        if let Some(point) = self.get_point_mut(glyph_name, contour_idx, point_idx) {
            point.x = x;
            point.y = y;
            true
        } else {
            false
        }
    }

    /// Get all points in a contour (read-only)
    pub fn get_contour_points(&self, glyph_name: &str, contour_idx: usize) -> Option<&Vec<PointData>> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()?
            .contours.get(contour_idx)
            .map(|contour| &contour.points)
    }

    /// Get the number of contours in a glyph
    pub fn get_contour_count(&self, glyph_name: &str) -> Option<usize> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()
            .map(|outline| outline.contours.len())
    }

    /// Get the number of points in a specific contour
    pub fn get_point_count(&self, glyph_name: &str, contour_idx: usize) -> Option<usize> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()?
            .contours.get(contour_idx)
            .map(|contour| contour.points.len())
    }
}

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
        let mut glyphs = HashMap::new();
        
        // Extract all glyphs from the default layer
        let layer = font.default_layer();
        
        // Iterate over glyphs in the layer
        for glyph in layer.iter() {
            let glyph_data = GlyphData::from_norad_glyph(glyph);
            glyphs.insert(glyph.name().to_string(), glyph_data);
        }
        
        Self { glyphs, path }
    }
    
    /// Get a glyph by name
    pub fn get_glyph(&self, name: &str) -> Option<&GlyphData> {
        self.glyphs.get(name)
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

impl FontInfo {
    /// Extract font info from norad Font
    pub fn from_norad_font(font: &Font) -> Self {
        Self {
            family_name: font.font_info.family_name.clone().unwrap_or_else(|| "Untitled".to_string()),
            style_name: font.font_info.style_name.clone().unwrap_or_else(|| "Regular".to_string()),
            units_per_em: font.font_info.units_per_em
                .map(|v| v.to_string().parse().unwrap_or(1024.0))
                .unwrap_or(1024.0),
            ascender: font.font_info.ascender.map(|v| v as f64),
            descender: font.font_info.descender.map(|v| v as f64),
            x_height: font.font_info.x_height.map(|v| v as f64),
            cap_height: font.font_info.cap_height.map(|v| v as f64),
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
    pub fn metrics(&self) -> FontMetrics {
        FontMetrics::from(self)
    }
}

/// Font metrics for rendering
#[derive(Resource, Default, Clone)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub ascender: f64,
    pub descender: f64,
    pub line_height: f64,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
}

impl From<&FontInfo> for FontMetrics {
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

/// Glyph navigation state
#[derive(Resource, Default)]
pub struct GlyphNavigation {
    pub current_glyph: Option<String>,
    pub glyph_list: Vec<String>,
    pub current_index: usize,
}

impl GlyphNavigation {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_current_glyph(&mut self, glyph_name: String) {
        self.current_glyph = Some(glyph_name);
    }
    
    pub fn get_current_glyph(&self) -> Option<&String> {
        self.current_glyph.as_ref()
    }
}

impl Workspace {
    /// Get a display name for the font
    pub fn get_font_display_name(&self) -> String {
        self.get_font_name()
    }

    /// Get a display name combining family and style names  
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
}
