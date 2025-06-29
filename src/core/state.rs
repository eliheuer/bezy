//! Application state management.
//!
//! This module defines thread-safe data structures optimized for our font editor.
//! We use norad only for loading/saving UFO files, not as runtime storage.
//! 
//! The main AppState resource contains all font data in a format optimized for
//! real-time editing operations. Changes are batched and synchronized with the
//! UFO format only when saving.

use bevy::prelude::*;
use norad::Font;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::errors::{BezyResult, BezyContext, validate_finite_coords, validate_ufo_path};
use crate::{glyph_not_found, point_out_of_bounds, contour_out_of_bounds};
use anyhow::{ensure, Context};

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
    #[allow(dead_code)]
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

impl AppState {
    /// Load a font from a UFO file path
    /// 
    /// This method loads a UFO font file and converts it into our optimized
    /// internal representation for real-time editing.
    pub fn load_font_from_path(&mut self, path: PathBuf) -> BezyResult<()> {
        // Validate the UFO path
        validate_ufo_path(&path)?;
        
        // Load the font using norad
        let font = Font::load(&path)
            .with_file_context("load", &path)?;
        
        // Extract data into our thread-safe structures
        self.workspace.font = FontData::from_norad_font(&font, Some(path));
        self.workspace.info = FontInfo::from_norad_font(&font);
        
        info!("Successfully loaded UFO font with {} glyphs", self.workspace.font.glyphs.len());
        Ok(())
    }

    /// Save the current font to its file path
    /// 
    /// This method converts our internal representation back to UFO format
    /// and saves it to the file path that was used to load the font.
    pub fn save_font(&self) -> BezyResult<()> {
        let path = self.workspace.font.path.as_ref()
            .context("No file path set - use Save As first")?;
        
        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(path)
            .with_file_context("save", path)?;
        
        info!("Saved font to {:?}", path);
        Ok(())
    }

    /// Save the font to a new path
    /// 
    /// This method saves the font to a new location and updates the internal
    /// path reference for future save operations.
    pub fn save_font_as(&mut self, path: PathBuf) -> BezyResult<()> {
        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(&path)
            .with_file_context("save", &path)?;
        
        // Update our stored path
        self.workspace.font.path = Some(path.clone());
        
        info!("Saved font to new location: {:?}", path);
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
    /// 
    /// This method updates a point's data with comprehensive validation.
    #[allow(dead_code)]
    pub fn update_point(&mut self, glyph_name: &str, contour_idx: usize, point_idx: usize, new_point: PointData) -> BezyResult<()> {
        // Validate coordinates first
        validate_finite_coords(new_point.x, new_point.y)?;
        
        // Validate glyph exists
        let glyph = self.workspace.font.glyphs.get(glyph_name)
            .ok_or_else(|| glyph_not_found!(glyph_name, self.workspace.font.glyphs.len()))?;
        
        // Validate outline exists
        let outline = glyph.outline.as_ref()
            .context(format!("Glyph '{}' has no outline data", glyph_name))?;
        
        // Validate contour exists
        ensure!(
            contour_idx < outline.contours.len(),
            contour_out_of_bounds!(glyph_name, contour_idx, outline.contours.len())
        );
        
        // Validate point exists
        let contour = &outline.contours[contour_idx];
        ensure!(
            point_idx < contour.points.len(),
            point_out_of_bounds!(glyph_name, contour_idx, point_idx, contour.points.len())
        );
        
        // Update the point (we know it exists after validation)
        let point = self.get_point_mut(glyph_name, contour_idx, point_idx)
            .context("Point should exist after validation")?;
        *point = new_point;
        
        Ok(())
    }

    /// Get a point by reference (read-only)
    #[allow(dead_code)]
    pub fn get_point(&self, glyph_name: &str, contour_idx: usize, point_idx: usize) 
        -> Option<&PointData> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()?
            .contours.get(contour_idx)?
            .points.get(point_idx)
    }

    /// Move a point by a delta amount
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn get_contour_points(&self, glyph_name: &str, contour_idx: usize) -> Option<&Vec<PointData>> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()?
            .contours.get(contour_idx)
            .map(|contour| &contour.points)
    }

    /// Get the number of contours in a glyph
    #[allow(dead_code)]
    pub fn get_contour_count(&self, glyph_name: &str) -> Option<usize> {
        self.workspace.font.glyphs
            .get(glyph_name)?
            .outline.as_ref()
            .map(|outline| outline.contours.len())
    }

    /// Get the number of points in a specific contour
    #[allow(dead_code)]
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

/// Glyph navigation state
#[derive(Resource, Default)]
pub struct GlyphNavigation {
    /// The current Unicode codepoint being viewed (like "0061" for 'a')
    pub current_codepoint: Option<String>,
    /// Whether we found this codepoint in the loaded font
    pub codepoint_found: bool,
    /// Legacy fields for compatibility
    pub current_glyph: Option<String>,
    #[allow(dead_code)]
    pub glyph_list: Vec<String>,
    #[allow(dead_code)]
    pub current_index: usize,
}

impl GlyphNavigation {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a new navigation state with a starting codepoint
    #[allow(dead_code)]
    pub fn with_codepoint(initial_codepoint: Option<String>) -> Self {
        Self {
            current_codepoint: initial_codepoint,
            codepoint_found: false,
            current_glyph: None,
            glyph_list: Vec::new(),
            current_index: 0,
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
    pub fn find_glyph(&self, app_state: &AppState) -> Option<String> {
        self.current_codepoint
            .as_ref()
            .and_then(|codepoint| find_glyph_by_unicode_codepoint(app_state, codepoint))
    }
    
    /// Legacy methods for compatibility
    #[allow(dead_code)]
    pub fn set_current_glyph(&mut self, glyph_name: String) {
        self.current_glyph = Some(glyph_name);
    }
    
    #[allow(dead_code)]
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

/// Find a glyph by Unicode codepoint in the app state
pub fn find_glyph_by_unicode_codepoint(app_state: &AppState, codepoint: &str) -> Option<String> {
    // Parse the codepoint string to a character
    if let Ok(codepoint_num) = u32::from_str_radix(codepoint, 16) {
        if let Some(ch) = char::from_u32(codepoint_num) {
            // Search through all glyphs for one with this unicode value
            for (glyph_name, glyph_data) in &app_state.workspace.font.glyphs {
                if glyph_data.unicode_values.contains(&ch) {
                    return Some(glyph_name.clone());
                }
            }
        }
    }
    None
}

/// Get all unicode codepoints available in the font
pub fn get_all_codepoints(app_state: &AppState) -> Vec<String> {
    let mut codepoints = Vec::new();
    
    for glyph_data in app_state.workspace.font.glyphs.values() {
        for &unicode_char in &glyph_data.unicode_values {
            let codepoint = format!("{:04X}", unicode_char as u32);
            if !codepoints.contains(&codepoint) {
                codepoints.push(codepoint);
            }
        }
    }
    
    // Sort and return
    codepoints.sort();
    codepoints
}

/// Navigation direction for cycling through codepoints
#[derive(Clone, Debug)]
pub enum CycleDirection {
    Next,
    Previous,
}

/// Find the next or previous codepoint in the font's available codepoints
pub fn cycle_codepoint_in_list(
    current_codepoint: Option<String>, 
    app_state: &AppState,
    direction: CycleDirection,
) -> Option<String> {
    let codepoints = get_all_codepoints(app_state);
    
    if codepoints.is_empty() {
        return None;
    }
    
    // If no current codepoint, return the first one
    let current = match current_codepoint {
        Some(cp) => cp,
        None => return codepoints.first().cloned(),
    };
    
    // Find the position of the current codepoint
    if let Some(current_index) = codepoints.iter().position(|cp| cp == &current) {
        match direction {
            CycleDirection::Next => {
                let next_index = (current_index + 1) % codepoints.len();
                codepoints.get(next_index).cloned()
            }
            CycleDirection::Previous => {
                let prev_index = if current_index == 0 {
                    codepoints.len() - 1
                } else {
                    current_index - 1
                };
                codepoints.get(prev_index).cloned()
            }
        }
    } else {
        // Current codepoint not found, return first
        codepoints.first().cloned()
    }
}

/// Text editor state for dynamic sort management
#[derive(Resource, Clone, Default)]
pub struct TextEditorState {
    /// The text buffer containing sort content (like a rope or gap buffer)
    pub buffer: SortBuffer,
    /// Current cursor position in the buffer
    pub cursor_position: usize,
    /// Selection range (start, end) if any
    pub selection: Option<(usize, usize)>,
    /// Viewport offset for scrolling
    pub viewport_offset: Vec2,
    /// Grid layout configuration
    pub grid_config: GridConfig,
}

/// Layout mode for individual sorts
#[derive(Clone, Debug, Default, PartialEq)]
pub enum SortLayoutMode {
    /// Sort flows like text characters in a line
    #[default]
    Text,
    /// Sort is positioned freely in the design space
    Freeform,
}

/// Text mode configuration
#[derive(Resource, Clone, Debug, Default)]
pub struct TextModeConfig {
    /// Whether new sorts should be placed in text or freeform mode
    pub default_placement_mode: SortLayoutMode,
    /// Whether to show the mode toggle UI
    #[allow(dead_code)]
    pub show_mode_toggle: bool,
}

/// Unified buffer of all sorts (both text and freeform) using gap buffer for efficient editing
#[derive(Clone)]
pub struct SortBuffer {
    /// The gap buffer storage
    buffer: Vec<SortEntry>,
    /// Gap start position
    gap_start: usize,
    /// Gap end position (exclusive)
    gap_end: usize,
}

impl Default for SortBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SortBuffer {
    /// Create a new gap buffer with initial capacity
    pub fn new() -> Self {
        let initial_capacity = 1024; // Start with room for plenty of sorts
        let mut buffer = Vec::with_capacity(initial_capacity);
        // Fill with default entries to create the gap
        buffer.resize(initial_capacity, SortEntry::default());
        
        Self {
            buffer,
            gap_start: 0,
            gap_end: initial_capacity,
        }
    }
    
    /// Create gap buffer from existing sorts (for font loading)
    #[allow(dead_code)]
    pub fn from_sorts(sorts: Vec<SortEntry>) -> Self {
        let len = sorts.len();
        let capacity = (len * 2).max(1024); // Double capacity for future edits
        let mut buffer = Vec::with_capacity(capacity);
        
        // Add the existing sorts
        buffer.extend(sorts);
        // Fill the rest with default entries to create the gap
        buffer.resize(capacity, SortEntry::default());
        
        Self {
            buffer,
            gap_start: len,
            gap_end: capacity,
        }
    }
    
    /// Get the logical length (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }
    
    /// Check if buffer is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get sort at logical position
    pub fn get(&self, index: usize) -> Option<&SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        if index < self.gap_start {
            self.buffer.get(index)
        } else {
            self.buffer.get(index + (self.gap_end - self.gap_start))
        }
    }
    
    /// Get mutable sort at logical position
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        if index < self.gap_start {
            self.buffer.get_mut(index)
        } else {
            self.buffer.get_mut(index + (self.gap_end - self.gap_start))
        }
    }
    
    /// Move gap to position for efficient insertion/deletion
    fn move_gap_to(&mut self, position: usize) {
        if position == self.gap_start {
            return;
        }
        
        if position < self.gap_start {
            // Move gap left
            let move_count = self.gap_start - position;
            let gap_size = self.gap_end - self.gap_start;
            
            // Move elements from before gap to after gap
            for i in 0..move_count {
                let src_idx = position + i;
                let dst_idx = self.gap_end - move_count + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }
            
            self.gap_start = position;
            self.gap_end = position + gap_size;
        } else {
            // Move gap right
            let move_count = position - self.gap_start;
            let gap_size = self.gap_end - self.gap_start;
            
            // Move elements from after gap to before gap
            for i in 0..move_count {
                let src_idx = self.gap_end + i;
                let dst_idx = self.gap_start + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
            }
            
            self.gap_start = position;
            self.gap_end = position + gap_size;
        }
    }
    
    /// Insert sort at position
    pub fn insert(&mut self, index: usize, sort: SortEntry) {
        if index > self.len() {
            return;
        }
        
        // Ensure we have space in the gap
        if self.gap_start >= self.gap_end {
            self.grow_gap();
        }
        
        // Move gap to insertion point
        self.move_gap_to(index);
        
        // Insert at gap start
        self.buffer[self.gap_start] = sort;
        self.gap_start += 1;
    }
    
    /// Delete sort at position
    pub fn delete(&mut self, index: usize) -> Option<SortEntry> {
        if index >= self.len() {
            return None;
        }
        
        // Move gap to deletion point
        self.move_gap_to(index);
        
        // The element to delete is now just before the gap
        if self.gap_start > 0 {
            self.gap_start -= 1;
            let deleted = self.buffer[self.gap_start].clone();
            self.buffer[self.gap_start] = SortEntry::default();
            Some(deleted)
        } else {
            None
        }
    }
    
    /// Grow the gap when it gets too small
    fn grow_gap(&mut self) {
        let old_capacity = self.buffer.len();
        let new_capacity = old_capacity * 2;
        let gap_size = self.gap_end - self.gap_start;
        let new_gap_size = gap_size + (new_capacity - old_capacity);
        
        // Extend buffer
        self.buffer.resize(new_capacity, SortEntry::default());
        
        // Move elements after gap to end of new buffer
        let elements_after_gap = old_capacity - self.gap_end;
        if elements_after_gap > 0 {
            for i in (0..elements_after_gap).rev() {
                let src_idx = self.gap_end + i;
                let dst_idx = new_capacity - elements_after_gap + i;
                self.buffer[dst_idx] = self.buffer[src_idx].clone();
                self.buffer[src_idx] = SortEntry::default();
            }
        }
        
        // Update gap end
        self.gap_end = self.gap_start + new_gap_size;
    }
    
    /// Get an iterator over all sorts (excluding gap)
    pub fn iter(&self) -> SortBufferIterator {
        SortBufferIterator {
            buffer: self,
            index: 0,
        }
    }
    
    /// Clear all sorts and reset gap
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = SortEntry::default();
        }
        self.gap_start = 0;
        self.gap_end = self.buffer.len();
    }
    
    /// Get all sorts as a vector (for debugging/serialization)
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<SortEntry> {
        let mut result = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            if let Some(sort) = self.get(i) {
                result.push(sort.clone());
            }
        }
        result
    }
}

/// Iterator for gap buffer
pub struct SortBufferIterator<'a> {
    buffer: &'a SortBuffer,
    index: usize,
}

impl<'a> Iterator for SortBufferIterator<'a> {
    type Item = &'a SortEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.buffer.len() {
            None
        } else {
            let item = self.buffer.get(self.index);
            self.index += 1;
            item
        }
    }
}

/// An entry in the sort buffer representing a glyph
#[derive(Clone, Debug)]
pub struct SortEntry {
    /// The name of the glyph this sort represents
    pub glyph_name: String,
    /// The advance width of the glyph (for spacing)
    pub advance_width: f32,
    /// Whether this sort is currently active (in edit mode with points showing)
    pub is_active: bool,
    /// Whether this sort is currently selected (handle is highlighted)
    pub is_selected: bool,
    /// Layout mode for this sort
    pub layout_mode: SortLayoutMode,
    /// Freeform position (only used when layout_mode is Freeform)
    pub freeform_position: Vec2,
    /// Buffer index (only used when layout_mode is Buffer)
    #[allow(dead_code)]
    pub buffer_index: Option<usize>,
    /// Whether this sort is a buffer root (first sort in a text buffer)
    pub is_buffer_root: bool,
    /// Cursor position within this buffer sequence (only for buffer roots)
    pub buffer_cursor_position: Option<usize>,
}

impl Default for SortEntry {
    fn default() -> Self {
        Self {
            glyph_name: String::new(),
            advance_width: 0.0,
            is_active: false,
            is_selected: false,
            layout_mode: SortLayoutMode::Text,
            freeform_position: Vec2::ZERO,
            buffer_index: None,
            is_buffer_root: false,
            buffer_cursor_position: None,
        }
    }
}

/// Grid layout configuration
#[derive(Clone)]
pub struct GridConfig {
    /// Number of sorts per row
    pub sorts_per_row: usize,
    /// Horizontal spacing between sorts
    pub horizontal_spacing: f32,
    /// Vertical spacing between rows
    pub vertical_spacing: f32,
    /// Starting position for the grid
    pub grid_origin: Vec2,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            sorts_per_row: 16,
            horizontal_spacing: 64.0,
            vertical_spacing: 400.0,
            grid_origin: Vec2::ZERO,
        }
    }
}

impl TextEditorState {
    /// Create a new text editor state from font data
    /// All initial sorts are created as freeform sorts arranged in a grid
    #[allow(dead_code)]
    pub fn from_font_data(font_data: &FontData) -> Self {
        // Convert font glyphs to sort entries in alphabetical order
        let mut glyph_names: Vec<_> = font_data.glyphs.keys().collect();
        glyph_names.sort();
        
        let mut sorts = Vec::new();
        let grid_config = GridConfig::default();
        
        for (index, glyph_name) in glyph_names.iter().enumerate() {
            if let Some(glyph_data) = font_data.glyphs.get(*glyph_name) {
                // Calculate freeform position in overview grid
                let row = index / grid_config.sorts_per_row;
                let col = index % grid_config.sorts_per_row;
                
                let freeform_position = Vec2::new(
                    grid_config.grid_origin.x + col as f32 * (1000.0 + grid_config.horizontal_spacing),
                    grid_config.grid_origin.y - row as f32 * (1200.0 + grid_config.vertical_spacing),
                );
                
                sorts.push(SortEntry {
                    glyph_name: glyph_name.to_string(),
                    advance_width: glyph_data.advance_width as f32,
                    is_active: false,
                    is_selected: false,
                    layout_mode: SortLayoutMode::Freeform, // Changed to Freeform
                    freeform_position, // Set actual position instead of Vec2::ZERO
                    buffer_index: None, // No buffer index for freeform sorts
                    is_buffer_root: false, // Overview sorts are not buffer roots
                    buffer_cursor_position: None, // No cursor for freeform sorts
                });
            }
        }
        
        let buffer = SortBuffer::from_sorts(sorts);
        
        Self {
            buffer,
            cursor_position: 0,
            selection: None,
            viewport_offset: Vec2::ZERO,
            grid_config,
        }
    }
    
    /// Get all sorts (both buffer and freeform)
    #[allow(dead_code)]
    pub fn get_all_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut all_sorts = Vec::new();
        
        // Add buffer sorts
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                all_sorts.push((i, sort));
            }
        }
        
        all_sorts
    }
    
    /// Get only text sorts (sorts that flow like text)
    pub fn get_text_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut text_sorts = Vec::new();
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Text {
                    text_sorts.push((i, sort));
                }
            }
        }
        
        text_sorts
    }
    
    /// Get only freeform sorts
    #[allow(dead_code)]
    pub fn get_freeform_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut freeform_sorts = Vec::new();
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Freeform {
                    freeform_sorts.push((i, sort));
                }
            }
        }
        
        freeform_sorts
    }
    
    /// Convert a sort from text mode to freeform mode
    #[allow(dead_code)]
    pub fn convert_sort_to_freeform(&mut self, buffer_position: usize, freeform_position: Vec2) -> bool {
        if let Some(sort) = self.buffer.get_mut(buffer_position) {
            sort.layout_mode = SortLayoutMode::Freeform;
            sort.freeform_position = freeform_position;
            sort.buffer_index = None;
            true
        } else {
            false
        }
    }
    
    /// Convert a sort from freeform mode to text mode
    #[allow(dead_code)]
    pub fn convert_sort_to_text(&mut self, buffer_position: usize, new_buffer_index: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(buffer_position) {
            sort.layout_mode = SortLayoutMode::Text;
            sort.freeform_position = Vec2::ZERO;
            sort.buffer_index = Some(new_buffer_index);
            true
        } else {
            false
        }
    }
    
    /// Add a new freeform sort at the specified position
    pub fn add_freeform_sort(&mut self, glyph_name: String, position: Vec2, advance_width: f32) {
        let sort = SortEntry {
            glyph_name,
            advance_width,
            is_active: false,
            is_selected: false,
            layout_mode: SortLayoutMode::Freeform,
            freeform_position: position,
            buffer_index: None,
            is_buffer_root: false,
            buffer_cursor_position: None,
        };
        
        // Insert at the end of the buffer
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, sort);
    }
    
    /// Calculate position for text sorts that flow from their text root
    fn get_text_sort_flow_position(&self, buffer_position: usize) -> Option<Vec2> {
        // Find the text root for this sort
        let mut text_root_position = None;
        let mut text_root_pos = Vec2::ZERO;
        
        // Look backwards to find the text root
        for i in (0..=buffer_position).rev() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Text && sort.is_buffer_root {
                    text_root_position = Some(i);
                    text_root_pos = sort.freeform_position;
                    break;
                }
            }
        }
        
        if let Some(root_pos) = text_root_position {
            // Calculate horizontal offset from text root
            let mut x_offset = 0.0;
            
            // Sum up advance widths from root to current position
            for i in root_pos..buffer_position {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.layout_mode == SortLayoutMode::Text && !sort.glyph_name.is_empty() {
                        x_offset += sort.advance_width;
                    }
                }
            }
            
            Some(text_root_pos + Vec2::new(x_offset, 0.0))
        } else {
            // Fallback to stored position if no text root found
            self.buffer.get(buffer_position).map(|sort| sort.freeform_position)
        }
    }
    
    /// Create a new text root at the specified world position
    pub fn create_text_root(&mut self, world_position: Vec2) {
        // Create an empty text root sort
        let text_root = SortEntry {
            glyph_name: String::new(), // Empty - will be populated when user types
            advance_width: 0.0,
            is_active: false,
            is_selected: true, // Select the new text root
            layout_mode: SortLayoutMode::Text,
            freeform_position: world_position, // Store the actual position for text sorts too
            buffer_index: Some(self.buffer.len()), // Position in buffer
            is_buffer_root: true, // This is a text root
            buffer_cursor_position: Some(0), // Position cursor at the empty root for replacement
        };
        
        // Insert at the end of the buffer
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);
        
        // FIXED: Position cursor at the text root so typing replaces it and moves cursor forward
        // This ensures when user types the first character, cursor ends up after that character
        self.cursor_position = insert_index;
        
        info!("Created new text root at world position ({:.1}, {:.1}), cursor at position {}", 
              world_position.x, world_position.y, self.cursor_position);
    }
    
    /// Create a text sort at a specific world position (for text tool)
    pub fn create_text_sort_at_position(&mut self, glyph_name: String, world_position: Vec2, advance_width: f32) {
        // Always create each clicked text sort as a new text root to preserve exact positioning
        // This ensures text sorts stay exactly where clicked, just like freeform sorts
        let text_root = SortEntry {
            glyph_name: glyph_name.clone(),
            advance_width,
            is_active: false,
            is_selected: true, // Select the new text root
            layout_mode: SortLayoutMode::Text,
            freeform_position: world_position,
            buffer_index: Some(self.buffer.len()),
            is_buffer_root: true, // Always make clicked text sorts into roots
            buffer_cursor_position: Some(1), // Position cursor after the new sort for typing
        };
        
        let insert_index = self.buffer.len();
        self.buffer.insert(insert_index, text_root);
        
        // FIXED: Position cursor after the new sort for immediate typing continuation
        // Since we inserted a real glyph (not empty), cursor should be positioned to continue typing
        self.cursor_position = insert_index + 1;
        
        info!("Created new text root '{}' at world position ({:.1}, {:.1}), cursor at position {}", 
              glyph_name, world_position.x, world_position.y, self.cursor_position);
    }
    
    /// Get the visual position for a sort based on its layout mode
    pub fn get_sort_visual_position(&self, buffer_position: usize) -> Option<Vec2> {
        if let Some(sort) = self.buffer.get(buffer_position) {
            match sort.layout_mode {
                SortLayoutMode::Text => {
                    // Text sorts now use their stored freeform_position
                    // But we need to calculate relative positions for text flow
                    if sort.is_buffer_root {
                        // Text roots use their exact stored position
                        Some(sort.freeform_position)
                    } else {
                        // Non-root text sorts flow from their text root
                        self.get_text_sort_flow_position(buffer_position)
                    }
                }
                SortLayoutMode::Freeform => {
                    Some(sort.freeform_position)
                }
            }
        } else {
            None
        }
    }

    /// Find a sort handle at a given world position (for freeform sorts)
    pub fn find_sort_handle_at_position(
        &self,
        world_position: Vec2,
        tolerance: f32,
        font_metrics: Option<&FontMetrics>,
    ) -> Option<usize> {
        // Check handles for all sorts (both buffer and freeform have handles)
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    let descender = if let Some(metrics) = font_metrics {
                        metrics.descender.unwrap_or(-200.0) as f32
                    } else {
                        -200.0 // Default descender value
                    };
                    // Match the rendering logic exactly: handle_position = world_pos + Vec2::new(0.0, descender)
                    let handle_pos = sort_pos + Vec2::new(0.0, descender);
                    let distance = world_position.distance(handle_pos);
                    
                    debug!(
                        "Checking sort '{}' at index {}: sort_pos=({:.1}, {:.1}), handle_pos=({:.1}, {:.1}), click_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                        sort.glyph_name, i, sort_pos.x, sort_pos.y, handle_pos.x, handle_pos.y, world_position.x, world_position.y, distance, tolerance
                    );
                    
                    if distance < tolerance {
                        debug!("Found matching handle for sort '{}' at index {} (distance={:.1} < tolerance={:.1})", sort.glyph_name, i, distance, tolerance);
                        return Some(i);
                    }
                }
            }
        }
        debug!("No handle found at position ({:.1}, {:.1}) with tolerance {:.1}", world_position.x, world_position.y, tolerance);
        None
    }

    /// Find a sort body at a given world position
    pub fn find_sort_body_at_position(&self, world_position: Vec2, tolerance: f32) -> Option<usize> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if let Some(sort_pos) = self.get_sort_visual_position(i) {
                    if world_position.distance(sort_pos) < tolerance {
                        debug!("Found matching body for sort {} at index {}", sort.glyph_name, i);
                        return Some(i);
                    }
                }
            }
        }
        None
    }
    
    /// Get the sort at a specific buffer position
    pub fn get_sort_at_position(&self, position: usize) -> Option<&SortEntry> {
        self.buffer.get(position)
    }
    
    /// Get the currently active sort
    pub fn get_active_sort(&self) -> Option<(usize, &SortEntry)> {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_active {
                    return Some((i, sort));
                }
            }
        }
        None
    }
    
    /// Activate a sort at the given buffer position (only one can be active)
    pub fn activate_sort(&mut self, position: usize) -> bool {
        // First deactivate all sorts
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        
        // Then activate the specified sort
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_active = true;
            debug!("Activated sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Select a sort at the given buffer position (multiple can be selected)
    pub fn select_sort(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = true;
            debug!("Selected sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Deselect a sort at the given buffer position
    #[allow(dead_code)]
    pub fn deselect_sort(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = false;
            debug!("Deselected sort '{}' at buffer position {}", sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Toggle selection state of a sort at the given buffer position
    pub fn toggle_sort_selection(&mut self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get_mut(position) {
            sort.is_selected = !sort.is_selected;
            let action = if sort.is_selected { "Selected" } else { "Deselected" };
            debug!("{} sort '{}' at buffer position {}", action, sort.glyph_name, position);
            true
        } else {
            false
        }
    }
    
    /// Clear all selections
    pub fn clear_selections(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_selected = false;
            }
        }
        debug!("Cleared all sort selections");
    }
    
    /// Get all currently selected sorts
    pub fn get_selected_sorts(&self) -> Vec<(usize, &SortEntry)> {
        let mut selected_sorts = Vec::new();
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_selected {
                    selected_sorts.push((i, sort));
                }
            }
        }
        selected_sorts
    }
    
    /// Check if a sort at the given position is selected
    #[allow(dead_code)]
    pub fn is_sort_selected(&self, position: usize) -> bool {
        if let Some(sort) = self.buffer.get(position) {
            sort.is_selected
        } else {
            false
        }
    }
    
    /// Clear active state from all sorts
    pub fn clear_active_state(&mut self) {
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
        debug!("Cleared active state from all sorts");
    }
    
    /// Get the visual position (world coordinates) for a buffer position
    #[allow(dead_code)]
    pub fn get_world_position_for_buffer_position(&self, buffer_position: usize) -> Vec2 {
        let row = buffer_position / self.grid_config.sorts_per_row;
        let col = buffer_position % self.grid_config.sorts_per_row;
        
        let x = col as f32 * (1000.0 + self.grid_config.horizontal_spacing);
        let y = -(row as f32) * (1200.0 + self.grid_config.vertical_spacing);
        
        self.grid_config.grid_origin + Vec2::new(x, y)
    }
    
    /// Get the buffer position for a world coordinate (for click detection)
    pub fn get_buffer_position_for_world_position(&self, world_pos: Vec2) -> Option<usize> {
        let relative_pos = world_pos - self.grid_config.grid_origin;
        
        // Calculate grid row and column
        let col = (relative_pos.x / (1000.0 + self.grid_config.horizontal_spacing)).floor() as usize;
        
        // Handle negative Y coordinates correctly for downward-growing grid
        let row = if relative_pos.y <= 0.0 {
            ((-relative_pos.y) / (1200.0 + self.grid_config.vertical_spacing)).floor() as usize
        } else {
            0
        };
        
        // Convert grid position to buffer position
        let buffer_position = row * self.grid_config.sorts_per_row + col;
        
        // Validate the position is within bounds
        if buffer_position < self.buffer.len() {
            Some(buffer_position)
        } else {
            None
        }
    }
    
    /// Insert a new sort at the cursor position (for typing)
    pub fn insert_sort_at_cursor(&mut self, glyph_name: String, advance_width: f32) {
        // Find an active buffer root using more robust logic
        let mut buffer_root_index = None;
        let mut cursor_pos_in_buffer = 0;
        
        // FIXED: Use more robust logic to find active buffer root
        // First try to find a selected buffer root
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    buffer_root_index = Some(i);
                    cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                    break;
                }
            }
        }
        
        // If no selected buffer root found, look for any buffer root with a cursor position
        if buffer_root_index.is_none() {
            for i in 0..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                        break;
                    }
                }
            }
        }
        
        // If still no buffer root found, look for the most recently added buffer root
        if buffer_root_index.is_none() {
            for i in (0..self.buffer.len()).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = 0; // Start at beginning if no cursor position set
                        // Set the cursor position on this buffer root
                        break;
                    }
                }
            }
        }
        
        if let Some(root_index) = buffer_root_index {
            // We have an active buffer root - handle insertion within this buffer sequence
            let actual_insert_position = root_index + cursor_pos_in_buffer;
            
            // Check if we're replacing an empty buffer root
            if cursor_pos_in_buffer == 0 {
                if let Some(root_sort) = self.buffer.get(root_index) {
                    if root_sort.glyph_name.is_empty() && root_sort.is_buffer_root {
                        // Replace the empty buffer root with the typed character
                        if let Some(sort) = self.buffer.get_mut(root_index) {
                            sort.glyph_name = glyph_name.clone();
                            sort.advance_width = advance_width;
                            sort.is_active = true;
                            // Update cursor position within this buffer
                            sort.buffer_cursor_position = Some(1);
                        }
                        info!("Replaced empty buffer root with '{}', cursor now at position 1", glyph_name);
                        return;
                    }
                }
            }
            
            // Calculate position for the new sort based on text flow from buffer root
            let buffer_root_position = self.buffer.get(root_index)
                .map(|sort| sort.freeform_position)
                .unwrap_or(Vec2::ZERO);
            
            // Calculate cumulative advance width up to cursor position
            let mut x_offset = 0.0;
            for i in 0..cursor_pos_in_buffer {
                let sort_index = root_index + i;
                if let Some(sort) = self.buffer.get(sort_index) {
                    if sort.layout_mode == SortLayoutMode::Text && !sort.glyph_name.is_empty() {
                        x_offset += sort.advance_width;
                    }
                }
            }
            
            let new_position = buffer_root_position + Vec2::new(x_offset, 0.0);
            
            let new_sort = SortEntry {
                glyph_name: glyph_name.clone(),
                advance_width,
                is_active: false,
                is_selected: false,
                layout_mode: SortLayoutMode::Text,
                freeform_position: new_position,
                buffer_index: Some(actual_insert_position),
                is_buffer_root: false,
                buffer_cursor_position: None,
            };
            
            // Insert the new sort
            self.buffer.insert(actual_insert_position, new_sort);
            
            // Update the buffer root's cursor position
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer + 1);
            }
            
            info!("Inserted '{}' at buffer position {}, cursor now at position {}", 
                  glyph_name, actual_insert_position, cursor_pos_in_buffer + 1);
        } else {
            // No active buffer root found - this shouldn't happen in normal typing flow
            warn!("No active buffer root found for typing - this should not happen");
        }
    }
    
    /// Delete the sort at the cursor position
    pub fn delete_sort_at_cursor(&mut self) {
        // FIXED: Use per-buffer-root cursor system instead of global cursor position
        // Find the active buffer root and delete from its cursor position
        let mut buffer_root_index = None;
        let mut cursor_pos_in_buffer = 0;
        
        // Find active buffer root using the same logic as insert_sort_at_cursor
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    buffer_root_index = Some(i);
                    cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                    break;
                }
            }
        }
        
        if buffer_root_index.is_none() {
            for i in 0..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                        break;
                    }
                }
            }
        }
        
        if buffer_root_index.is_none() {
            for i in (0..self.buffer.len()).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        buffer_root_index = Some(i);
                        cursor_pos_in_buffer = 0;
                        break;
                    }
                }
            }
        }
        
        if let Some(root_index) = buffer_root_index {
            if cursor_pos_in_buffer > 0 {
                let delete_position = root_index + cursor_pos_in_buffer - 1;
                if delete_position < self.buffer.len() {
                    self.buffer.delete(delete_position);
                    // Update the buffer root's cursor position
                    if let Some(root_sort) = self.buffer.get_mut(root_index) {
                        root_sort.buffer_cursor_position = Some(cursor_pos_in_buffer - 1);
                    }
                    info!("Deleted sort at position {}, cursor now at position {}", 
                          delete_position, cursor_pos_in_buffer - 1);
                }
            } else {
                // At position 0 - check if this is an empty buffer root we can delete
                if let Some(root_sort) = self.buffer.get(root_index) {
                    if root_sort.glyph_name.is_empty() && root_sort.is_buffer_root {
                        self.buffer.delete(root_index);
                        info!("Deleted empty buffer root at position {}", root_index);
                    }
                }
            }
        }
    }
    
    /// Move cursor to a specific position (now works with per-buffer-root cursors)
    pub fn move_cursor_to(&mut self, position: usize) {
        // FIXED: Update the active buffer root's cursor position instead of global cursor
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                // Clamp the position to valid range for this buffer sequence
                let clamped_position = position.min(max_pos);
                root_sort.buffer_cursor_position = Some(clamped_position);
                info!("Moved cursor to position {} in buffer root at index {}", 
                      clamped_position, root_index);
            }
        }
    }
    
    /// Move cursor left by one position
    pub fn move_cursor_left(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos > 0 {
                    root_sort.buffer_cursor_position = Some(current_pos - 1);
                    info!("Moved cursor left to position {} in buffer root at index {}", 
                          current_pos - 1, root_index);
                }
            }
        }
    }
    
    /// Move cursor right by one position
    pub fn move_cursor_right(&mut self) {
        // FIXED: Update the active buffer root's cursor position
        if let Some(root_index) = self.find_active_buffer_root_index() {
            // Get max_pos before getting mutable reference to avoid borrow conflicts
            let max_pos = self.get_buffer_sequence_length(root_index);
            if let Some(root_sort) = self.buffer.get_mut(root_index) {
                let current_pos = root_sort.buffer_cursor_position.unwrap_or(0);
                if current_pos < max_pos {
                    root_sort.buffer_cursor_position = Some(current_pos + 1);
                    info!("Moved cursor right to position {} in buffer root at index {}", 
                          current_pos + 1, root_index);
                }
            }
        }
    }
    
    /// Move cursor up by one row (for buffer mode, move to previous buffer root)
    pub fn move_cursor_up(&mut self) {
        // FIXED: In buffer mode, move to previous buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the previous buffer root
            for i in (0..current_root_index).rev() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        // Deselect current buffer root and select previous one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_selected = false;
                        }
                        // Get buffer_length before getting mutable reference to avoid borrow conflicts
                        let buffer_length = self.get_buffer_sequence_length(i);
                        if let Some(prev_root) = self.buffer.get_mut(i) {
                            prev_root.is_selected = true;
                            // Set cursor to end of previous buffer
                            prev_root.buffer_cursor_position = Some(buffer_length);
                            info!("Moved up to buffer root at index {}, cursor at position {}", 
                                  i, buffer_length);
                        }
                        return;
                    }
                }
            }
        }
    }
    
    /// Move cursor down by one row (for buffer mode, move to next buffer root)
    pub fn move_cursor_down(&mut self) {
        // FIXED: In buffer mode, move to next buffer root
        if let Some(current_root_index) = self.find_active_buffer_root_index() {
            // Find the next buffer root
            for i in (current_root_index + 1)..self.buffer.len() {
                if let Some(sort) = self.buffer.get(i) {
                    if sort.is_buffer_root {
                        // Deselect current buffer root and select next one
                        if let Some(current_root) = self.buffer.get_mut(current_root_index) {
                            current_root.is_selected = false;
                        }
                        if let Some(next_root) = self.buffer.get_mut(i) {
                            next_root.is_selected = true;
                            // Set cursor to beginning of next buffer
                            next_root.buffer_cursor_position = Some(0);
                            info!("Moved down to buffer root at index {}, cursor at position 0", i);
                        }
                        return;
                    }
                }
            }
        }
    }
    
    /// Helper: Find the index of the active buffer root
    fn find_active_buffer_root_index(&self) -> Option<usize> {
        // Use same logic as insert_sort_at_cursor
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.is_selected {
                    return Some(i);
                }
            }
        }
        
        for i in 0..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                    return Some(i);
                }
            }
        }
        
        for i in (0..self.buffer.len()).rev() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.is_buffer_root {
                    return Some(i);
                }
            }
        }
        
        None
    }
    
    /// Helper: Get the length of a buffer sequence starting from a buffer root
    fn get_buffer_sequence_length(&self, root_index: usize) -> usize {
        let mut length = 0;
        for i in root_index..self.buffer.len() {
            if let Some(sort) = self.buffer.get(i) {
                if sort.layout_mode == SortLayoutMode::Text {
                    if i == root_index || !sort.is_buffer_root {
                        length += 1;
                    } else {
                        // Hit another buffer root, stop counting
                        break;
                    }
                } else {
                    // Hit a freeform sort, stop counting
                    break;
                }
            }
        }
        length
    }
}


