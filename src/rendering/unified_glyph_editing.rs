//! Unified glyph editing rendering system
//!
//! This module combines all glyph editing rendering (points, outlines, handles) into a single
//! system to eliminate any visual lag between components during nudging operations.

#![allow(clippy::too_many_arguments)]

use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selected,
};
use crate::editing::sort::ActiveSort;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use std::collections::{HashMap, HashSet};

// Lyon imports for filled glyph tessellation
use lyon::geom::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex,
    VertexBuffers,
};

/// Component to mark entities as unified glyph editing elements
#[derive(Component)]
pub struct UnifiedGlyphElement {
    pub element_type: UnifiedElementType,
    pub sort_entity: Entity,
}

/// Types of unified glyph editing elements
#[derive(Debug, Clone)]
pub enum UnifiedElementType {
    Point {
        point_entity: Entity,
        is_outer: bool,
    },
    OutlineSegment,
    Handle,
}

/// Resource to track unified editing entities
#[derive(Resource, Default)]
pub struct UnifiedGlyphEntities {
    pub elements: HashMap<Entity, Vec<Entity>>, // sort_entity -> element entities
}

/// Resource to track when sorts need visual updates (prevents unnecessary rebuilding)
#[derive(Resource, Default)]
pub struct SortVisualUpdateTracker {
    pub needs_update: bool,
}

/// Z-levels for proper layering
const UNIFIED_HANDLE_Z: f32 = 7.0; // Behind outlines
const UNIFIED_OUTLINE_Z: f32 = 8.0; // Above handles, behind points
const UNIFIED_POINT_Z: f32 = 10.0; // Unselected points
const UNIFIED_SELECTED_POINT_Z: f32 = 15.0; // Selected points - always above unselected

/// Unified system that renders all sorts - both active (with points/handles) and inactive (filled outlines)
/// This eliminates the need for the separate mesh_glyph_outline system and coordination complexity
#[allow(clippy::type_complexity)]
pub fn render_unified_glyph_editing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut unified_entities: ResMut<UnifiedGlyphEntities>,
    mut update_tracker: ResMut<SortVisualUpdateTracker>,
    camera_scale: Res<CameraResponsiveScale>,
    // Include ALL sorts (both active and inactive)
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<ActiveSort>,
    >,
    inactive_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        (With<crate::editing::sort::InactiveSort>, Without<ActiveSort>),
    >,
    point_query: Query<
        (
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
            Option<&Selected>,
            &SortPointEntity,
        ),
        With<SortPointEntity>,
    >,
    app_state: Option<Res<crate::core::state::AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    existing_elements: Query<(Entity, &UnifiedGlyphElement)>,
    // Debug: Check entities with just SortPointEntity
    existing_sort_points: Query<Entity, With<SortPointEntity>>,
    theme: Res<CurrentTheme>,
) {
    // PERFORMANCE: Early exit if no sorts to render
    let active_count = active_sort_query.iter().count();
    let inactive_count = inactive_sort_query.iter().count();
    
    // Only rebuild if we actually need to update (prevents flash)
    if !update_tracker.needs_update {
        return;
    }
    
    let point_count = point_query.iter().count();
    info!("🎨 UNIFIED RENDERING EXECUTING: active_sorts={}, inactive_sorts={}, points={}", active_count, inactive_count, point_count);
    
    // Debug: Check what components all entities with SortPointEntity have
    if point_count == 0 && !existing_sort_points.is_empty() {
        let all_sort_point_entities = existing_sort_points.iter().count();
        info!("🔍 DEBUG: {} entities with SortPointEntity exist, but 0 match full point query - component mismatch!", all_sort_point_entities);
        
        // Let's see what the first few SortPointEntity entities actually have
        for (i, entity) in existing_sort_points.iter().enumerate() {
            if i >= 3 { break; } // Only check first 3 for debugging
            info!("🔍 DEBUG: Entity {:?} has SortPointEntity", entity);
        }
    }
    
    if active_count == 0 && inactive_count == 0 {
        update_tracker.needs_update = false;
        return;
    }
    
    // Clear the update flag since we're processing it now
    update_tracker.needs_update = false;

    // PERFORMANCE FIX: Only clear elements for sorts that have changed activation status
    // Collect all current sort entities
    let mut current_active_sorts = HashSet::new();
    let mut current_inactive_sorts = HashSet::new();
    
    for (sort_entity, _, _) in active_sort_query.iter() {
        current_active_sorts.insert(sort_entity);
    }
    
    for (sort_entity, _, _) in inactive_sort_query.iter() {
        current_inactive_sorts.insert(sort_entity);
    }
    
    let all_current_sorts: HashSet<_> = current_active_sorts.union(&current_inactive_sorts).cloned().collect();
    
    // CRITICAL FIX: Only clear elements for sorts that actually changed or no longer exist
    // This prevents cross contamination between sorts during placement
    let mut sorts_to_clear = HashSet::new();
    
    // Find sorts that no longer exist
    for &tracked_sort in unified_entities.elements.keys() {
        if !all_current_sorts.contains(&tracked_sort) {
            sorts_to_clear.insert(tracked_sort);
        }
    }
    
    // Find sorts that changed state by checking change detection queries
    for (sort_entity, _, _) in active_sort_query.iter() {
        sorts_to_clear.insert(sort_entity);
    }
    for (sort_entity, _, _) in inactive_sort_query.iter() {
        sorts_to_clear.insert(sort_entity);
    }
    
    // SELECTIVE CLEARING: Only despawn elements for sorts that actually changed
    // This prevents cross contamination between sorts during placement
    let mut cleared_count = 0;
    let mut skipped_count = 0;
    let total_count = existing_elements.iter().count();
    
    for (element_entity, unified_element) in existing_elements.iter() {
        // Only despawn elements that belong to sorts that need clearing
        if sorts_to_clear.contains(&unified_element.sort_entity) {
            commands.entity(element_entity).despawn();
            cleared_count += 1;
        } else {
            skipped_count += 1;
        }
    }
    
    info!("🧹 SELECTIVE CLEANUP: Cleared {}/{} elements, skipped {} (sorts_to_clear: {}, total_sorts: {})", 
          cleared_count, total_count, skipped_count, sorts_to_clear.len(), all_current_sorts.len());
    
    // Only clear tracking for sorts that changed, not all sorts
    if sorts_to_clear.len() == all_current_sorts.len() {
        // If all sorts changed, clear everything (fallback to nuclear approach)
        unified_entities.elements.clear();
    } else {
        // Selective clearing: only remove tracking for changed sorts
        for sort_entity in &sorts_to_clear {
            unified_entities.elements.remove(sort_entity);
        }
    }

    // Process ACTIVE sorts (with editing capabilities) - only those that changed
    for (sort_entity, sort, sort_transform) in active_sort_query.iter() {
        // Skip sorts that don't need re-rendering (selective update)
        if !sorts_to_clear.contains(&sort_entity) && !sorts_to_clear.is_empty() {
            continue;
        }
        
        let sort_position = sort_transform.translation.truncate();
        let mut element_entities = Vec::new();

        // Collect all points for this specific sort entity (prevents cross contamination)
        let mut sort_points = Vec::new();
        let mut checked_points = 0;
        let mut matching_points = 0;
        
        for (point_entity, point_transform, point_ref, point_type, selected, sort_point_entity) in
            point_query.iter()
        {
            checked_points += 1;
            // CRITICAL FIX: Filter by sort entity, not glyph name, to prevent cross contamination
            // Only include points that belong to this specific sort entity
            if sort_point_entity.sort_entity == sort_entity {
                matching_points += 1;
                sort_points.push((
                    point_entity,
                    point_transform.translation.truncate(),
                    point_ref,
                    point_type,
                    selected.is_some(),
                ));
            } else if checked_points <= 3 {
                // Log first few mismatches for debugging
                info!("🔍 SORT_ENTITY MISMATCH: Point {:?} belongs to sort {:?}, not current sort {:?}", 
                      point_entity, sort_point_entity.sort_entity, sort_entity);
            }
        }
        
        info!("🔍 UNIFIED POINT COLLECTION: Sort '{}' found {}/{} matching points, sort_points.len()={}", sort.glyph_name, matching_points, checked_points, sort_points.len());

        if !sort_points.is_empty() {
            info!("🎨 RENDERING COMPONENTS: {} points for sort '{}'", sort_points.len(), sort.glyph_name);
            // UNIFIED RENDERING: Render all components using the same live Transform data

            // 1. Render outlines using live Transform positions
            render_unified_outline(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut element_entities,
                sort_entity,
                &sort_points,
                sort_position,
                fontir_app_state.as_deref(),
                app_state.as_deref(),
                &camera_scale,
            );

            // 2. Render handles using live Transform positions
            render_unified_handles(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut element_entities,
                sort_entity,
                &sort_points,
                &camera_scale,
            );

            // 3. Render points using live Transform positions
            render_unified_points(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut element_entities,
                sort_entity,
                &sort_points,
                &camera_scale,
                &theme,
            );
        } else {
            info!("🚫 NO POINTS: Rendering static outline for sort '{}' (no points found)", sort.glyph_name);
            // No points visible, render static outline from FontIR/UFO data
            render_static_outline(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut element_entities,
                sort_entity,
                &sort.glyph_name,
                sort_position,
                fontir_app_state.as_deref(),
                app_state.as_deref(),
                &camera_scale,
            );
        }

        unified_entities
            .elements
            .insert(sort_entity, element_entities);
    }

    // Process INACTIVE sorts (filled outlines only, no points/handles) - only those that changed
    for (sort_entity, sort, sort_transform) in inactive_sort_query.iter() {
        // Skip sorts that don't need re-rendering (selective update)
        if !sorts_to_clear.contains(&sort_entity) && !sorts_to_clear.is_empty() {
            continue;
        }
        
        let sort_position = sort_transform.translation.truncate();
        let mut element_entities = Vec::new();

        // Render filled outline for inactive sorts
        render_filled_outline(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut element_entities,
            sort_entity,
            &sort.glyph_name,
            sort_position,
            fontir_app_state.as_deref(),
            app_state.as_deref(),
            &camera_scale,
            &theme,
        );

        unified_entities
            .elements
            .insert(sort_entity, element_entities);
    }
}

/// Render filled shapes for inactive sorts using Lyon tessellation
fn render_filled_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    glyph_name: &str,
    position: Vec2,
    fontir_state: Option<&crate::core::state::FontIRAppState>,
    _app_state: Option<&crate::core::state::AppState>,
    _camera_scale: &CameraResponsiveScale,
    _theme: &CurrentTheme,
) {
    if let Some(fontir_state) = fontir_state {
        if let Some(paths) = fontir_state.get_glyph_paths_with_edits(glyph_name) {
            // Combine all contours into a single Lyon path for proper winding rule handling
            let mut lyon_path_builder = Path::builder();
            
            // Convert all kurbo paths (contours) to a single Lyon path
            for kurbo_path in paths {
                for element in kurbo_path.elements().iter() {
                    match element {
                        kurbo::PathEl::MoveTo(pt) => {
                            lyon_path_builder.begin(point(pt.x as f32, pt.y as f32));
                        },
                        kurbo::PathEl::LineTo(pt) => {
                            lyon_path_builder.line_to(point(pt.x as f32, pt.y as f32));
                        },
                        kurbo::PathEl::CurveTo(c1, c2, pt) => {
                            lyon_path_builder.cubic_bezier_to(
                                point(c1.x as f32, c1.y as f32),
                                point(c2.x as f32, c2.y as f32),
                                point(pt.x as f32, pt.y as f32),
                            );
                        },
                        kurbo::PathEl::QuadTo(c, pt) => {
                            lyon_path_builder.quadratic_bezier_to(
                                point(c.x as f32, c.y as f32),
                                point(pt.x as f32, pt.y as f32),
                            );
                        },
                        kurbo::PathEl::ClosePath => {
                            lyon_path_builder.close();
                        }
                    }
                }
            }
            
            let lyon_path = lyon_path_builder.build();
            
            // Tessellate the combined path for filled rendering with proper winding rules
            let mut tessellator = FillTessellator::new();
            let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
            
            let tessellation_result = tessellator.tessellate_path(
                &lyon_path,
                &FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    [vertex.position().x, vertex.position().y]
                }),
            );
            
            if tessellation_result.is_ok() && !geometry.vertices.is_empty() {
                // Convert tessellated geometry to Bevy mesh
                let vertices: Vec<[f32; 3]> = geometry.vertices
                    .iter()
                    .map(|&[x, y]| [x + position.x, y + position.y, 0.0])
                    .collect();
                
                let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];
                let uvs = vec![[0.0, 0.0]; vertices.len()]; // Simple UV mapping
                
                let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, default());
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                mesh.insert_indices(bevy::render::mesh::Indices::U32(geometry.indices));
                
                // Create filled mesh entity
                let entity = commands.spawn((
                    UnifiedGlyphElement { 
                        element_type: UnifiedElementType::OutlineSegment,
                        sort_entity,
                    },
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(FILLED_GLYPH_COLOR))),
                    Transform::from_translation(Vec3::new(0.0, 0.0, UNIFIED_OUTLINE_Z)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                )).id();
                
                element_entities.push(entity);
            }
        }
    }
}

/// Render outline using live Transform positions from points
fn render_unified_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    sort_points: &[(Entity, Vec2, &GlyphPointReference, &PointType, bool)],
    sort_position: Vec2,
    fontir_state: Option<&crate::core::state::FontIRAppState>,
    _app_state: Option<&crate::core::state::AppState>,
    camera_scale: &CameraResponsiveScale,
) {
    // Build position map from live Transform data
    let mut live_positions = HashMap::new();
    for (_entity, world_pos, point_ref, point_type, _selected) in sort_points {
        let relative_pos = *world_pos - sort_position;
        live_positions.insert(
            (point_ref.contour_index, point_ref.point_index),
            (relative_pos, point_type.is_on_curve),
        );
    }

    // Get original path structure and render with live positions
    if let Some(fontir_state) = fontir_state {
        if let Some(original_paths) = fontir_state
            .get_glyph_paths_with_edits(&sort_points[0].2.glyph_name)
        {
            render_fontir_outline_unified(
                commands,
                meshes,
                materials,
                element_entities,
                sort_entity,
                &original_paths,
                &live_positions,
                sort_position,
                camera_scale,
            );
        }
    }
}

/// Render handles using live Transform positions
fn render_unified_handles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    sort_points: &[(Entity, Vec2, &GlyphPointReference, &PointType, bool)],
    camera_scale: &CameraResponsiveScale,
) {
    // Group points by contour
    let mut contours: HashMap<usize, Vec<_>> = HashMap::new();
    for point_data in sort_points {
        contours
            .entry(point_data.2.contour_index)
            .or_default()
            .push(point_data);
    }

    // Render handles for each contour
    for contour_points in contours.values() {
        if contour_points.len() < 2 {
            continue;
        }

        // Sort by point index
        let mut sorted_points = contour_points.clone();
        sorted_points.sort_by_key(|p| p.2.point_index);

        // Create handles between on-curve and off-curve points
        for i in 0..sorted_points.len() {
            let current = sorted_points[i];
            let next = sorted_points[(i + 1) % sorted_points.len()];

            // Draw handle if one is on-curve and other is off-curve
            if current.3.is_on_curve != next.3.is_on_curve {
                let entity = spawn_unified_line_mesh(
                    commands,
                    meshes,
                    materials,
                    current.1,
                    next.1,
                    1.0, // 1px width
                    HANDLE_LINE_COLOR,
                    UNIFIED_HANDLE_Z,
                    sort_entity,
                    UnifiedElementType::Handle,
                    camera_scale,
                );
                element_entities.push(entity);
            }
        }
    }
}

/// Render points using live Transform positions
fn render_unified_points(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    sort_points: &[(Entity, Vec2, &GlyphPointReference, &PointType, bool)],
    camera_scale: &CameraResponsiveScale,
    theme: &CurrentTheme,
) {
    for (point_entity, position, _point_ref, point_type, is_selected) in
        sort_points
    {
        // Determine colors and z-depth for two-layer system
        let (primary_color, secondary_color, base_z) = if *is_selected {
            (
                theme.theme().selected_primary_color(),
                theme.theme().selected_secondary_color(),
                UNIFIED_SELECTED_POINT_Z,
            )
        } else if point_type.is_on_curve {
            (
                theme.theme().on_curve_primary_color(),
                theme.theme().on_curve_secondary_color(),
                UNIFIED_POINT_Z,
            )
        } else {
            (
                theme.theme().off_curve_primary_color(),
                theme.theme().off_curve_secondary_color(),
                UNIFIED_POINT_Z,
            )
        };

        // Create the three-layer point shape
        if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            // On-curve points: square with three layers
            let base_size =
                ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT * 2.0;
            let size = camera_scale.adjusted_point_size(base_size);

            // Layer 1: Base shape (full width) - primary color
            let entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: true,
                        },
                        sort_entity,
                    },
                    Sprite {
                        color: primary_color,
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    Transform::from_translation(position.extend(base_z)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(entity);

            // Layer 2: Slightly smaller shape - secondary color
            let secondary_size = size * 0.7;
            let secondary_entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: false,
                        },
                        sort_entity,
                    },
                    Sprite {
                        color: secondary_color,
                        custom_size: Some(Vec2::splat(secondary_size)),
                        ..default()
                    },
                    Transform::from_translation(position.extend(base_z + 1.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(secondary_entity);

            // Layer 3: Small center shape - primary color (only for non-selected points)
            if !*is_selected {
                let center_size = size * ON_CURVE_INNER_CIRCLE_RATIO;
                let center_entity = commands
                    .spawn((
                        UnifiedGlyphElement {
                            element_type: UnifiedElementType::Point {
                                point_entity: *point_entity,
                                is_outer: false,
                            },
                            sort_entity,
                        },
                        Sprite {
                            color: primary_color,
                            custom_size: Some(Vec2::splat(center_size)),
                            ..default()
                        },
                        Transform::from_translation(
                            position.extend(base_z + 2.0),
                        ),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    ))
                    .id();
                element_entities.push(center_entity);
            }
        } else {
            // Off-curve points and circular on-curve points: circle with three layers
            let base_radius = if point_type.is_on_curve {
                ON_CURVE_POINT_RADIUS
            } else {
                OFF_CURVE_POINT_RADIUS
            };
            let radius = camera_scale.adjusted_point_size(base_radius);

            // Layer 1: Base circle (full size) - primary color
            let entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: true,
                        },
                        sort_entity,
                    },
                    Mesh2d(meshes.add(Circle::new(radius))),
                    MeshMaterial2d(
                        materials.add(ColorMaterial::from_color(primary_color)),
                    ),
                    Transform::from_translation(position.extend(base_z)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(entity);

            // Layer 2: Slightly smaller circle - secondary color
            let secondary_radius = radius * 0.7;
            let secondary_entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: false,
                        },
                        sort_entity,
                    },
                    Mesh2d(meshes.add(Circle::new(secondary_radius))),
                    MeshMaterial2d(
                        materials
                            .add(ColorMaterial::from_color(secondary_color)),
                    ),
                    Transform::from_translation(position.extend(base_z + 1.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(secondary_entity);

            // Layer 3: Small center circle - primary color (only for non-selected points)
            if !*is_selected {
                let center_radius = radius
                    * if point_type.is_on_curve {
                        ON_CURVE_INNER_CIRCLE_RATIO
                    } else {
                        OFF_CURVE_INNER_CIRCLE_RATIO
                    };
                let center_entity = commands
                    .spawn((
                        UnifiedGlyphElement {
                            element_type: UnifiedElementType::Point {
                                point_entity: *point_entity,
                                is_outer: false,
                            },
                            sort_entity,
                        },
                        Mesh2d(meshes.add(Circle::new(center_radius))),
                        MeshMaterial2d(
                            materials
                                .add(ColorMaterial::from_color(primary_color)),
                        ),
                        Transform::from_translation(
                            position.extend(base_z + 2.0),
                        ),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    ))
                    .id();
                element_entities.push(center_entity);
            }
        }

        // Add crosshairs for selected points using two-color system
        if *is_selected {
            let base_line_size = if point_type.is_on_curve {
                ON_CURVE_POINT_RADIUS
            } else {
                OFF_CURVE_POINT_RADIUS
            };
            let line_size = camera_scale.adjusted_point_size(base_line_size);
            let line_width = camera_scale.adjusted_line_width();

            // Make crosshair lines slightly shorter to fit within point bounds
            let crosshair_length = line_size * 1.6;

            // Horizontal line - primary color only
            let h_primary_entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: false,
                        },
                        sort_entity,
                    },
                    Sprite {
                        color: primary_color,
                        custom_size: Some(Vec2::new(
                            crosshair_length,
                            line_width,
                        )),
                        ..default()
                    },
                    Transform::from_translation(position.extend(base_z + 3.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(h_primary_entity);

            // Vertical line - primary color only
            let v_primary_entity = commands
                .spawn((
                    UnifiedGlyphElement {
                        element_type: UnifiedElementType::Point {
                            point_entity: *point_entity,
                            is_outer: false,
                        },
                        sort_entity,
                    },
                    Sprite {
                        color: primary_color,
                        custom_size: Some(Vec2::new(
                            line_width,
                            crosshair_length,
                        )),
                        ..default()
                    },
                    Transform::from_translation(position.extend(base_z + 3.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();
            element_entities.push(v_primary_entity);
        }
    }
}

/// Render static outline when no points are visible
fn render_static_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    glyph_name: &str,
    position: Vec2,
    fontir_state: Option<&crate::core::state::FontIRAppState>,
    _app_state: Option<&crate::core::state::AppState>,
    camera_scale: &CameraResponsiveScale,
) {
    if let Some(fontir_state) = fontir_state {
        if let Some(paths) = fontir_state.get_glyph_paths_with_edits(glyph_name)
        {
            // Render static outline from FontIR working copy
            for path in paths {
                let elements: Vec<_> = path.elements().iter().collect();
                let mut current_pos = None;

                for element in elements {
                    match element {
                        kurbo::PathEl::MoveTo(pt) => {
                            current_pos = Some(
                                Vec2::new(pt.x as f32, pt.y as f32) + position,
                            );
                        }
                        kurbo::PathEl::LineTo(pt) => {
                            if let Some(start) = current_pos {
                                let end = Vec2::new(pt.x as f32, pt.y as f32)
                                    + position;
                                let entity = spawn_unified_line_mesh(
                                    commands,
                                    meshes,
                                    materials,
                                    start,
                                    end,
                                    1.0,
                                    PATH_STROKE_COLOR,
                                    UNIFIED_OUTLINE_Z,
                                    sort_entity,
                                    UnifiedElementType::OutlineSegment,
                                    camera_scale,
                                );
                                element_entities.push(entity);
                                current_pos = Some(end);
                            }
                        }
                        kurbo::PathEl::CurveTo(c1, c2, pt) => {
                            if let Some(start) = current_pos {
                                let end = Vec2::new(pt.x as f32, pt.y as f32)
                                    + position;
                                let cp1 = Vec2::new(c1.x as f32, c1.y as f32)
                                    + position;
                                let cp2 = Vec2::new(c2.x as f32, c2.y as f32)
                                    + position;

                                // Tessellate curve
                                let segments = 32;
                                debug!("Unified system: Rendering CurveTo for sort {:?} with {} segments", sort_entity, segments);
                                let mut last_pos = start;

                                for i in 1..=segments {
                                    let t = i as f32 / segments as f32;
                                    let t2 = t * t;
                                    let t3 = t2 * t;
                                    let mt = 1.0 - t;
                                    let mt2 = mt * mt;
                                    let mt3 = mt2 * mt;

                                    let curve_pos = Vec2::new(
                                        mt3 * start.x
                                            + 3.0 * mt2 * t * cp1.x
                                            + 3.0 * mt * t2 * cp2.x
                                            + t3 * end.x,
                                        mt3 * start.y
                                            + 3.0 * mt2 * t * cp1.y
                                            + 3.0 * mt * t2 * cp2.y
                                            + t3 * end.y,
                                    );

                                    let entity = spawn_unified_line_mesh(
                                        commands,
                                        meshes,
                                        materials,
                                        last_pos,
                                        curve_pos,
                                        1.0,
                                        PATH_STROKE_COLOR,
                                        UNIFIED_OUTLINE_Z,
                                        sort_entity,
                                        UnifiedElementType::OutlineSegment,
                                        camera_scale,
                                    );
                                    element_entities.push(entity);
                                    last_pos = curve_pos;
                                }
                                current_pos = Some(end);
                            }
                        }
                        kurbo::PathEl::QuadTo(c, pt) => {
                            if let Some(start) = current_pos {
                                let end = Vec2::new(pt.x as f32, pt.y as f32)
                                    + position;
                                let cp = Vec2::new(c.x as f32, c.y as f32)
                                    + position;

                                // Tessellate quadratic curve
                                let segments = 24;
                                let mut last_pos = start;

                                for i in 1..=segments {
                                    let t = i as f32 / segments as f32;
                                    let mt = 1.0 - t;

                                    let curve_pos = Vec2::new(
                                        mt * mt * start.x
                                            + 2.0 * mt * t * cp.x
                                            + t * t * end.x,
                                        mt * mt * start.y
                                            + 2.0 * mt * t * cp.y
                                            + t * t * end.y,
                                    );

                                    let entity = spawn_unified_line_mesh(
                                        commands,
                                        meshes,
                                        materials,
                                        last_pos,
                                        curve_pos,
                                        1.0,
                                        PATH_STROKE_COLOR,
                                        UNIFIED_OUTLINE_Z,
                                        sort_entity,
                                        UnifiedElementType::OutlineSegment,
                                        camera_scale,
                                    );
                                    element_entities.push(entity);
                                    last_pos = curve_pos;
                                }
                                current_pos = Some(end);
                            }
                        }
                        kurbo::PathEl::ClosePath => {
                            // ClosePath doesn't add geometry
                        }
                    }
                }
            }
        }
    }
}

/// Render FontIR outline using live positions
fn render_fontir_outline_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    element_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    original_paths: &[kurbo::BezPath],
    live_positions: &HashMap<(usize, usize), (Vec2, bool)>,
    sort_position: Vec2,
    camera_scale: &CameraResponsiveScale,
) {
    // Process each contour with live positions
    for (contour_idx, original_path) in original_paths.iter().enumerate() {
        let elements: Vec<_> = original_path.elements().iter().collect();
        let mut element_point_index = 0;
        let mut current_pos = None;

        for element in elements {
            match element {
                kurbo::PathEl::MoveTo(_) => {
                    if let Some((live_pos, _)) =
                        live_positions.get(&(contour_idx, element_point_index))
                    {
                        current_pos = Some(*live_pos + sort_position);
                    } else if let kurbo::PathEl::MoveTo(pt) = element {
                        current_pos = Some(
                            Vec2::new(pt.x as f32, pt.y as f32) + sort_position,
                        );
                    }
                    element_point_index += 1;
                }
                kurbo::PathEl::LineTo(_) => {
                    if let Some(start) = current_pos {
                        let end = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::LineTo(pt) = element {
                            Vec2::new(pt.x as f32, pt.y as f32) + sort_position
                        } else {
                            start
                        };

                        let entity = spawn_unified_line_mesh(
                            commands,
                            meshes,
                            materials,
                            start,
                            end,
                            1.0,
                            PATH_STROKE_COLOR,
                            UNIFIED_OUTLINE_Z,
                            sort_entity,
                            UnifiedElementType::OutlineSegment,
                            camera_scale,
                        );
                        element_entities.push(entity);
                        current_pos = Some(end);
                    }
                    element_point_index += 1;
                }
                kurbo::PathEl::CurveTo(_, _, _) => {
                    if let Some(start) = current_pos {
                        // Get live positions for control points and end point
                        let cp1 = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::CurveTo(c1, _, _) = element
                        {
                            Vec2::new(c1.x as f32, c1.y as f32) + sort_position
                        } else {
                            start
                        };

                        let cp2 = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index + 1))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::CurveTo(_, c2, _) = element
                        {
                            Vec2::new(c2.x as f32, c2.y as f32) + sort_position
                        } else {
                            start
                        };

                        let end = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index + 2))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::CurveTo(_, _, pt) = element
                        {
                            Vec2::new(pt.x as f32, pt.y as f32) + sort_position
                        } else {
                            start
                        };

                        // Tessellate cubic curve with live positions
                        let segments = 32;
                        let mut last_pos = start;

                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let mt = 1.0 - t;
                            let mt2 = mt * mt;
                            let mt3 = mt2 * mt;

                            let curve_pos = Vec2::new(
                                mt3 * start.x
                                    + 3.0 * mt2 * t * cp1.x
                                    + 3.0 * mt * t2 * cp2.x
                                    + t3 * end.x,
                                mt3 * start.y
                                    + 3.0 * mt2 * t * cp1.y
                                    + 3.0 * mt * t2 * cp2.y
                                    + t3 * end.y,
                            );

                            let entity = spawn_unified_line_mesh(
                                commands,
                                meshes,
                                materials,
                                last_pos,
                                curve_pos,
                                1.0,
                                PATH_STROKE_COLOR,
                                UNIFIED_OUTLINE_Z,
                                sort_entity,
                                UnifiedElementType::OutlineSegment,
                                camera_scale,
                            );
                            element_entities.push(entity);
                            last_pos = curve_pos;
                        }
                        current_pos = Some(end);
                    }
                    element_point_index += 3;
                }
                kurbo::PathEl::QuadTo(_, _) => {
                    if let Some(start) = current_pos {
                        let cp = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::QuadTo(c, _) = element {
                            Vec2::new(c.x as f32, c.y as f32) + sort_position
                        } else {
                            start
                        };

                        let end = if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index + 1))
                        {
                            *live_pos + sort_position
                        } else if let kurbo::PathEl::QuadTo(_, pt) = element {
                            Vec2::new(pt.x as f32, pt.y as f32) + sort_position
                        } else {
                            start
                        };

                        // Tessellate quadratic curve
                        let segments = 24;
                        let mut last_pos = start;

                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let mt = 1.0 - t;

                            let curve_pos = Vec2::new(
                                mt * mt * start.x
                                    + 2.0 * mt * t * cp.x
                                    + t * t * end.x,
                                mt * mt * start.y
                                    + 2.0 * mt * t * cp.y
                                    + t * t * end.y,
                            );

                            let entity = spawn_unified_line_mesh(
                                commands,
                                meshes,
                                materials,
                                last_pos,
                                curve_pos,
                                1.0,
                                PATH_STROKE_COLOR,
                                UNIFIED_OUTLINE_Z,
                                sort_entity,
                                UnifiedElementType::OutlineSegment,
                                camera_scale,
                            );
                            element_entities.push(entity);
                            last_pos = curve_pos;
                        }
                        current_pos = Some(end);
                    }
                    element_point_index += 2;
                }
                kurbo::PathEl::ClosePath => {
                    // Draw closing line from current position back to contour start
                    if let Some(end) = current_pos {
                        // Find the start position of this contour (first MoveTo)
                        let contour_elements: Vec<_> = original_path.elements().iter().collect();
                        if let Some(kurbo::PathEl::MoveTo(start_pt)) = contour_elements.first() {
                            let start = if let Some((live_pos, _)) = live_positions.get(&(contour_idx, 0)) {
                                *live_pos + sort_position
                            } else {
                                Vec2::new(start_pt.x as f32, start_pt.y as f32) + sort_position
                            };
                            
                            // Only draw closing line if start and end are different
                            if (end - start).length() > 0.1 {
                                let entity = spawn_unified_line_mesh(
                                    commands,
                                    meshes,
                                    materials,
                                    end,
                                    start,
                                    1.0,
                                    PATH_STROKE_COLOR,
                                    UNIFIED_OUTLINE_Z,
                                    sort_entity,
                                    UnifiedElementType::OutlineSegment,
                                    camera_scale,
                                );
                                element_entities.push(entity);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper to spawn a line mesh entity for unified rendering
fn spawn_unified_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: Color,
    z: f32,
    sort_entity: Entity,
    element_type: UnifiedElementType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    let adjusted_width = camera_scale.adjusted_line_width() * width;
    let line_mesh = crate::rendering::mesh_utils::create_line_mesh(
        start,
        end,
        adjusted_width,
    );

    commands
        .spawn((
            UnifiedGlyphElement {
                element_type,
                sort_entity,
            },
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(
                (start.x + end.x) * 0.5,
                (start.y + end.y) * 0.5,
                z,
            ),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id()
}

// Type aliases for complex query types
type ActiveSortChangeQuery<'w, 's> = Query<'w, 's, Entity, (With<ActiveSort>, Or<(Changed<ActiveSort>, Added<ActiveSort>)>)>;
type InactiveSortChangeQuery<'w, 's> = Query<'w, 's, Entity, (With<crate::editing::sort::InactiveSort>, Or<(Changed<crate::editing::sort::InactiveSort>, Added<crate::editing::sort::InactiveSort>)>)>;

/// System to detect when sorts change and trigger visual updates
fn detect_sort_changes(
    mut update_tracker: ResMut<SortVisualUpdateTracker>,
    active_sort_query: ActiveSortChangeQuery,
    inactive_sort_query: InactiveSortChangeQuery,
    removed_active: RemovedComponents<ActiveSort>,
    removed_inactive: RemovedComponents<crate::editing::sort::InactiveSort>,
    // CRITICAL FIX: Also trigger updates when points are available for active sorts
    point_query: Query<&crate::systems::sort_manager::SortPointEntity>,
    buffer_active_sorts: Query<Entity, (With<ActiveSort>, With<crate::systems::text_editor_sorts::sort_entities::BufferSortIndex>)>,
) {
    let active_changed = !active_sort_query.is_empty();
    let inactive_changed = !inactive_sort_query.is_empty();
    let removed_active_count = removed_active.len();
    let removed_inactive_count = removed_inactive.len();
    
    // CRITICAL FIX: Also check if active buffer sorts have points but visual update wasn't triggered
    let mut points_ready_for_rendering = false;
    for sort_entity in buffer_active_sorts.iter() {
        let point_count = point_query.iter().filter(|point_parent| point_parent.sort_entity == sort_entity).count();
        if point_count > 0 && !update_tracker.needs_update {
            points_ready_for_rendering = true;
            info!("🔄 POINTS READY: Sort {:?} has {} points but visual update not triggered", sort_entity, point_count);
            break;
        }
    }
    
    let needs_update = active_changed || inactive_changed || removed_active_count > 0 || removed_inactive_count > 0 || points_ready_for_rendering;
    
    if needs_update {
        update_tracker.needs_update = true;
        if points_ready_for_rendering {
            info!("🔄 POINTS READY DETECTED: Setting update flag for active sorts with existing points");
        } else {
            info!("🔄 SORT CHANGES DETECTED: Setting update flag - active_changed: {}, inactive_changed: {}, removed_active: {}, removed_inactive: {}", 
                  active_changed, inactive_changed, removed_active_count, removed_inactive_count);
        }
    }
}

/// Plugin for unified glyph editing rendering
pub struct UnifiedGlyphEditingPlugin;

impl Plugin for UnifiedGlyphEditingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnifiedGlyphEntities>()
           .init_resource::<SortVisualUpdateTracker>()
           .add_systems(Update, (
               detect_sort_changes,
               render_unified_glyph_editing,
           ).chain()
               .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized)
               .after(crate::editing::selection::nudge::handle_nudge_input));
    }
}

