// Module for handling glyph components in the Bezy font editor
// This file provides conversion utilities between Bevy's transform system and Norad's font data structures
//
// === GLYPH COMPONENTS IN FONT DESIGN ===
// In typography and font design, a "component" refers to a reusable part of a glyph that can be
// referenced by other glyphs. This is a fundamental concept in modern font design that enables:
//
// 1. CONSISTENCY: By reusing the same component (like an accent mark or a radical in CJK characters),
//    designers ensure consistent appearance across the font.
//
// 2. EFFICIENCY: Instead of redrawing the same elements repeatedly, designers can define them once
//    and reuse them with transformations (scaling, rotation, translation).
//
// 3. MAINTAINABILITY: When a component is updated, all glyphs using that component are automatically
//    updated, saving significant time during font development and refinement.
//
// === BEZY'S IMPLEMENTATION ===
// In the Bezy font editor, glyph components are managed through this module, which provides:
//
// - An abstraction over the raw UFO component format (from the Norad library)
// - Integration with Bevy's ECS (Entity Component System)
// - Transformation utilities that allow components to be positioned, scaled, and oriented
//
// The editor allows designers to:
// - Insert components from existing glyphs
// - Position components with precise transformations
// - Edit component references across multiple glyphs simultaneously
// - Visualize and manipulate components in the editor viewport
//
// Components are a key part of the UFO (Unified Font Object) format that Bezy works with,
// making this module essential for correctly importing, displaying, editing, and exporting
// composite glyphs.
use bevy::prelude::*;
use bevy::math::Vec2 as BevyVec2;
// use kurbo::{Affine, BezPath};
use norad::Component as NoradComponent;
use std::sync::Arc;

/// A Bevy component representing a composite glyph component
/// 
/// Composite glyphs (also called component glyphs) are glyphs that are composed of
/// other glyphs with transformations applied. This is commonly used in font design
/// for creating compound characters or reusing components (like accents).
/// 
/// Fields:
/// - `base`: The name of the base glyph that this component references
/// - `transform`: The transformation matrix applied to the base glyph
#[derive(Component, Debug, Clone)]
pub struct GlyphComponent {
    pub base: String,
    pub transform: Transform,
}

impl GlyphComponent {
    /// Converts a Norad component (from the norad UFO library) to a Bevy-compatible GlyphComponent
    /// 
    /// This function maps the Norad AffineTransform (used in UFO format) to Bevy's Transform system
    /// by building a 4x4 transformation matrix.
    pub fn from_norad(comp: &NoradComponent) -> Self {
        // Create a Bevy Transform from Norad's AffineTransform
        // Norad uses a 2x3 matrix while Bevy uses a 4x4 homogeneous transformation matrix
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

    /// Converts a Bevy GlyphComponent back to a Norad Component
    /// 
    /// This is the inverse operation of from_norad, extracting the relevant
    /// transformation values from Bevy's 4x4 matrix to construct Norad's AffineTransform.
    pub fn to_norad(&self) -> NoradComponent {
        // Extract the full 4x4 matrix from Bevy's Transform
        let matrix = self.transform.compute_matrix();
        // Unpack the matrix into individual elements
        let [m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33] = matrix.to_cols_array();

        // Create a new Norad Component with the extracted transformation values
        // Only using the relevant parts of the 4x4 matrix for the 2D transformation
        NoradComponent {
            base: self.base.clone(),
            transform: norad::AffineTransform {
                x_scale: m00,  // Horizontal scaling
                yx_scale: m01, // Y-shearing
                xy_scale: m10, // X-shearing
                y_scale: m11,  // Vertical scaling
                x_offset: m30, // X-translation
                y_offset: m31, // Y-translation
            },
            identifier: None,  // Not setting an identifier when converting back
        }
    }

    /// Moves the component by the specified delta in 2D space
    /// 
    /// This is a convenience method for modifying the translation part of
    /// the transformation without affecting rotation or scale.
    pub fn nudge(&mut self, delta: BevyVec2) {
        // Add the delta to the translation component of the transform
        // The z-component remains unchanged (0.0)
        self.transform.translation += Vec3::new(delta.x, delta.y, 0.0);
    }
} 