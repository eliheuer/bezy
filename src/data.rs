//! Application state.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;
use norad::glyph::{ContourPoint, Glyph, GlyphName, PointType};
use norad::{FontInfo, Ufo};

/// This is by convention.
const DEFAULT_UNITS_PER_EM: f64 = 1000.;

/// The top level application state.
#[derive(Resource, Default, Clone)]
pub struct AppState {
    pub workspace: Workspace,
}

impl AppState {
    pub fn set_font(&mut self, ufo: Ufo, path: Option<PathBuf>) {
        self.workspace.set_file(ufo, path);
    }

    pub fn get_font_display_name(&self) -> String {
        self.workspace.info.get_display_name()
    }
}

/// A workspace is a single font, corresponding to a UFO file on disk.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct Workspace {
    pub font: Arc<FontObject>,
    /// The currently selected glyph (in the main glyph list) if any.
    pub selected: Option<GlyphName>,
    /// glyphs that are already open in an editor
    pub open_glyphs: Arc<HashMap<GlyphName, Entity>>,
    pub info: SimpleFontInfo,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct FontObject {
    pub path: Option<Arc<Path>>,
    pub ufo: Ufo,
}

/// Detailed information about a specific glyph.
#[derive(Clone)]
#[allow(dead_code)]
pub struct GlyphDetail {
    pub glyph: Arc<Glyph>,
    pub outline: Arc<BezPath>,
    pub metrics: FontMetrics,
    pub is_placeholder: bool,
}

#[derive(Clone)]
pub struct SimpleFontInfo {
    pub metrics: FontMetrics,
    pub family_name: String,
    pub style_name: String,
}

/// Things in `FontInfo` that are relevant while editing or drawing.
#[derive(Clone)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub descender: Option<f64>,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
    pub ascender: Option<f64>,
    pub italic_angle: Option<f64>,
}

impl Workspace {
    pub fn set_file(&mut self, ufo: Ufo, path: impl Into<Option<PathBuf>>) {
        let obj = FontObject {
            path: path.into().map(Into::into),
            ufo,
        };
        self.font = obj.into();
        self.info = SimpleFontInfo::from_font(&self.font);
    }

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

    #[allow(dead_code)]
    pub fn units_per_em(&self) -> f64 {
        self.font
            .ufo
            .font_info
            .as_ref()
            .and_then(|info| info.units_per_em.map(|v| v.get() as f64))
            .unwrap_or(DEFAULT_UNITS_PER_EM)
    }

    #[allow(dead_code)]
    pub fn font_mut(&mut self) -> &mut FontObject {
        Arc::make_mut(&mut self.font)
    }
}

impl FontObject {
    #[allow(dead_code)]
    fn update_info(&mut self, info: &SimpleFontInfo) {
        let existing_info = SimpleFontInfo::from_font(self);
        if existing_info != *info {
            let font_info = self.ufo.font_info.get_or_insert_with(FontInfo::default);
            
            if existing_info.family_name != info.family_name {
                font_info.family_name = Some(info.family_name.clone());
            }
            if existing_info.style_name != info.style_name {
                font_info.style_name = Some(info.style_name.clone());
            }
            if existing_info.metrics.units_per_em != info.metrics.units_per_em {
                font_info.units_per_em = Some(norad::NonNegativeIntegerOrFloat::try_from(info.metrics.units_per_em).unwrap());
            }
            if existing_info.metrics.descender != info.metrics.descender {
                font_info.descender = info.metrics.descender.map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.ascender != info.metrics.ascender {
                font_info.ascender = info.metrics.ascender.map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.x_height != info.metrics.x_height {
                font_info.x_height = info.metrics.x_height.map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.cap_height != info.metrics.cap_height {
                font_info.cap_height = info.metrics.cap_height.map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
            if existing_info.metrics.italic_angle != info.metrics.italic_angle {
                font_info.italic_angle = info.metrics.italic_angle.map(|v| norad::IntegerOrFloat::try_from(v).unwrap());
            }
        }
    }
}

impl Default for FontObject {
    fn default() -> FontObject {
        let font_info = FontInfo {
            family_name: Some(String::from("Untitled")),
            ..Default::default()
        };

        let mut ufo = Ufo::new();
        ufo.font_info = Some(font_info);

        FontObject {
            path: None,
            ufo,
        }
    }
}

impl SimpleFontInfo {
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

    pub fn get_display_name(&self) -> String {
        if self.family_name.is_empty() && self.style_name.is_empty() {
            "Untitled Font".to_string()
        } else {
            format!("{} {}", self.family_name, self.style_name)
        }
    }
}

impl Default for SimpleFontInfo {
    fn default() -> Self {
        SimpleFontInfo {
            metrics: Default::default(),
            family_name: String::new(),
            style_name: String::new(),
        }
    }
}

impl PartialEq for SimpleFontInfo {
    fn eq(&self, other: &Self) -> bool {
        self.family_name == other.family_name 
            && self.style_name == other.style_name
            && self.metrics == other.metrics
    }
}

impl From<&FontInfo> for FontMetrics {
    fn from(src: &FontInfo) -> FontMetrics {
        FontMetrics {
            units_per_em: src.units_per_em.map(|v| v.get() as f64).unwrap_or(DEFAULT_UNITS_PER_EM),
            descender: src.descender.map(|v| v.get() as f64),
            x_height: src.x_height.map(|v| v.get() as f64),
            cap_height: src.cap_height.map(|v| v.get() as f64),
            ascender: src.ascender.map(|v| v.get() as f64),
            italic_angle: src.italic_angle.map(|v| v.get() as f64),
        }
    }
}

impl Default for FontMetrics {
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
#[allow(dead_code)]
pub(crate) fn path_for_glyph(glyph: &Glyph) -> Option<BezPath> {
    let mut path = BezPath::new();
    if let Some(outline) = &glyph.outline {
        for contour in &outline.contours {
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

            let mut add_curve = |to_point: &ContourPoint, controls: &mut Vec<ContourPoint>| {
                match controls.as_slice() {
                    &[] => path.line_to((to_point.x as f64, to_point.y as f64)),
                    &[ref a] => path.quad_to(
                        (a.x as f64, a.y as f64),
                        (to_point.x as f64, to_point.y as f64),
                    ),
                    &[ref a, ref b] => path.curve_to(
                        (a.x as f64, a.y as f64),
                        (b.x as f64, b.y as f64),
                        (to_point.x as f64, to_point.y as f64),
                    ),
                    _illegal => panic!("existence of second point implies first"),
                };
                controls.clear();
            };

            let mut idx = (start_idx + 1) % contour.points.len();
            while idx != start_idx {
                let next = &contour.points[idx];
                match next.typ {
                    PointType::OffCurve => controls.push(next.clone()),
                    PointType::Line => {
                        debug_assert!(controls.is_empty(), "line type cannot follow offcurve");
                        add_curve(next, &mut controls);
                    }
                    PointType::Curve => add_curve(next, &mut controls),
                    PointType::QCurve => {
                        warn!("quadratic curves are currently ignored");
                        add_curve(next, &mut controls);
                    }
                    PointType::Move => debug_assert!(false, "illegal move point in path?"),
                }
                idx = (idx + 1) % contour.points.len();
            }

            if let Some(to_close) = close {
                add_curve(to_close, &mut controls);
                path.close_path();
            }
        }
    }
    Some(path)
}

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

#[derive(Component)]
pub struct BezPath {
    pub path: Vec<PathCommand>,
}

#[derive(Clone, Component)]
#[allow(dead_code)]
pub enum PathCommand {
    #[allow(dead_code)]
    MoveTo(Vec2),
    #[allow(dead_code)]
    LineTo(Vec2),
    #[allow(dead_code)]
    QuadTo(Vec2, Vec2),
    #[allow(dead_code)]
    CurveTo(Vec2, Vec2, Vec2),
    ClosePath,
}

impl BezPath {
    pub fn new() -> Self {
        BezPath {
            path: Vec::new(),
        }
    }

    pub fn move_to(&mut self, to: (f64, f64)) {
        self.path.push(PathCommand::MoveTo(Vec2::new(to.0 as f32, to.1 as f32)));
    }

    pub fn line_to(&mut self, to: (f64, f64)) {
        self.path.push(PathCommand::LineTo(Vec2::new(to.0 as f32, to.1 as f32)));
    }

    pub fn quad_to(&mut self, ctrl: (f64, f64), to: (f64, f64)) {
        self.path.push(PathCommand::QuadTo(
            Vec2::new(ctrl.0 as f32, ctrl.1 as f32),
            Vec2::new(to.0 as f32, to.1 as f32),
        ));
    }

    pub fn curve_to(&mut self, ctrl1: (f64, f64), ctrl2: (f64, f64), to: (f64, f64)) {
        self.path.push(PathCommand::CurveTo(
            Vec2::new(ctrl1.0 as f32, ctrl1.1 as f32),
            Vec2::new(ctrl2.0 as f32, ctrl2.1 as f32),
            Vec2::new(to.0 as f32, to.1 as f32),
        ));
    }

    pub fn close_path(&mut self) {
        self.path.push(PathCommand::ClosePath);
    }
}
