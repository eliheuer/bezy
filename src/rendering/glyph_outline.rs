//! This module is responsible for rendering glyph outlines. It is designed to be used
//! by both the main glyph rendering system and individual sorts.

use bevy::prelude::*;
use kurbo::{BezPath, Point};
use norad::{ContourPoint, Font, Glyph, PointType};

// This struct will be a component on the same entity as the font.
// It will be responsible for drawing the glyph outlines.
#[derive(Component, Debug, Default)]
pub struct GlyphOutline {
    pub glyph_name: String,
    pub glyph_data: Option<Glyph>,
    pub path: BezPath,
}

impl GlyphOutline {
    pub fn new(glyph_name: &str, font: &Font) -> Self {
        let glyph_data = font.get_glyph(glyph_name).cloned();
        let mut path = BezPath::new();

        if let Some(glyph) = &glyph_data {
            for contour in &glyph.contours {
                if contour.points.is_empty() {
                    continue;
                }
                path.move_to(Point::new(
                    contour.points[0].x as f64,
                    contour.points[0].y as f64,
                ));
                for point in contour.points.windows(2) {
                    let p0 = Point::new(point[0].x as f64, point[0].y as f64);
                    let p1 = Point::new(point[1].x as f64, point[1].y as f64);
                    match point[1].typ {
                        PointType::Line => path.line_to(p1),
                        PointType::Curve => path.curve_to(p0, p1, p1),
                        _ => {}
                    }
                }
            }
        }

        Self {
            glyph_name: glyph_name.to_string(),
            glyph_data,
            path,
        }
    }
} 