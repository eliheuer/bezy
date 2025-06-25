//! Handling glyph components
//!
//! Glyph components are references to other glyphs that can be transformed
//! (moved, scaled, rotated) and placed within a glyph. This is useful for
//! creating composite characters like accented letters.
//!
//! # Example
//! If you have a letter "e" and want to create "é", you can use the "e"
//! glyph as a base and add an acute accent component positioned above it.

use bevy::prelude::*;
#[allow(unused_imports)]
use norad::Component as NoradComponent;

/// A simple 2D transformation that's easy to understand
///
/// This groups all the transformation values together so we don't need
/// to pass around 6 separate parameters.
///
/// Scale values: 1.0 = normal size, <1.0 = smaller, >1.0 = larger
/// Skew values: 0.0 = no slanting, positive = slant right/up
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Transform2D {
    scale_x: f32,     // Horizontal stretch/shrink
    scale_y: f32,     // Vertical stretch/shrink
    skew_x: f32,      // Horizontal slant
    skew_y: f32,      // Vertical slant
    translate_x: f32, // Horizontal movement
    translate_y: f32, // Vertical movement
}

#[allow(dead_code)]
impl Transform2D {
    /// Creates a new transformation from UFO affine transform data
    fn from_affine(affine: &norad::AffineTransform) -> Self {
        Self {
            scale_x: affine.x_scale as f32,
            scale_y: affine.y_scale as f32,
            skew_x: affine.xy_scale as f32,
            skew_y: affine.yx_scale as f32,
            translate_x: affine.x_offset as f32,
            translate_y: affine.y_offset as f32,
        }
    }

    /// Converts this transformation to UFO affine transform data
    fn to_affine(self) -> norad::AffineTransform {
        norad::AffineTransform {
            x_scale: self.scale_x as f64,
            y_scale: self.scale_y as f64,
            xy_scale: self.skew_x as f64,
            yx_scale: self.skew_y as f64,
            x_offset: self.translate_x as f64,
            y_offset: self.translate_y as f64,
        }
    }

    /// Converts this to a Bevy Transform for rendering
    fn to_bevy_transform(self) -> Transform {
        // Build a 2D transformation matrix
        // Each column represents how that axis gets transformed
        let matrix = Mat4::from_cols(
            // X-axis: scale and skew
            Vec4::new(self.scale_x, self.skew_y, 0.0, 0.0),
            // Y-axis: skew and scale
            Vec4::new(self.skew_x, self.scale_y, 0.0, 0.0),
            // Z-axis: not used in 2D
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            // Translation
            Vec4::new(self.translate_x, self.translate_y, 0.0, 1.0),
        );

        Transform::from_matrix(matrix)
    }

    /// Extracts transformation values from a Bevy Transform
    fn from_bevy_transform(transform: &Transform) -> Self {
        let matrix = transform.compute_matrix();

        Self {
            scale_x: matrix.x_axis.x,
            scale_y: matrix.y_axis.y,
            skew_x: matrix.y_axis.x,
            skew_y: matrix.x_axis.y,
            translate_x: matrix.w_axis.x,
            translate_y: matrix.w_axis.y,
        }
    }
} 