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
use crate::selection::components::GlyphPointReference;

/// Default units per em value used when creating new fonts or when this value is missing
const DEFAULT_UNITS_PER_EM: f64 = 1024.;

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
    
    /// Gets a point reference from an entity
    ///
    /// Returns the GlyphPointReference for the given entity if it exists
    pub fn get_point_reference(&self, _entity: Entity) -> Option<GlyphPointReference> {
        // This would normally require querying the entity for its GlyphPointReference component
        // Since we can't do that directly from AppState, this is just a placeholder
        // In the actual implementation, we would rely on the system querying the component
        None
    }
    
    /// Gets a mutable reference to a point in the font data
    ///
    /// # Parameters
    /// - `point_ref`: Reference to the glyph point to update
    ///
    /// Returns a mutable reference to the ContourPoint if found
    pub fn get_point_mut(&mut self, point_ref: &GlyphPointReference) -> Option<&mut ContourPoint> {
        // Convert the glyph name from String to GlyphName
        let glyph_name = GlyphName::from(&*point_ref.glyph_name);
        
        // Try to get the glyph
        if let Some(default_layer) = self.workspace.font_mut().ufo.get_default_layer_mut() {
            if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
                // Get the outline
                if let Some(outline) = glyph.outline.as_mut() {
                    // Check if the contour index is valid
                    if point_ref.contour_index < outline.contours.len() {
                        let contour = &mut outline.contours[point_ref.contour_index];
                        
                        // Check if the point index is valid
                        if point_ref.point_index < contour.points.len() {
                            // Return mutable reference to the point
                            return Some(&mut contour.points[point_ref.point_index]);
                        }
                    }
                }
            }
        }
        
        None
    }
}

/// A workspace is a single font, corresponding to a UFO file on disk.
///
/// This structure maintains the state of the current editing session,
/// including the font data, selected glyphs, and open editor instances.
#[derive(Clone)]
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

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            font: Arc::new(FontObject::default()),
            selected: None,
            open_glyphs: Arc::new(HashMap::new()),
            info: SimpleFontInfo::default(),
        }
    }
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

/// Represents the spacing on either side of a glyph
///
/// Sidebearings control the horizontal spacing of glyphs:
/// - Left sidebearing: Space before the glyph
/// - Right sidebearing: Space after the glyph
///
/// # Usage in Bevy
/// This struct can be used in a glyph editor system to adjust spacing:
/// 1. Create a Bevy system that controls sidebearing adjustments
/// 2. Use it with UI sliders or input fields for direct editing
/// 3. Connect to a GlyphComponent to apply spacing changes
#[derive(Clone, Debug, PartialEq)]
pub struct Sidebearings {
    pub left: f64,
    pub right: f64,
}

impl Sidebearings {
    /// Creates a new Sidebearings instance
    #[allow(dead_code)]
    pub fn new(left: f64, right: f64) -> Self {
        Sidebearings { left, right }
    }

    /// Creates default sidebearings with zero spacing
    #[allow(dead_code)]
    pub fn zero() -> Self {
        Sidebearings {
            left: 0.0,
            right: 0.0,
        }
    }
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
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let font_obj = Arc::make_mut(&mut self.font);
        font_obj.update_info(&self.info);

        if let Some(path) = font_obj.path.as_ref() {
            // Write to a temporary location first
            let temp_path = temp_write_path(path);
            info!("saving to {:?}", temp_path);

            // Save the UFO to the temporary path
            font_obj.ufo.save(&temp_path)?;

            // Backup existing data if needed
            if let Some(backup_path) = backup_ufo_at_path(path)? {
                info!("backing up existing data to {:?}", backup_path);
            }

            // Move the temporary file to the target path
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

    /* COMMENTED OUT DUE TO NORAD API INCOMPATIBILITY
    /// Find a glyph by Unicode codepoint
    ///
    /// # Usage in Bevy
    /// Implement a character search function in the UI:
    /// ```
    /// fn search_character_system(
    ///     mut app_state: ResMut<AppState>,
    ///     mut char_input: ResMut<CharSearchState>,
    /// ) {
    ///     if let Some(search_char) = char_input.get_char() {
    ///         if let Some(glyph_name) = app_state.workspace.find_glyph_by_codepoint(search_char) {
    ///             // Select the found glyph
    ///             app_state.workspace.selected = Some(glyph_name);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn find_glyph_by_codepoint(&self, codepoint: char) -> Option<GlyphName> {
        // Implementation needs to be updated for norad's API
    }
    */

    /* COMMENTED OUT DUE TO INCOMPATIBILITY
    /// Create a new preview session
    ///
    /// # Usage in Bevy
    /// Create a preview panel in the UI:
    /// ```
    /// fn spawn_preview_panel(
    ///     mut commands: Commands,
    ///     mut app_state: ResMut<AppState>,
    /// ) {
    ///     let entity = commands.spawn(NodeBundle {
    ///         // UI configuration
    ///     }).id();
    ///
    ///     let preview_session = app_state.workspace.create_preview_session(entity);
    ///
    ///     commands.entity(entity).insert(PreviewSessionComponent(preview_session));
    /// }
    /// ```
    pub fn create_preview_session(&mut self, entity: Entity) -> PreviewSession {
        PreviewSession::new(entity)
    }

    /// Build a simple representation of this font for previews
    pub fn build_preview_data(&self) -> FontPreviewData {
        // Implementation needs to be updated to use the correct norad API
    }
    */
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

    /// Calculate the bounds of this path
    #[allow(dead_code)]
    pub fn bounds(&self) -> Rect {
        // Start with an empty rectangle
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // Current point state - prefix with underscore to indicate it's intentionally unused
        let mut _current = Vec2::ZERO;

        // Helper to update bounds with a point
        let mut update_bounds = |point: Vec2| {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        };

        // Process each command to find bounds
        for cmd in &self.path {
            match cmd {
                PathCommand::MoveTo(point) => {
                    _current = *point;
                    update_bounds(_current);
                }
                PathCommand::LineTo(point) => {
                    _current = *point;
                    update_bounds(_current);
                }
                PathCommand::QuadTo(control, point) => {
                    // For quadratic curves, we check the control point and end point
                    update_bounds(*control);
                    _current = *point;
                    update_bounds(_current);

                    // In a perfect implementation, we would also check points along the curve
                    // where the derivative is zero, but this is a reasonable approximation
                }
                PathCommand::CurveTo(control1, control2, point) => {
                    // For cubic curves, we check both control points and end point
                    update_bounds(*control1);
                    update_bounds(*control2);
                    _current = *point;
                    update_bounds(_current);

                    // In a perfect implementation, we would also check points along the curve
                    // where the derivative is zero, but this is a reasonable approximation
                }
                PathCommand::ClosePath => {
                    // ClosePath doesn't change bounds
                }
            }
        }

        // If we didn't find any points, return a zero rect
        if min_x == f32::MAX {
            return Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
        }

        // Create rectangle from min/max values
        Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }
}

impl GlyphDetail {
    /// Calculate the sidebearings for this glyph
    #[allow(dead_code)]
    pub fn compute_sidebearings(&self) -> Sidebearings {
        // If the glyph has an outline
        if let Some(_outline) = &self.glyph.outline {
            // Use advance instead of width since that's what norad provides
            let advance_width = self
                .glyph
                .advance
                .as_ref()
                .map(|v| v.width as f64)
                .unwrap_or(0.0);

            // Calculate bounds of the glyph's outline
            let bounds = self.compute_bounds();

            // Left sidebearing is the left edge of the glyph outline
            let left = bounds.min.x as f64;

            // Right sidebearing is the distance from the right edge
            // of the outline to the advance width
            let right = advance_width - bounds.max.x as f64;

            Sidebearings::new(left, right)
        } else {
            // Default sidebearings for glyphs without outlines
            Sidebearings::zero()
        }
    }

    /// Calculate the bounds of the glyph's outline
    #[allow(dead_code)]
    pub fn compute_bounds(&self) -> Rect {
        // Use the BezPath's bounds method
        self.outline.bounds()
    }

    /// Calculate the layout bounds of the glyph
    ///
    /// This includes both the outline and the advance width
    #[allow(dead_code)]
    pub fn layout_bounds(&self) -> Rect {
        // Get the outline bounds
        let mut bounds = self.compute_bounds();

        // If the glyph has an advance, extend the bounds to include it
        if let Some(advance) = &self.glyph.advance {
            let w = advance.width as f32;
            if bounds.max.x < w {
                bounds.max.x = w;
            }
        }

        bounds
    }

    /// Get the advance width of the glyph
    #[allow(dead_code)]
    pub fn advance(&self) -> f64 {
        self.glyph
            .advance
            .as_ref()
            .map(|v| v.width as f64)
            .unwrap_or(0.0)
    }

    /// Set the advance width of the glyph
    #[allow(dead_code)]
    pub fn set_advance(&mut self, width: f64) {
        let glyph = Arc::make_mut(&mut self.glyph);
        // Use the correct way to set advance in norad
        glyph.advance = Some(norad::Advance {
            width: width as f32,
            height: 0.0, // Height is a required field, not optional
        });
    }
}

/// A preview session for testing fonts with custom text
///
/// # Usage in Bevy
/// Create a preview system that renders text using the current font:
/// ```
/// fn update_preview_system(
///     app_state: Res<AppState>,
///     mut preview_query: Query<(&PreviewTag, &mut Text)>,
///     preview_sessions: Query<(&Entity, &PreviewSessionComponent)>,
/// ) {
///     // For each preview session
///     // Get the preview data from app_state
///     // Update the Text component with the rendered glyphs
/// }
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct PreviewSession {
    /// The font size to render at (in pixels)
    pub font_size: f64,
    /// The text to display
    pub text: Arc<String>,
    /// Entity associated with this preview in the UI
    pub entity: Entity,
}

impl PreviewSession {
    /// Create a new preview session with default values
    #[allow(dead_code)]
    pub fn new(entity: Entity) -> Self {
        PreviewSession {
            font_size: 72.0,
            text: Arc::new(
                "The quick brown fox jumps over the lazy dog".to_string(),
            ),
            entity,
        }
    }

    /// Set the text to display
    #[allow(dead_code)]
    pub fn set_text(&mut self, text: String) {
        self.text = Arc::new(text);
    }

    /// Set the font size
    #[allow(dead_code)]
    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size.max(1.0); // Ensure size is at least 1
    }
}

/// A simplified glyph representation for preview rendering
///
/// # Usage in Bevy
/// This can be used in a text preview rendering system:
/// ```
/// fn render_preview_text(
///     preview_data: &FontPreviewData,
///     text: &str,
///     font_size: f64,
/// ) -> Vec<Entity> {
///     // For each character in text
///     // Get the corresponding glyph from preview_data
///     // Create entity with the glyph path scaled to font_size
///     // Position correctly based on advance width
/// }
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct PreviewGlyph {
    /// The glyph name
    pub name: GlyphName,
    /// Unicode codepoints associated with this glyph
    pub codepoints: Vec<char>,
    /// The bezier path representing the glyph
    pub path: Arc<BezPath>,
    /// The advance width of the glyph
    pub advance: f64,
}

/// Simplified font data for preview rendering
///
/// # Usage in Bevy
/// Create a resource that caches this data and updates when the font changes:
/// ```
/// fn update_preview_data_system(
///     app_state: Res<AppState>,
///     mut preview_data: ResMut<FontPreviewResource>,
/// ) {
///     // Check if font has changed since last update
///     // If so, update the preview_data with app_state.workspace.build_preview_data()
/// }
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct FontPreviewData {
    /// Map of glyph name to preview glyph data
    pub glyphs: HashMap<GlyphName, PreviewGlyph>,
    /// Font metrics for proper rendering
    pub metrics: FontMetrics,
    /// Font family name
    pub family_name: String,
    /// Font style name
    pub style_name: String,
}

/// Represents a 2D transformation matrix
///
/// # Usage in Bevy
/// Create transformation tools in the glyph editor:
/// ```
/// fn glyph_transform_system(
///     mut app_state: ResMut<AppState>,
///     input: Res<Input<KeyCode>>,
///     mouse: Res<MouseInput>,
/// ) {
///     // Create appropriate Transform based on input
///     // Apply to selected glyph using GlyphDetail::transform_outline
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Scale X component
    pub xx: f64,
    /// Shear Y component
    pub xy: f64,
    /// Shear X component
    pub yx: f64,
    /// Scale Y component
    pub yy: f64,
    /// Translation X component
    pub tx: f64,
    /// Translation Y component
    pub ty: f64,
}

impl Transform {
    /// Create a new identity transform
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Transform {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Create a translation transform
    #[allow(dead_code)]
    pub fn translate(tx: f64, ty: f64) -> Self {
        Transform {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
            tx,
            ty,
        }
    }

    /// Create a scaling transform
    #[allow(dead_code)]
    pub fn scale(sx: f64, sy: f64) -> Self {
        Transform {
            xx: sx,
            xy: 0.0,
            yx: 0.0,
            yy: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Create a rotation transform (angle in radians)
    #[allow(dead_code)]
    pub fn rotate(angle: f64) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        Transform {
            xx: cos_angle,
            xy: -sin_angle,
            yx: sin_angle,
            yy: cos_angle,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Transform a point using this transform
    #[allow(dead_code)]
    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let new_x = x * self.xx + y * self.xy + self.tx;
        let new_y = x * self.yx + y * self.yy + self.ty;
        (new_x, new_y)
    }

    /// Concatenate this transform with another
    #[allow(dead_code)]
    pub fn concat(&self, other: &Transform) -> Transform {
        Transform {
            xx: self.xx * other.xx + self.xy * other.yx,
            xy: self.xx * other.xy + self.xy * other.yy,
            yx: self.yx * other.xx + self.yy * other.yx,
            yy: self.yx * other.xy + self.yy * other.yy,
            tx: self.tx * other.xx + self.ty * other.yx + other.tx,
            ty: self.tx * other.xy + self.ty * other.yy + other.ty,
        }
    }
}
