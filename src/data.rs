//! # Bezy Font Editor - Data Model
//! 
//! This file defines the core data structures that represent the state of the Bezy font editor 
//! application. The data model follows a hierarchical structure:
//! 
//! - `AppState`: The top-level application state containing the current workspace
//! - `Workspace`: Represents an editing session for a single font (UFO file)
//! - `FontObject`: Wraps a UFO font and its file path
//! - `SimpleFontInfo` and `FontMetrics`: Store font metadata and metrics
//! - `GlyphDetail`: Contains information about a specific glyph
//! - `BezPath` and `PathCommand`: Represent Bezier paths for glyph outlines
//! 
//! The module also provides utility functions for font file operations (saving, backup)
//! and conversion between UFO's representation of paths and Bezy's internal BezPath format.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;
use norad::glyph::{ContourPoint, Glyph, GlyphName, PointType};
use norad::{FontInfo, Ufo};

/// Default units per em value used when creating new fonts or when this value is missing
const DEFAULT_UNITS_PER_EM: f64 = 1000.;

/// The top level application state.
/// 
/// This is the main state container for the application, stored as a Bevy Resource.
/// It holds the current workspace and provides methods to interact with it.
#[derive(Resource, Default, Clone)]
pub struct AppState {
    pub workspace: Workspace,
}

impl AppState {
    /// Sets a new font in the workspace
    /// 
    /// # Parameters
    /// - `ufo`: The UFO font to set
    /// - `path`: Optional file path for the font on disk
    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.workspace.set_file(ufo, path);
    }

    /// Returns a display name for the current font
    /// 
    /// The display name is generated from family and style names
    pub fn get_font_display_name(&self) -> String {
        self.workspace.info.get_display_name()
    }
}

/// A workspace is a single font, corresponding to a UFO file on disk.
/// 
/// This structure maintains the state of the current editing session,
/// including the font data, selected glyphs, and open editor instances.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct Workspace {
    /// The font being edited, wrapped in an Arc for efficient sharing
    pub font: Arc<FontObject>,
    /// The currently selected glyph (in the main glyph list) if any
    pub selected: Option<GlyphName>,
    /// Glyphs that are already open in an editor, mapped to their UI entity
    pub open_glyphs: Arc<HashMap<GlyphName, Entity>>,
    /// Simplified font information for quick access to common properties
    pub info: SimpleFontInfo,
}

/// Represents a UFO font and its file path
/// 
/// This structure wraps the norad::Ufo type and associates it with
/// an optional file path on disk.
#[derive(Clone)]
#[allow(dead_code)]
pub struct FontObject {
    /// Optional path to the UFO file on disk (None for unsaved fonts)
    pub path: Option<Arc<Path>>,
    /// The actual UFO font data from the norad library
    pub ufo: Ufo,
}

/// Detailed information about a specific glyph.
/// 
/// This structure contains all the information needed to display
/// and edit a glyph, including its outline and font metrics.
#[derive(Clone)]
#[allow(dead_code)]
pub struct GlyphDetail {
    /// The glyph data from norad
    pub glyph: Arc<Glyph>,
    /// The bezier path representing the glyph's outline
    pub outline: Arc<BezPath>,
    /// Font metrics needed for properly displaying the glyph
    pub metrics: FontMetrics,
    /// Whether this glyph is a placeholder or a real glyph
    pub is_placeholder: bool,
}

/// Simplified font information for UI and quick access
/// 
/// Contains the most commonly accessed properties of a font
/// without needing to access the full UFO structure.
#[derive(Clone)]
pub struct SimpleFontInfo {
    /// Font metrics such as units per em, ascender, descender, etc.
    pub metrics: FontMetrics,
    /// The font family name (e.g., "Helvetica")
    pub family_name: String,
    /// The font style name (e.g., "Bold", "Italic")
    pub style_name: String,
}

/// Font metrics relevant during editing or drawing.
/// 
/// Contains the key measurements that define a font's proportions
/// and are needed when editing or rendering glyphs.
#[derive(Clone)]
pub struct FontMetrics {
    /// The font's units per em value, which defines the coordinate space
    pub units_per_em: f64,
    /// The font's descender value (distance below the baseline)
    pub descender: Option<f64>,
    /// Height of lowercase letters like 'x'
    pub x_height: Option<f64>,
    /// Height of capital letters
    pub cap_height: Option<f64>,
    /// The font's ascender value (distance above the baseline)
    pub ascender: Option<f64>,
    /// The slant angle for italic fonts (in degrees)
    pub italic_angle: Option<f64>,
}

impl Workspace {
    /// Sets a new font file in the workspace
    /// 
    /// Creates a new FontObject from the provided UFO and path,
    /// then updates the workspace's font and info.
    pub fn set_file(&mut self, ufo: Ufo, path: impl Into<Option<PathBuf>>) {
        let obj = FontObject {
            path: path.into().map(Into::into),
            ufo,
        };
        self.font = obj.into();
        self.info = SimpleFontInfo::from_font(&self.font);
    }

    /// Saves the current font to disk
    /// 
    /// This method:
    /// 1. Updates the font object with current info
    /// 2. Writes to a temporary file first
    /// 3. Backs up existing data if present
    /// 4. Renames the temporary file to the final path
    #[allow(dead_code)]
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let font_obj = Arc::make_mut(&mut self.font);
        font_obj.update_info(&self.info);

        if let Some(path) = font_obj.path.as_ref() {
            // Write to a temporary location first
            let temp_path = temp_write_path(path);
            info!("saving to {:?}", temp_path);
            font_obj.ufo.save(&temp_path)?;

            // Backup existing data if needed
            if let Some(backup_path) = backup_ufo_at_path(path)? {
                info!("backing up existing data to {:?}", backup_path);
            }

            std::fs::rename(&temp_path, path)?;
            Ok(())
        } else {
            Err("save called with no path set".into())
        }
    }

    /// Gets the units per em value for the current font
    /// 
    /// Returns the default value if not specified in the font
    #[allow(dead_code)]
    pub fn units_per_em(&self) -> f64 {
        self.font
            .ufo
            .font_info
            .as_ref()
            .and_then(|info| info.units_per_em.map(|v| v.get() as f64))
            .unwrap_or(DEFAULT_UNITS_PER_EM)
    }

    /// Get a mutable reference to the font object
    /// 
    /// Uses Arc::make_mut to ensure unique ownership when mutating
    #[allow(dead_code)]
    pub fn font_mut(&mut self) -> &mut FontObject {
        Arc::make_mut(&mut self.font)
    }
}

impl FontObject {
    /// Updates the UFO font info with values from SimpleFontInfo
    /// 
    /// This method is used before saving to ensure the UFO data
    /// reflects any changes made through the editor UI.
    #[allow(dead_code)]
    fn update_info(&mut self, info: &SimpleFontInfo) {
        let existing_info = SimpleFontInfo::from_font(self);
        if existing_info != *info {
            let font_info =
                self.ufo.font_info.get_or_insert_with(FontInfo::default);

            // Only update fields that have changed to avoid unnecessary modifications
            if existing_info.family_name != info.family_name {
                font_info.family_name = Some(info.family_name.clone());
            }
            if existing_info.style_name != info.style_name {
                font_info.style_name = Some(info.style_name.clone());
            }
            if existing_info.metrics.units_per_em != info.metrics.units_per_em {
                font_info.units_per_em = Some(
                    norad::NonNegativeIntegerOrFloat::try_from(
                        info.metrics.units_per_em,
                    )
                    .unwrap(),
                );
            }
            if existing_info.metrics.descender != info.metrics.descender {
                font_info.descender = info
                    .metrics
                    .descender
                    .map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.ascender != info.metrics.ascender {
                font_info.ascender = info
                    .metrics
                    .ascender
                    .map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.x_height != info.metrics.x_height {
                font_info.x_height = info
                    .metrics
                    .x_height
                    .map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.cap_height != info.metrics.cap_height {
                font_info.cap_height = info
                    .metrics
                    .cap_height
                    .map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.italic_angle != info.metrics.italic_angle {
                font_info.italic_angle = info
                    .metrics
                    .italic_angle
                    .map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
        }
    }
}

impl Default for FontObject {
    /// Creates a default font object with minimal settings
    /// 
    /// Sets up a new UFO with a default family name of "Untitled"
    fn default() -> FontObject {
        let font_info = FontInfo {
            family_name: Some(String::from("Untitled")),
            ..Default::default()
        };

        let mut ufo = Ufo::new();
        ufo.font_info = Some(font_info);

        FontObject { path: None, ufo }
    }
}

impl SimpleFontInfo {
    /// Creates a SimpleFontInfo from a FontObject
    /// 
    /// Extracts relevant information from the UFO's font_info,
    /// providing defaults for missing values.
    fn from_font(font: &FontObject) -> Self {
        SimpleFontInfo {
            family_name: font
                .ufo
                .font_info
                .as_ref()
                .and_then(|f| f.family_name.as_ref().map(|s| s.to_string()))
                .unwrap_or_else(|| "Untitled".to_string()),
            style_name: font
                .ufo
                .font_info
                .as_ref()
                .and_then(|f| f.style_name.as_ref().map(|s| s.to_string()))
                .unwrap_or_else(|| "Regular".to_string()),
            metrics: font
                .ufo
                .font_info
                .as_ref()
                .map(FontMetrics::from)
                .unwrap_or_default(),
        }
    }

    /// Gets a display name combining family and style names
    /// 
    /// Returns "Untitled Font" if both family and style names are empty
    pub fn get_display_name(&self) -> String {
        if self.family_name.is_empty() && self.style_name.is_empty() {
            "Untitled Font".to_string()
        } else {
            format!("{} {}", self.family_name, self.style_name)
        }
    }
}

impl Default for SimpleFontInfo {
    /// Creates default SimpleFontInfo with empty name fields
    fn default() -> Self {
        SimpleFontInfo {
            metrics: Default::default(),
            family_name: String::new(),
            style_name: String::new(),
        }
    }
}

impl PartialEq for SimpleFontInfo {
    /// Compares two SimpleFontInfo instances for equality
    /// 
    /// Used to detect if font info has changed and needs updating
    fn eq(&self, other: &Self) -> bool {
        self.family_name == other.family_name
            && self.style_name == other.style_name
            && self.metrics == other.metrics
    }
}

impl From<&FontInfo> for FontMetrics {
    /// Converts from norad's FontInfo to our simplified FontMetrics
    /// 
    /// Extracts the relevant metrics and provides default values when needed
    fn from(src: &FontInfo) -> FontMetrics {
        FontMetrics {
            units_per_em: src
                .units_per_em
                .map(|v| v.get() as f64)
                .unwrap_or(DEFAULT_UNITS_PER_EM),
            descender: src.descender.map(|v| v.get() as f64),
            x_height: src.x_height.map(|v| v.get() as f64),
            cap_height: src.cap_height.map(|v| v.get() as f64),
            ascender: src.ascender.map(|v| v.get() as f64),
            italic_angle: src.italic_angle.map(|v| v.get() as f64),
        }
    }
}

impl Default for FontMetrics {
    /// Creates default FontMetrics with standard values
    /// 
    /// Sets units_per_em to the default value and leaves other metrics as None
    fn default() -> Self {
        FontMetrics {
            units_per_em: DEFAULT_UNITS_PER_EM,
            descender: None,
            x_height: None,
            cap_height: None,
            ascender: None,
            italic_angle: None,
        }
    }
}

impl PartialEq for FontMetrics {
    /// Compares two FontMetrics instances for equality
    /// 
    /// Used to detect if metrics have changed and need updating
    fn eq(&self, other: &Self) -> bool {
        self.units_per_em == other.units_per_em
            && self.descender == other.descender
            && self.x_height == other.x_height
            && self.cap_height == other.cap_height
            && self.ascender == other.ascender
            && self.italic_angle == other.italic_angle
    }
}

/// Convert a glyph's path from the UFO representation into a `BezPath`
/// 
/// This function translates the contour-based representation used by UFO
/// into our own BezPath format for rendering and editing.
#[allow(dead_code)]
pub(crate) fn path_for_glyph(glyph: &Glyph) -> Option<BezPath> {
    let mut path = BezPath::new();
    if let Some(outline) = &glyph.outline {
        for contour in &outline.contours {
            // Find the first non-off-curve point to start the path
            let mut close: Option<&ContourPoint> = None;

            let start_idx = match contour
                .points
                .iter()
                .position(|pt| pt.typ != PointType::OffCurve)
            {
                Some(idx) => idx,
                None => continue,
            };

            let first = &contour.points[start_idx];
            path.move_to((first.x as f64, first.y as f64));
            if first.typ != PointType::Move {
                close = Some(first);
            }

            let mut controls = Vec::with_capacity(2);

            // Helper function to add curves to the path
            let mut add_curve =
                |to_point: &ContourPoint, controls: &mut Vec<ContourPoint>| {
                    match controls.as_slice() {
                        &[] => {
                            path.line_to((to_point.x as f64, to_point.y as f64))
                        }
                        &[ref a] => path.quad_to(
                            (a.x as f64, a.y as f64),
                            (to_point.x as f64, to_point.y as f64),
                        ),
                        &[ref a, ref b] => path.curve_to(
                            (a.x as f64, a.y as f64),
                            (b.x as f64, b.y as f64),
                            (to_point.x as f64, to_point.y as f64),
                        ),
                        _illegal => {
                            panic!("existence of second point implies first")
                        }
                    };
                    controls.clear();
                };

            // Process each point in the contour
            let mut idx = (start_idx + 1) % contour.points.len();
            while idx != start_idx {
                let next = &contour.points[idx];
                match next.typ {
                    PointType::OffCurve => controls.push(next.clone()),
                    PointType::Line => {
                        debug_assert!(
                            controls.is_empty(),
                            "line type cannot follow offcurve"
                        );
                        add_curve(next, &mut controls);
                    }
                    PointType::Curve => add_curve(next, &mut controls),
                    PointType::QCurve => {
                        warn!("quadratic curves are currently ignored");
                        add_curve(next, &mut controls);
                    }
                    PointType::Move => {
                        debug_assert!(false, "illegal move point in path?")
                    }
                }
                idx = (idx + 1) % contour.points.len();
            }

            // Close the path if needed
            if let Some(to_close) = close {
                add_curve(to_close, &mut controls);
                path.close_path();
            }
        }
    }
    Some(path)
}

/// Creates a backup of a UFO file before saving
/// 
/// This function:
/// 1. Creates a backup directory if it doesn't exist
/// 2. Generates a timestamped backup filename
/// 3. Moves the original file to the backup location
/// 
/// Returns the path to the backup file if created
#[allow(dead_code)]
fn backup_ufo_at_path(path: &Path) -> Result<Option<PathBuf>, std::io::Error> {
    if !path.exists() {
        return Ok(None);
    }
    let backup_dir = format!(
        "{}_backups",
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
    );
    let mut backup_dir = path.with_file_name(backup_dir);
    if !backup_dir.exists() {
        std::fs::create_dir(&backup_dir)?;
    }

    let backup_date = chrono::Local::now();
    let date_str = backup_date.format("%Y-%m-%d_%Hh%Mm%Ss.ufo");
    backup_dir.push(date_str.to_string());
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }
    std::fs::rename(path, &backup_dir)?;
    Ok(Some(backup_dir))
}

/// Generates a temporary path for saving a UFO file
/// 
/// Creates a unique timestamped filename in the same directory
/// as the original file for atomic save operations.
#[allow(dead_code)]
fn temp_write_path(path: &Path) -> PathBuf {
    let mut n = 0;
    let backup_date = chrono::Local::now();
    let date_str = backup_date.format("%Y-%m-%d_%Hh%Mm%Ss");
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");
    loop {
        let file_path = format!("{}-savefile-{}_{}.ufo", stem, date_str, n);
        let full_path = path.with_file_name(file_path);
        if !full_path.exists() {
            break full_path;
        }
        n += 1;
    }
}

/// Represents a Bezier path for glyph outlines
/// 
/// This custom path representation is used for rendering glyphs
/// in the editor and is compatible with Bevy's Entity Component System.
#[derive(Component)]
pub struct BezPath {
    /// The sequence of path commands that define the shape
    pub path: Vec<PathCommand>,
}

/// Commands that make up a Bezier path
/// 
/// These correspond to the standard SVG path commands
/// and represent the different segments in a Bezier path.
#[derive(Clone, Component)]
#[allow(dead_code)]
pub enum PathCommand {
    /// Move to a point without drawing
    #[allow(dead_code)]
    MoveTo(Vec2),
    /// Draw a straight line to a point
    #[allow(dead_code)]
    LineTo(Vec2),
    /// Draw a quadratic Bezier curve with one control point
    #[allow(dead_code)]
    QuadTo(Vec2, Vec2),
    /// Draw a cubic Bezier curve with two control points
    #[allow(dead_code)]
    CurveTo(Vec2, Vec2, Vec2),
    /// Close the current path
    ClosePath,
}

impl BezPath {
    /// Creates a new empty BezPath
    pub fn new() -> Self {
        BezPath { path: Vec::new() }
    }

    /// Adds a MoveTo command to the path
    pub fn move_to(&mut self, to: (f64, f64)) {
        self.path
            .push(PathCommand::MoveTo(Vec2::new(to.0 as f32, to.1 as f32)));
    }

    /// Adds a LineTo command to the path
    pub fn line_to(&mut self, to: (f64, f64)) {
        self.path
            .push(PathCommand::LineTo(Vec2::new(to.0 as f32, to.1 as f32)));
    }

    /// Adds a QuadTo command to the path
    pub fn quad_to(&mut self, ctrl: (f64, f64), to: (f64, f64)) {
        self.path.push(PathCommand::QuadTo(
            Vec2::new(ctrl.0 as f32, ctrl.1 as f32),
            Vec2::new(to.0 as f32, to.1 as f32),
        ));
    }

    /// Adds a CurveTo command to the path
    pub fn curve_to(
        &mut self,
        ctrl1: (f64, f64),
        ctrl2: (f64, f64),
        to: (f64, f64),
    ) {
        self.path.push(PathCommand::CurveTo(
            Vec2::new(ctrl1.0 as f32, ctrl1.1 as f32),
            Vec2::new(ctrl2.0 as f32, ctrl2.1 as f32),
            Vec2::new(to.0 as f32, to.1 as f32),
        ));
    }

    /// Adds a ClosePath command to the path
    pub fn close_path(&mut self) {
        self.path.push(PathCommand::ClosePath);
    }
}
