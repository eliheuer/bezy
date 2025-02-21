use bevy::prelude::*;
use bevy::math::Vec2 as BevyVec2;
use kurbo::{BezPath, Point, Vec2};
use norad::Contour;

use crate::design_space::DPoint;
use crate::point::{EntityId, PathPoint, PointType};
use crate::point_list::{PathPoints, RawSegment};
use crate::selection::Selection;

#[derive(Component, Debug, Clone)]
pub struct Path {
    points: PathPoints,
    closed: bool,
    trailing: Option<DPoint>,
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub points: Vec<PathPoint>,
}

impl Path {
    pub fn from_norad(contour: &Contour) -> Self {
        let mut points = Vec::new();
        for point in &contour.points {
            let typ = match point.typ {
                norad::PointType::Move => PointType::OnCurve { smooth: false },
                norad::PointType::Line => PointType::OnCurve { smooth: false },
                norad::PointType::Curve => PointType::OnCurve { smooth: true },
                norad::PointType::QCurve => PointType::OnCurve { smooth: true },
                norad::PointType::OffCurve => PointType::OffCurve { auto: false },
            };
            points.push(PathPoint::new(
                DPoint::new(point.x as f64, point.y as f64),
                typ,
            ));
        }

        Path {
            points: PathPoints::new(points),
            closed: true,
            trailing: None,
        }
    }

    pub fn to_norad(&self) -> Contour {
        let mut points = Vec::new();
        for point in self.points.iter() {
            let typ = match point.typ {
                PointType::OnCurve { smooth: false } => norad::PointType::Line,
                PointType::OnCurve { smooth: true } => norad::PointType::Curve,
                PointType::OffCurve { .. } => norad::PointType::OffCurve,
            };
            points.push(norad::ContourPoint {
                x: point.point.x() as f32,
                y: point.point.y() as f32,
                typ,
                smooth: matches!(point.typ, PointType::OnCurve { smooth: true }),
                name: None,
                identifier: None,
            });
        }

        Contour {
            points,
            identifier: None,
        }
    }

    pub fn points(&self) -> &[PathPoint] {
        self.points.points()
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.points.contains(id)
    }

    pub fn start_point(&self) -> DPoint {
        self.points.first().point
    }

    pub fn trailing(&self) -> Option<DPoint> {
        self.trailing
    }

    pub fn should_draw_trailing(&self) -> bool {
        self.trailing.is_some()
    }

    pub fn next_point(&self, id: EntityId) -> Option<&PathPoint> {
        self.points.next_point(id)
    }

    pub fn prev_point(&self, id: EntityId) -> Option<&PathPoint> {
        self.points.prev_point(id)
    }

    pub fn path_point_for_id(&self, id: EntityId) -> Option<PathPoint> {
        self.points.point_for_id(id)
    }

    pub fn delete_points(&mut self, points: &[EntityId]) -> Option<EntityId> {
        self.points.delete_points(points)
    }

    pub fn nudge_points(&mut self, points: &[EntityId], delta: BevyVec2) {
        self.points.nudge_points(points, delta);
    }

    pub fn nudge_all_points(&mut self, delta: BevyVec2) {
        self.points.nudge_all_points(delta);
    }

    pub fn toggle_point_type(&mut self, id: EntityId) {
        self.points.toggle_point_type(id);
    }

    pub fn align_point(&mut self, id: EntityId, val: f64, set_x: bool) {
        self.points.align_point(id, val, set_x);
    }

    pub fn scale_points(&mut self, points: &[EntityId], scale: BevyVec2, anchor: DPoint) {
        self.points.scale_points(points, scale, anchor);
    }

    pub fn reverse_contour(&mut self) {
        self.points.reverse();
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = PathSegment> + '_ {
        self.points
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_on_curve())
            .map(move |(i, _)| {
                let mut points = Vec::new();
                let mut j = i;
                loop {
                    points.push(self.points[j].clone());
                    j = (j + 1) % self.points.len();
                    if j == i || self.points[j].is_on_curve() {
                        break;
                    }
                }
                if j != i {
                    points.push(self.points[j].clone());
                }
                PathSegment { points }
            })
    }

    pub fn segments_for_points(&self, sel: &Selection) -> Vec<PathSegment> {
        let mut result = Vec::new();
        for seg in self.iter_segments() {
            let points: Vec<_> = seg.points.iter().map(|p| p.id).collect();
            if points.iter().all(|id| sel.contains(id)) {
                result.push(seg);
            }
        }
        result
    }

    pub fn bezier(&self) -> BezPath {
        let mut bez = BezPath::new();
        if let Some(first) = self.points.first() {
            bez.move_to(first.point.to_raw());
            for seg in self.iter_segments() {
                match seg.raw_segment() {
                    RawSegment::Line(_, p1) => bez.line_to(p1.to_raw()),
                    RawSegment::Cubic(_, p1, p2, p3) => bez.curve_to(p1.to_raw(), p2.to_raw(), p3.to_raw()),
                }
            }
            if self.closed {
                bez.close_path();
            }
        }
        bez
    }

    pub fn append_to_bezier(&self, bez: &mut BezPath) {
        if let Some(first) = self.points.first() {
            bez.move_to(first.point.to_raw());
            for seg in self.iter_segments() {
                match seg.raw_segment() {
                    RawSegment::Line(_, p1) => bez.line_to(p1.to_raw()),
                    RawSegment::Cubic(_, p1, p2, p3) => bez.curve_to(p1.to_raw(), p2.to_raw(), p3.to_raw()),
                }
            }
            if self.closed {
                bez.close_path();
            }
        }
    }
}

impl PathSegment {
    pub fn raw_segment(&self) -> RawSegment {
        match self.points.as_slice() {
            [p0, p1] => RawSegment::Line(p0.point, p1.point),
            [p0, p1, p2, p3] => RawSegment::Cubic(p0.point, p1.point, p2.point, p3.point),
            _ => panic!("invalid segment"),
        }
    }
} 