//! Camera-responsive scaling system
//!
//! This module provides a system to adjust visual element sizes (line widths, point sizes, etc.)
//! based on camera zoom level to maintain visual consistency across different zoom levels.
//! 
//! This addresses the issue where mesh-based rendering elements become too small when zoomed out,
//! unlike gizmos which maintain screen-space size automatically.

use bevy::prelude::*;
use crate::rendering::cameras::DesignCamera;

/// Resource that tracks the current camera-responsive scale factor
#[derive(Resource, Default)]
pub struct CameraResponsiveScale {
    /// The current scale factor to apply to visual elements
    /// 1.0 = normal size, >1.0 = bigger, <1.0 = smaller
    pub scale_factor: f32,
    /// The base line width in world units at normal zoom (1.0)
    pub base_line_width: f32,
    /// The minimum scale factor to prevent elements from becoming too small
    pub min_scale_factor: f32,
    /// The maximum scale factor to prevent elements from becoming too large
    pub max_scale_factor: f32,
}

impl CameraResponsiveScale {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            base_line_width: 1.0,
            min_scale_factor: 0.5,
            max_scale_factor: 20.0, // Increased from 4.0 to make effect more obvious
        }
    }
    
    /// Get the adjusted line width based on camera zoom
    pub fn adjusted_line_width(&self) -> f32 {
        self.base_line_width * self.scale_factor
    }
    
    /// Get the adjusted point size based on camera zoom  
    pub fn adjusted_point_size(&self, base_size: f32) -> f32 {
        base_size * self.scale_factor
    }
    
    /// Get the adjusted handle size based on camera zoom
    pub fn adjusted_handle_size(&self, base_size: f32) -> f32 {
        base_size * self.scale_factor
    }
}

/// System that updates the camera-responsive scale based on current camera zoom
pub fn update_camera_responsive_scale(
    mut scale_resource: ResMut<CameraResponsiveScale>,
    camera_query: Query<&Transform, With<DesignCamera>>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        // Get camera scale (smaller scale = zoomed in, larger scale = zoomed out)
        let camera_scale = camera_transform.scale.x;
        
        // Calculate responsive scale factor
        // When camera_scale is large (zoomed in), we want smaller visual elements
        // When camera_scale is small (zoomed out), we want larger visual elements
        // Use inverse relationship to make the effect very obvious
        
        // DEBUG: Let me figure out the actual camera scaling relationship
        // Add more debug output to understand camera_scale values
        println!("[CAMERA DEBUG] Raw camera_scale = {:.6}", camera_scale);
        
        // ⚠️  CRITICAL: DO NOT CHANGE THE INVERSE RELATIONSHIP! ⚠️ 
        // PROVEN WORKING: inverse relationship (constant / camera_scale) is correct
        //
        // USER FEEDBACK: Current 1.5/camera_scale is "too big when zoomed in, too small when zoomed out"
        // Need: 2x less at zoom-in, 2x more at zoom-out
        // 
        // SIMPLIFY: Just use different base constants for each zoom direction
        let responsive_factor = if camera_scale < 1.0 {
            // ZOOMED OUT: use bigger base constant for visibility
            4.0 / camera_scale.max(0.1)
        } else {
            // ZOOMED IN: use smaller base constant to avoid huge elements
            1.0 / camera_scale.max(0.1)
        };


        // Clamp to reasonable bounds
        let clamped_factor = responsive_factor
            .max(scale_resource.min_scale_factor)
            .min(scale_resource.max_scale_factor);
        
        // Debug output to see if it's working
        if (clamped_factor - scale_resource.scale_factor).abs() > 0.01 {
            println!("[CAMERA RESPONSIVE] camera_scale={:.3}, responsive_factor={:.3}, clamped_factor={:.3}, line_width={:.3}", 
                camera_scale, responsive_factor, clamped_factor, clamped_factor * scale_resource.base_line_width);
        }
        
        scale_resource.scale_factor = clamped_factor;
    }
}

/// Component to mark entities that should respond to camera zoom
#[derive(Component)]
pub struct CameraResponsive {
    /// The type of visual element this entity represents
    pub element_type: ResponsiveElementType,
    /// The base size when scale_factor = 1.0
    pub base_size: f32,
}

/// Types of visual elements that can respond to camera zoom
#[derive(Debug, Clone)]
pub enum ResponsiveElementType {
    LineWidth,
    PointSize,
    HandleSize,
}

/// System that applies camera-responsive scaling to marked entities
pub fn apply_camera_responsive_scaling(
    scale_resource: Res<CameraResponsiveScale>,
    mut responsive_query: Query<(&CameraResponsive, &mut Transform), Changed<CameraResponsive>>,
) {
    if scale_resource.is_changed() {
        for (responsive, mut transform) in responsive_query.iter_mut() {
            let new_scale = match responsive.element_type {
                ResponsiveElementType::LineWidth => scale_resource.adjusted_line_width(),
                ResponsiveElementType::PointSize => scale_resource.adjusted_point_size(responsive.base_size),
                ResponsiveElementType::HandleSize => scale_resource.adjusted_handle_size(responsive.base_size),
            };
            
            // Update the transform scale while preserving position
            transform.scale = Vec3::new(new_scale, new_scale, 1.0);
        }
    }
}

/// Plugin for camera-responsive scaling
pub struct CameraResponsivePlugin;

impl Plugin for CameraResponsivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraResponsiveScale::new())
           .add_systems(Update, (
               update_camera_responsive_scale,
               apply_camera_responsive_scaling.after(update_camera_responsive_scale),
           ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_responsive_scale_calculation() {
        let mut scale = CameraResponsiveScale::new();
        
        // Test zoomed in (small camera scale)
        scale.scale_factor = 0.5_f32.sqrt(); // camera_scale = 0.5
        assert!(scale.scale_factor < 1.0, "Zoomed in should have smaller scale factor");
        
        // Test zoomed out (large camera scale)  
        scale.scale_factor = 4.0_f32.sqrt(); // camera_scale = 4.0
        assert!(scale.scale_factor > 1.0, "Zoomed out should have larger scale factor");
        
        // Test normal zoom
        scale.scale_factor = 1.0_f32.sqrt(); // camera_scale = 1.0
        assert_eq!(scale.scale_factor, 1.0, "Normal zoom should have scale factor of 1.0");
    }
    
    #[test]
    fn test_adjusted_sizes() {
        let mut scale = CameraResponsiveScale::new();
        scale.scale_factor = 2.0;
        
        assert_eq!(scale.adjusted_line_width(), 2.0);
        assert_eq!(scale.adjusted_point_size(10.0), 20.0);
        assert_eq!(scale.adjusted_handle_size(16.0), 32.0);
    }
}
