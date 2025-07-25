//! Mesh-based sort handle rendering
//!
//! This module provides mesh-based handle rendering for sorts.
//! All gizmo-based rendering has been removed in favor of camera-responsive mesh rendering.

use crate::core::state::FontIRAppState;
use crate::editing::selection::components::Selected;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortRenderStyle {
    TextBuffer,
    Freeform,
}

/// Component to mark entities as sort handle visual elements
#[derive(Component)]
pub struct SortHandle {
    pub sort_entity: Entity,
    pub handle_type: SortHandleType,
}

/// Types of sort handle elements
#[derive(Debug, Clone)]
pub enum SortHandleType {
    Square,
    Circle,
    SelectionIndicator,
}

/// Resource to track sort handle entities
#[derive(Resource, Default)]
pub struct SortHandleEntities {
    pub handles: HashMap<Entity, Vec<Entity>>, // sort_entity -> handle entities
}

/// Helper to spawn a square handle mesh
fn spawn_square_handle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    size: f32,
    color: Color,
    sort_entity: Entity,
) -> Entity {
    let square_mesh = Rectangle::new(size, size);
    
    commands.spawn((
        SortHandle {
            sort_entity,
            handle_type: SortHandleType::Square,
        },
        Mesh2d(meshes.add(square_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_xyz(position.x, position.y, 15.0), // Above metrics
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id()
}

/// Helper to spawn a circle handle mesh
fn spawn_circle_handle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: Color,
    sort_entity: Entity,
) -> Entity {
    let circle_mesh = Circle::new(radius);
    
    commands.spawn((
        SortHandle {
            sort_entity,
            handle_type: SortHandleType::Circle,
        },
        Mesh2d(meshes.add(circle_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_xyz(position.x, position.y, 15.0), // Above metrics
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id()
}

/// System to render mesh-based sort handles for all active sorts
pub fn render_mesh_sort_handles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handle_entities: ResMut<SortHandleEntities>,
    sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        With<crate::editing::sort::ActiveSort>,
    >,
    existing_handles: Query<Entity, With<SortHandle>>,
    selected_query: Query<Entity, With<Selected>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clear existing handles
    for entity in existing_handles.iter() {
        commands.entity(entity).despawn();
    }
    handle_entities.handles.clear();

    if let Some(fontir_state) = fontir_app_state {
        let fontir_metrics = fontir_state.get_font_metrics();
        let descender = fontir_metrics.descender.unwrap_or(-200.0);

        for (sort_entity, sort_transform, sort) in sort_query.iter() {
            let position = sort_transform.translation.truncate();
            let handle_position = position + Vec2::new(0.0, descender);
            
            // Check if this sort is selected
            let is_selected = selected_query.iter().any(|e| e == sort_entity);
            let handle_color = if is_selected {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
            } else {
                Color::srgb(0.0, 1.0, 0.0) // Green for active
            };
            
            // Camera-responsive handle size
            let base_size = 16.0;
            let adjusted_size = camera_scale.adjusted_point_size(base_size);
            
            let mut handles = Vec::new();
            
            // Choose handle style based on layout mode
            match sort.layout_mode {
                crate::core::state::SortLayoutMode::LTRText | 
                crate::core::state::SortLayoutMode::RTLText => {
                    // Square handle for text sorts
                    let handle = spawn_square_handle(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        handle_position,
                        adjusted_size * 2.0,
                        handle_color,
                        sort_entity,
                    );
                    handles.push(handle);
                }
                crate::core::state::SortLayoutMode::Freeform => {
                    // Circle handle for freeform sorts
                    let handle = spawn_circle_handle(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        handle_position,
                        adjusted_size,
                        handle_color,
                        sort_entity,
                    );
                    handles.push(handle);
                }
            }
            
            // Add selection indicator if selected
            if is_selected {
                let indicator_size = adjusted_size * 1.5;
                let indicator = spawn_circle_handle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    handle_position,
                    indicator_size,
                    Color::srgb(1.0, 1.0, 0.0).with_alpha(0.3),
                    sort_entity,
                );
                commands.entity(indicator).insert(ChildOf(handles[0]));
                handles.push(indicator);
            }
            
            handle_entities.handles.insert(sort_entity, handles);
        }
    }
}

pub struct SortHandleRenderingPlugin;

impl Plugin for SortHandleRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SortHandleEntities>()
            .add_systems(Update, render_mesh_sort_handles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_render_style_distinction() {
        // Test that TextBuffer and Freeform styles are distinct
        assert_ne!(SortRenderStyle::TextBuffer, SortRenderStyle::Freeform);
        
        // Test that each style is equal to itself
        assert_eq!(SortRenderStyle::TextBuffer, SortRenderStyle::TextBuffer);
        assert_eq!(SortRenderStyle::Freeform, SortRenderStyle::Freeform);
    }

    #[test]
    fn test_sort_render_style_debug() {
        // Test that styles can be debug printed
        let text_style = SortRenderStyle::TextBuffer;
        let freeform_style = SortRenderStyle::Freeform;
        
        let debug_text = format!("{:?}", text_style);
        let debug_freeform = format!("{:?}", freeform_style);
        
        assert!(debug_text.contains("TextBuffer"));
        assert!(debug_freeform.contains("Freeform"));
    }
}