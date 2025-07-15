//! Handle rendering utilities for control points
//!
//! This module contains utilities for rendering bezier control handles
//! and other handle-related visualization elements.

use bevy::prelude::*;

/// Utilities for rendering control handles between bezier points
pub struct HandleRenderer;

impl HandleRenderer {
    /// Draw a control handle line between two points
    pub fn draw_handle_line(
        gizmos: &mut Gizmos,
        start: Vec2,
        end: Vec2,
        color: Color,
    ) {
        gizmos.line_2d(start, end, color);
    }

    /// Draw a control handle with visual endpoint indicators
    pub fn draw_handle_with_endpoints(
        gizmos: &mut Gizmos,
        start: Vec2,
        end: Vec2,
        line_color: Color,
        endpoint_color: Color,
        endpoint_radius: f32,
    ) {
        // Draw the connecting line
        gizmos.line_2d(start, end, line_color);

        // Draw endpoint circles
        gizmos.circle_2d(start, endpoint_radius, endpoint_color);
        gizmos.circle_2d(end, endpoint_radius, endpoint_color);
    }

    /// Calculate the optimal handle length for smooth curves
    pub fn calculate_handle_length(distance: f32, smoothness: f32) -> f32 {
        distance * smoothness.clamp(0.1, 0.9)
    }
}
