use std::collections::BTreeSet;

use bevy::prelude::*;
use kurbo::{BezPath, Point, Rect, Shape, Size, Vec2};
use norad::glyph::Outline;
use norad::{Glyph, GlyphName};

use crate::component::GlyphComponent;
use crate::viewport::ViewPort;
use crate::guide::Guide;
use crate::path::{Path, PathSegment};
use crate::point::{EntityId, PathPoint};
use crate::quadrant::Quadrant;
use crate::selection::Selection;

/// Minimum distance in screen units that a click must occur to be considered
/// on a point
pub const MIN_CLICK_DISTANCE: f64 = 10.0;
pub const SEGMENT_CLICK_DISTANCE: f64 = 6.0;
/// Amount of bias penalizing on-curve points; we want to break ties in favor
/// of off-curve.
pub const ON_CURVE_PENALTY: f64 = MIN_CLICK_DISTANCE / 2.0;

#[derive(Component, Debug, Clone)]
pub struct EditSession {
    pub name: GlyphName,
    pub glyph: Glyph,
    pub paths: Vec<Path>,
    pub selection: Selection,
    pub components: Vec<GlyphComponent>,
    pub guides: Vec<Guide>,
    pub viewport: ViewPort,
    pub work_bounds: Rect,
    pub quadrant: Quadrant,
}

#[derive(Component, Debug, Clone)]
pub struct CoordinateSelection {
    /// the number of selected points
    pub count: usize,
    /// the bounding box of the selection
    pub frame: Rect,
    pub quadrant: Quadrant,
}

impl EditSession {
    pub fn new(name: &GlyphName, glyph: &Glyph) -> Self {
        let name = name.to_owned();
        let paths: Vec<Path> = glyph
            .outline
            .as_ref()
            .map(|ol| ol.contours.iter().map(Path::from_norad).collect())
            .unwrap_or_default();
        
        let components = glyph
            .outline
            .as_ref()
            .map(|ol| ol.components.iter().map(GlyphComponent::from_norad).collect())
            .unwrap_or_default();
            
        let guides = glyph
            .guidelines
            .as_ref()
            .map(|guides| guides.iter().map(Guide::from_norad).collect())
            .unwrap_or_default();

        // Calculate work bounds from bezier paths
        let work_bounds = paths
            .iter()
            .filter_map(|p| p.to_bezier().bounding_box())
            .fold(Rect::ZERO, |acc, bbox| acc.union(bbox));

        EditSession {
            name,
            glyph: glyph.clone(),
            paths,
            selection: Selection::new(),
            components,
            guides,
            viewport: ViewPort::default(),
            quadrant: Quadrant::Center,
            work_bounds,
        }
    }

    /// Construct a bezier of the paths in this glyph, ignoring components.
    pub fn to_bezier(&self) -> BezPath {
        let mut bez = BezPath::new();
        for path in self.paths.iter() {
            path.append_to_bezier(&mut bez);
        }
        bez
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PathPoint> {
        self.paths.iter().flat_map(|p| p.points().iter())
    }

    /// Find the best hit, considering all items.
    pub fn hit_test_all(&self, point: Point, max_dist: Option<f64>) -> Option<EntityId> {
        if let Some(hit) = self.hit_test_filtered(point, max_dist, |_| true) {
            return Some(hit);
        }
        let max_dist = max_dist.unwrap_or(MIN_CLICK_DISTANCE);
        let mut best = None;
        for g in &self.guides {
            let dist = g.screen_dist(self.viewport, point);
            if dist < max_dist && best.map(|(d, _id)| dist < d).unwrap_or(true) {
                best = Some((dist, g.id))
            }
        }
        best.map(|(_dist, id)| id)
    }

    /// Hit test a point against points.
    pub fn hit_test_filtered(
        &self,
        point: Point,
        max_dist: Option<f64>,
        mut f: impl FnMut(&PathPoint) -> bool,
    ) -> Option<EntityId> {
        let max_dist = max_dist.unwrap_or(MIN_CLICK_DISTANCE);
        let mut best = None;
        for p in self.iter_points() {
            if f(p) {
                let dist = p.screen_dist(self.viewport, point);
                let score = dist
                    + if p.is_on_curve() {
                        ON_CURVE_PENALTY
                    } else {
                        0.0
                    };
                if dist < max_dist && best.map(|(s, _id)| score < s).unwrap_or(true) {
                    best = Some((score, p.id))
                }
            }
        }
        best.map(|(_score, id)| id)
    }

    pub fn delete_selection(&mut self) {
        let to_delete = self.selection.per_path_selection();
        self.selection.clear();
        let set_sel = to_delete.path_len() == 1;

        for path_points in to_delete.iter() {
            if let Some(path) = self.path_for_point_mut(path_points[0]) {
                if let Some(new_sel) = path.delete_points(path_points) {
                    if set_sel {
                        self.selection.select_one(new_sel);
                    }
                }
            } else if path_points[0].is_guide() {
                self.guides.retain(|g| !path_points.contains(&g.id));
            }
        }
        self.paths.retain(|p| !p.points().is_empty());
    }

    pub fn select_all(&mut self) {
        self.selection.clear();
        self.selection = self.iter_points().map(|p| p.id).collect();
    }

    pub fn path_for_point(&self, point: EntityId) -> Option<&Path> {
        self.path_idx_for_point(point)
            .and_then(|idx| self.paths.get(idx))
    }

    pub fn path_for_point_mut(&mut self, point: EntityId) -> Option<&mut Path> {
        let idx = self.path_idx_for_point(point)?;
        self.paths.get_mut(idx)
    }

    fn path_idx_for_point(&self, point: EntityId) -> Option<usize> {
        self.paths.iter().position(|p| p.contains(&point))
    }

    pub fn nudge_selection(&mut self, nudge: Vec2) {
        if self.selection.is_empty() {
            return;
        }

        let to_nudge = self.selection.per_path_selection();
        for path_points in to_nudge.iter() {
            if let Some(path) = self.path_for_point_mut(path_points[0]) {
                path.nudge_points(path_points, nudge);
            } else if path_points[0].is_guide() {
                for id in path_points {
                    if let Some(guide) = self.guides.iter_mut().find(|g| g.id == *id) {
                        guide.nudge(nudge);
                    }
                }
            }
        }
    }

    pub fn to_norad_glyph(&self) -> Glyph {
        let mut glyph = Glyph::new_named("");
        glyph.name = self.name.clone();
        glyph.advance = self.glyph.advance.clone();
        glyph.codepoints = self.glyph.codepoints.clone();

        let contours: Vec<_> = self.paths.iter().map(Path::to_norad).collect();
        let components: Vec<_> = self.components.iter().map(GlyphComponent::to_norad).collect();
        if !contours.is_empty() || !components.is_empty() {
            glyph.outline = Some(Outline {
                components,
                contours,
            });
        }
        let guidelines: Vec<_> = self.guides.iter().map(Guide::to_norad).collect();
        if !guidelines.is_empty() {
            glyph.guidelines = Some(guidelines);
        }
        glyph
    }
}

// Systems for Bevy integration
pub fn edit_session_system(mut query: Query<&mut EditSession>) {
    for mut session in query.iter_mut() {
        // Add system logic here
    }
}

// Plugin to register the EditSession component and systems
pub struct EditSessionPlugin;

impl Plugin for EditSessionPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<EditSession>()
            .register_type::<CoordinateSelection>()
            .add_systems(Update, edit_session_system);
    }
}
