//! Point entity management for text editor sorts

use bevy::prelude::*;
use crate::core::state::AppState;
use crate::core::state::font_data::PointTypeData;
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use crate::editing::selection::components::{GlyphPointReference, PointType, Selectable};
use crate::systems::sort_manager::SortPointEntity;
use crate::geometry::point::EditPoint;
use kurbo::Point;

/// Component to track which sort entity a point belongs to
#[derive(Component)]
pub struct PointSortParent(pub Entity);

/// Spawn active sort points optimized
pub fn spawn_active_sort_points_optimized(
    mut commands: Commands,
    active_sort_query: Query<(Entity, &Sort, &Transform), With<ActiveSort>>,
    existing_points: Query<&PointSortParent>,
    app_state: Res<AppState>,
) {
    let active_sort_count = active_sort_query.iter().count();
    if active_sort_count > 0 {
        info!("Point spawning system called: {} active sorts found", active_sort_count);
    }
    
    for (sort_entity, sort, transform) in active_sort_query.iter() {
        // Check if points already exist for this sort
        let has_points = existing_points.iter().any(|parent| parent.0 == sort_entity);
        if has_points {
            debug!("Skipping point spawning for sort entity {:?} - points already exist", sort_entity);
            // TEMP: Force spawn even if points exist to debug the issue
            // continue; // Skip if points already exist
        }
        
        info!("Spawning points for active sort entity {:?}, glyph: {}", sort_entity, sort.glyph_name);
        let position = transform.translation.truncate();
        
        info!("Spawning points for ACTIVE sort '{}' at position ({:.1}, {:.1})", 
              sort.glyph_name, position.x, position.y);
        
        // Get glyph data
        if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
            if let Some(outline) = &glyph_data.outline {
                let mut point_count = 0;
                
                // Spawn points for each contour
                for (contour_index, contour) in outline.contours.iter().enumerate() {
                    for (point_index, point) in contour.points.iter().enumerate() {
                        // Calculate world position for this point
                        let world_pos = position + Vec2::new(point.x as f32, point.y as f32);
                        
                        if point_count < 5 {
                            info!("Point {}: raw=({}, {}), sort_pos=({:.1}, {:.1}), world_pos=({:.1}, {:.1})", 
                                  point_count, point.x, point.y, position.x, position.y, world_pos.x, world_pos.y);
                        }
                        
                        // Create the EditPoint with proper Point type
                        let edit_point = EditPoint {
                            position: Point::new(world_pos.x as f64, world_pos.y as f64),
                            point_type: point.point_type,
                        };
                        
                        // Create PointType component
                        let point_type_component = PointType {
                            is_on_curve: matches!(point.point_type, PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve),
                        };
                        
                        // Spawn the point entity
                        commands.spawn((
                            Transform::from_xyz(world_pos.x, world_pos.y, 10.0), // Z=10 for points above glyphs
                            GlobalTransform::default(),
                            edit_point,
                            point_type_component,
                            GlyphPointReference {
                                glyph_name: sort.glyph_name.clone(),
                                contour_index,
                                point_index,
                            },
                            Selectable,
                            SortPointEntity { sort_entity },
                            PointSortParent(sort_entity),
                            Name::new(format!("Point[{},{}]", contour_index, point_index)),
                        ));
                        
                        point_count += 1;
                        info!("Spawned point {} at world position ({:.1}, {:.1})", 
                              point_count, world_pos.x, world_pos.y);
                    }
                }
                
                debug!("Spawned {} points for active sort '{}'", point_count, sort.glyph_name);
            }
        }
    }
}

/// Despawn inactive sort points optimized
pub fn despawn_inactive_sort_points_optimized(
    mut commands: Commands,
    inactive_sort_query: Query<Entity, (With<InactiveSort>, Changed<InactiveSort>)>,
    point_query: Query<(Entity, &PointSortParent)>,
) {
    for inactive_sort_entity in inactive_sort_query.iter() {
        // Find and despawn all points belonging to this sort
        let mut despawn_count = 0;
        for (point_entity, parent) in point_query.iter() {
            if parent.0 == inactive_sort_entity {
                commands.entity(point_entity).despawn();
                despawn_count += 1;
            }
        }
        
        if despawn_count > 0 {
            debug!("Despawned {} points for inactive sort", despawn_count);
        }
    }
}
