//! 'Design space' is the fixed coordinate space in which we describe glyphs,
//! guides, and other entities.
//!
//! When drawing to the screen or handling mouse input, we need to translate from
//! 'screen space' or `world space` to design space, taking into account things like the current
//! pan offset and zoom level.

use bevy::prelude::*;

/// Represents a point in design space coordinates
pub struct DesignPoint {
    pub x: f32,
    pub y: f32,
}

impl DesignPoint {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32) -> Self {
        DesignPoint { x, y }
    }

    /// Convert a point from screen space to design space
    #[allow(dead_code)]
    pub fn from_screen_space(
        screen_point: Vec2,
        camera_transform: &GlobalTransform,
    ) -> Self {
        // Convert screen point to world space using the camera's transform
        let world_point = camera_transform
            .compute_matrix()
            .inverse()
            .transform_point3(screen_point.extend(0.0));

        DesignPoint {
            x: world_point.x,
            y: world_point.y,
        }
    }

    /// Convert a design space point to screen space
    #[allow(dead_code)]
    pub fn to_screen_space(&self, camera_transform: &GlobalTransform) -> Vec2 {
        // Convert design point to screen space using the camera's transform
        let screen_point = camera_transform
            .compute_matrix()
            .transform_point3(Vec3::new(self.x, self.y, 0.0));

        Vec2::new(screen_point.x, screen_point.y)
    }
}
