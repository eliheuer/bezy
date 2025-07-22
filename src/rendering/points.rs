//! Point rendering system
//!
//! This module handles the mesh-based rendering of points (both on-curve and off-curve)
//! replacing the previous gizmo-based point rendering.

use crate::editing::selection::components::{PointType, Selected};
use crate::editing::sort::ActiveSort;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::render::mesh::Mesh2d;
use bevy::render::view::Visibility;

/// Component to mark entities as point visual meshes
#[derive(Component)]
pub struct PointMesh {
    pub point_entity: Entity,
    pub is_outer: bool, // true for outer shape, false for inner dot
}

/// System to render points using meshes instead of gizmos
#[allow(clippy::type_complexity)]
pub fn render_points_with_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_entities: Query<
        (Entity, &GlobalTransform, &PointType),
        (With<SortPointEntity>, Without<Selected>),
    >,
    all_point_entities: Query<
        (Entity, &GlobalTransform, &PointType, Option<&Selected>),
        With<SortPointEntity>,
    >,
    active_sorts: Query<Entity, With<ActiveSort>>,
    existing_point_meshes: Query<Entity, With<PointMesh>>,
) {
    let all_point_count = all_point_entities.iter().count();
    let existing_mesh_count = existing_point_meshes.iter().count();
    let active_sort_count = active_sorts.iter().count();
    
    // Early return if no active sorts
    if active_sort_count == 0 {
        // Clean up existing point meshes when no active sorts
        for entity in existing_point_meshes.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Clear existing point meshes
    for entity in existing_point_meshes.iter() {
        commands.entity(entity).despawn();
    }

    // Render all points using meshes
    for (point_entity, transform, point_type, selected) in all_point_entities.iter() {
        let position = transform.translation().truncate();
        
        // Determine color based on selection state
        let point_color = if selected.is_some() {
            SELECTED_POINT_COLOR
        } else if point_type.is_on_curve {
            ON_CURVE_POINT_COLOR
        } else {
            OFF_CURVE_POINT_COLOR
        };
        
        // Create the main point shape
        if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            // On-curve points: square
            let size = ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT * 2.0;
            commands.spawn((
                PointMesh { point_entity, is_outer: true },
                Sprite {
                    color: point_color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            
            // Inner crosshair/dot
            let inner_size = size * ON_CURVE_INNER_CIRCLE_RATIO;
            commands.spawn((
                PointMesh { point_entity, is_outer: false },
                Sprite {
                    color: point_color,
                    custom_size: Some(Vec2::splat(inner_size)),
                    ..default()
                },
                Transform::from_translation(position.extend(11.0)), // Above outer shape
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        } else {
            // Off-curve points and circular on-curve points: circle
            let radius = if point_type.is_on_curve { 
                ON_CURVE_POINT_RADIUS 
            } else { 
                OFF_CURVE_POINT_RADIUS 
            };
            
            commands.spawn((
                PointMesh { point_entity, is_outer: true },
                Mesh2d(meshes.add(Circle::new(radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(point_color))),
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            
            // Inner circle
            let inner_radius = radius * if point_type.is_on_curve { 
                ON_CURVE_INNER_CIRCLE_RATIO 
            } else { 
                OFF_CURVE_INNER_CIRCLE_RATIO 
            };
            commands.spawn((
                PointMesh { point_entity, is_outer: false },
                Mesh2d(meshes.add(Circle::new(inner_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(point_color))),
                Transform::from_translation(position.extend(11.0)), // Above outer shape
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
        
        // Add crosshairs for selected points
        if selected.is_some() {
            let line_size = if point_type.is_on_curve { 
                ON_CURVE_POINT_RADIUS 
            } else { 
                OFF_CURVE_POINT_RADIUS 
            };
            
            // Horizontal line
            commands.spawn((
                PointMesh { point_entity, is_outer: false },
                Sprite {
                    color: point_color,
                    custom_size: Some(Vec2::new(line_size * 2.0, 1.0)),
                    ..default()
                },
                Transform::from_translation(position.extend(12.0)), // Above everything
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            
            // Vertical line
            commands.spawn((
                PointMesh { point_entity, is_outer: false },
                Sprite {
                    color: point_color,
                    custom_size: Some(Vec2::new(1.0, line_size * 2.0)),
                    ..default()
                },
                Transform::from_translation(position.extend(12.0)), // Above everything
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }
}


/// Plugin for mesh-based point rendering
pub struct PointRenderingPlugin;



impl Plugin for PointRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            render_points_with_meshes
                .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized)
                .after(crate::editing::selection::nudge::handle_nudge_input),
        );
    }
}