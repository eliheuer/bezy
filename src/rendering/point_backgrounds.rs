//! Point background rendering system
//!
//! This module handles the rendering of transparent background sprites behind points
//! for improved visual appearance in screenshots and videos.

use crate::editing::selection::components::{PointType, Selected};
use crate::editing::sort::ActiveSort;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::render::mesh::Mesh2d;
use bevy::render::view::Visibility;

/// Component to mark entities as point background sprites
#[derive(Component)]
pub struct PointBackground {
    pub point_entity: Entity,
}

/// Component to mark entities as point center dots
#[derive(Component)]
pub struct PointCenterDot {
    pub point_entity: Entity,
}

/// System to spawn/update transparent background sprites for points
#[allow(clippy::type_complexity)]
pub fn manage_point_backgrounds(
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
    background_query: Query<(Entity, &PointBackground)>,
    existing_backgrounds: Query<Entity, With<PointBackground>>,
    existing_center_dots: Query<Entity, With<PointCenterDot>>,
) {
    let point_count = point_entities.iter().count();
    let all_point_count = all_point_entities.iter().count();
    let existing_bg_count = existing_backgrounds.iter().count();
    let active_sort_count = active_sorts.iter().count();
    
    info!("[PointBackgrounds] Non-selected points: {}, All points: {}, Existing backgrounds: {}, Active sorts: {}", 
          point_count, all_point_count, existing_bg_count, active_sort_count);
    
    // Early return if no active sorts
    if active_sort_count == 0 {
        info!("[PointBackgrounds] No active sorts - skipping background creation");
        return;
    }

    // Clear existing backgrounds and center dots
    for entity in existing_backgrounds.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_center_dots.iter() {
        commands.entity(entity).despawn();
    }

    // Create new backgrounds for ALL points (not just non-selected)
    for (point_entity, transform, point_type, selected) in all_point_entities.iter() {
        let position = transform.translation().truncate();
        
        // Determine base color based on selection state
        let base_color = if selected.is_some() {
            SELECTED_POINT_COLOR
        } else if point_type.is_on_curve {
            ON_CURVE_POINT_COLOR
        } else {
            OFF_CURVE_POINT_COLOR
        };
        
        let (bg_size, bg_color) = if point_type.is_on_curve {
            let size = if USE_SQUARE_FOR_ON_CURVE {
                let adjusted_radius = ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT;
                Vec2::splat(adjusted_radius * 2.0) // Match exact point size
            } else {
                Vec2::splat(ON_CURVE_POINT_RADIUS * 2.0) // Match exact diameter
            };
            let color = Color::srgba(
                base_color.to_srgba().red,
                base_color.to_srgba().green, 
                base_color.to_srgba().blue,
                0.25 // 25% transparency
            );
            (size, color)
        } else {
            let size = Vec2::splat(OFF_CURVE_POINT_RADIUS * 2.0); // Match exact diameter
            let color = Color::srgba(
                base_color.to_srgba().red,
                base_color.to_srgba().green,
                base_color.to_srgba().blue, 
                0.25 // 25% transparency
            );
            (size, color)
        };

        // Spawn background shape behind the point
        let bg_entity = if point_type.is_on_curve {
            // On-curve points: use square sprite (as before)
            commands.spawn((
                PointBackground { point_entity },
                Sprite {
                    color: bg_color,
                    custom_size: Some(bg_size),
                    ..default()
                },
                Transform::from_translation(position.extend(5.0)), // Behind points (10.0) but in front of checkerboard (0.1)
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            )).id()
        } else {
            // Off-curve points: use circular mesh
            commands.spawn((
                PointBackground { point_entity },
                Mesh2d(meshes.add(Circle::new(bg_size.x / 2.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(bg_color))),
                Transform::from_translation(position.extend(5.0)), // Behind points (10.0) but in front of checkerboard (0.1)
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            )).id()
        };
        
        info!("[PointBackgrounds] Spawned background {:?} for point {:?} at ({:.1}, {:.1}) with size {:?} and color {:?}", 
              bg_entity, point_entity, position.x, position.y, bg_size, bg_color);
              
        // Spawn center dot
        let center_multiplier = if selected.is_some() { 2.0 } else { 1.0 };
        let center_color = if selected.is_some() {
            SELECTED_POINT_COLOR
        } else if point_type.is_on_curve {
            ON_CURVE_POINT_COLOR
        } else {
            OFF_CURVE_POINT_COLOR
        };
        
        if point_type.is_on_curve {
            // On-curve points: use square for center dot
            let inner_size = if USE_SQUARE_FOR_ON_CURVE {
                let adjusted_radius = ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT;
                Vec2::splat(adjusted_radius * 2.0 * ON_CURVE_INNER_CIRCLE_RATIO * center_multiplier)
            } else {
                Vec2::splat(ON_CURVE_POINT_RADIUS * 2.0 * ON_CURVE_INNER_CIRCLE_RATIO * center_multiplier)
            };
            
            commands.spawn((
                PointCenterDot { point_entity },
                Sprite {
                    color: center_color,
                    custom_size: Some(inner_size),
                    ..default()
                },
                Transform::from_translation(position.extend(1000.0)), // Very high z-value to try to render on top
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        } else {
            // Off-curve points: use circular mesh for center dot
            let inner_radius = OFF_CURVE_POINT_RADIUS * OFF_CURVE_INNER_CIRCLE_RATIO * center_multiplier;
            commands.spawn((
                PointCenterDot { point_entity },
                Mesh2d(meshes.add(Circle::new(inner_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(center_color))),
                Transform::from_translation(position.extend(1000.0)), // Very high z-value to try to render on top
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }
}

/// Plugin for point background rendering
pub struct PointBackgroundPlugin;

const POINT_BACKGROUND_Z_LEVEL: f32 = 0.05; // Just behind checkerboard (0.1)


impl Plugin for PointBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manage_point_backgrounds.after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized),
        );
    }
}