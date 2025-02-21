use bevy::prelude::*;
use bevy::math::Vec2 as BevyVec2;
use kurbo::{Affine, BezPath};
use norad::Component as NoradComponent;
use std::sync::Arc;

#[derive(Component, Debug, Clone)]
pub struct GlyphComponent {
    pub base: String,
    pub transform: Transform,
}

impl GlyphComponent {
    pub fn from_norad(comp: &NoradComponent) -> Self {
        let transform = Transform::from_matrix(Mat4::from_cols_array(&[
            comp.transform.x_scale as f32, comp.transform.yx_scale as f32, 0.0, 0.0,
            comp.transform.xy_scale as f32, comp.transform.y_scale as f32, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            comp.transform.x_offset as f32, comp.transform.y_offset as f32, 0.0, 1.0,
        ]));

        GlyphComponent {
            base: comp.base.clone(),
            transform,
        }
    }

    pub fn to_norad(&self) -> NoradComponent {
        let matrix = self.transform.compute_matrix();
        let [m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33] = matrix.to_cols_array();

        NoradComponent {
            base: self.base.clone(),
            transform: norad::AffineTransform {
                x_scale: m00,
                yx_scale: m01,
                xy_scale: m10,
                y_scale: m11,
                x_offset: m30,
                y_offset: m31,
            },
            identifier: None,
        }
    }

    pub fn nudge(&mut self, delta: BevyVec2) {
        self.transform.translation += Vec3::new(delta.x, delta.y, 0.0);
    }
} 