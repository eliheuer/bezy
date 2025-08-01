//! Point entity management for text editor sorts

use crate::core::state::font_data::PointTypeData;
use crate::core::state::{AppState, FontIRAppState};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable,
};
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use crate::geometry::point::EditPoint;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;
use kurbo::{PathEl, Point};

/// Component to track which sort entity a point belongs to
#[derive(Component)]
pub struct PointSortParent(pub Entity);

/// Spawn active sort points optimized
pub fn spawn_active_sort_points_optimized(
    mut commands: Commands,
    active_sort_query: Query<(Entity, &Sort, &Transform), With<ActiveSort>>,
    existing_points: Query<&PointSortParent>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
) {
    let active_sort_count = active_sort_query.iter().count();
    if active_sort_count > 0 {
        info!(
            "Point spawning system called: {} active sorts found",
            active_sort_count
        );
    }

    for (sort_entity, sort, transform) in active_sort_query.iter() {
        // Check if points already exist for this sort
        let has_points =
            existing_points.iter().any(|parent| parent.0 == sort_entity);
        if has_points {
            debug!("Skipping point spawning for sort entity {:?} - points already exist", sort_entity);
            continue; // Skip if points already exist
        }

        info!(
            "Spawning points for active sort entity {:?}, glyph: {}",
            sort_entity, sort.glyph_name
        );
        let position = transform.translation.truncate();

        info!(
            "Spawning points for ACTIVE sort '{}' at position ({:.1}, {:.1})",
            sort.glyph_name, position.x, position.y
        );

        // Try FontIR first, then fallback to AppState
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            // Get FontIR BezPaths for this glyph
            if let Some(paths) = fontir_state.get_glyph_paths(&sort.glyph_name)
            {
                let mut point_count = 0;
                info!("FontIR: Spawning points for active sort '{}' with {} paths", sort.glyph_name, paths.len());

                // Spawn points for each BezPath (contour)
                for (contour_index, path) in paths.iter().enumerate() {
                    let elements: Vec<_> = path.elements().iter().collect();
                    let mut element_point_index = 0; // Track actual point index within the contour

                    for element in elements.iter() {
                        // Extract all points (on-curve and off-curve) from PathEl
                        let points = extract_points_from_path_element(element);

                        for (point_pos, point_type) in points {
                            // Calculate world position for this point
                            let world_pos = position
                                + Vec2::new(
                                    point_pos.x as f32,
                                    point_pos.y as f32,
                                );

                            if point_count < 10 {
                                info!("FontIR Point {}: type={:?}, raw=({:.1}, {:.1}), sort_pos=({:.1}, {:.1}), world_pos=({:.1}, {:.1})", 
                                      point_count, point_type, point_pos.x, point_pos.y, position.x, position.y, world_pos.x, world_pos.y);
                            }

                            // Create the EditPoint
                            let edit_point = EditPoint {
                                position: Point::new(
                                    world_pos.x as f64,
                                    world_pos.y as f64,
                                ),
                                point_type,
                            };

                            // Create PointType component
                            let point_type_component = PointType {
                                is_on_curve: matches!(
                                    point_type,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            };

                            // Spawn the point entity
                            commands.spawn((
                                Transform::from_xyz(
                                    world_pos.x,
                                    world_pos.y,
                                    10.0,
                                ),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                edit_point,
                                point_type_component,
                                GlyphPointReference {
                                    glyph_name: sort.glyph_name.clone(),
                                    contour_index,
                                    point_index: element_point_index,
                                },
                                Selectable,
                                SortPointEntity { sort_entity },
                                PointSortParent(sort_entity),
                                Name::new(format!(
                                    "FontIR_Point[{},{},{}]",
                                    contour_index,
                                    element_point_index,
                                    if point_type == PointTypeData::OffCurve {
                                        "off"
                                    } else {
                                        "on"
                                    }
                                )),
                            ));

                            element_point_index += 1;
                            point_count += 1;
                            if point_count <= 10 {
                                info!("FontIR: Spawned {} point {} at world position ({:.1}, {:.1})", 
                                      if point_type == PointTypeData::OffCurve { "off-curve" } else { "on-curve" },
                                      point_count, world_pos.x, world_pos.y);
                            }
                        }
                    }
                }

                info!(
                    "FontIR: Spawned {} points for active sort '{}'",
                    point_count, sort.glyph_name
                );
            } else {
                warn!("FontIR: No paths found for glyph '{}'", sort.glyph_name);
            }
        } else if let Some(state) = app_state.as_ref() {
            // Fallback to UFO AppState logic
            if let Some(glyph_data) =
                state.workspace.font.get_glyph(&sort.glyph_name)
            {
                if let Some(outline) = &glyph_data.outline {
                    let mut point_count = 0;

                    for (contour_index, contour) in
                        outline.contours.iter().enumerate()
                    {
                        for (point_index, point) in
                            contour.points.iter().enumerate()
                        {
                            let world_pos = position
                                + Vec2::new(point.x as f32, point.y as f32);

                            let edit_point = EditPoint {
                                position: Point::new(
                                    world_pos.x as f64,
                                    world_pos.y as f64,
                                ),
                                point_type: point.point_type,
                            };

                            let point_type_component = PointType {
                                is_on_curve: matches!(
                                    point.point_type,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            };

                            commands.spawn((
                                Transform::from_xyz(
                                    world_pos.x,
                                    world_pos.y,
                                    10.0,
                                ),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
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
                                Name::new(format!(
                                    "UFO_Point[{contour_index},{point_index}]"
                                )),
                            ));

                            point_count += 1;
                        }
                    }

                    info!(
                        "UFO: Spawned {} points for active sort '{}'",
                        point_count, sort.glyph_name
                    );
                } else {
                    warn!(
                        "UFO: No outline found for glyph '{}'",
                        sort.glyph_name
                    );
                }
            } else {
                warn!("UFO: No glyph data found for '{}'", sort.glyph_name);
            }
        } else {
            warn!(
                "Point spawning failed - neither FontIR nor AppState available"
            );
        }
    }
}

/// Despawn inactive sort points optimized
pub fn despawn_inactive_sort_points_optimized(
    mut commands: Commands,
    inactive_sort_query: Query<
        Entity,
        (With<InactiveSort>, Changed<InactiveSort>),
    >,
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

/// Extract all points (on-curve and off-curve) from a kurbo PathEl
fn extract_points_from_path_element(
    element: &PathEl,
) -> Vec<(Point, PointTypeData)> {
    match element {
        PathEl::MoveTo(pt) => vec![(*pt, PointTypeData::Move)],
        PathEl::LineTo(pt) => vec![(*pt, PointTypeData::Line)],
        PathEl::CurveTo(c1, c2, pt) => vec![
            (*c1, PointTypeData::OffCurve), // First control point
            (*c2, PointTypeData::OffCurve), // Second control point
            (*pt, PointTypeData::Curve),    // End point
        ],
        PathEl::QuadTo(c, pt) => vec![
            (*c, PointTypeData::OffCurve), // Control point
            (*pt, PointTypeData::Curve),   // End point
        ],
        PathEl::ClosePath => vec![], // ClosePath doesn't have points
    }
}
