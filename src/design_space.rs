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
        camera_transform: &Transform,
    ) -> Self {
        // Unproject the screen point using the camera's transform
        let design_point = screen_point / camera_transform.scale.x;
        DesignPoint {
            x: design_point.x,
            y: design_point.y,
        }
    }

    /// Convert a design space point to screen space
    #[allow(dead_code)]
    pub fn to_screen_space(&self, camera_transform: &Transform) -> Vec2 {
        // Project the design point using the camera's transform
        Vec2::new(self.x, self.y) * camera_transform.scale.x
    }
}
