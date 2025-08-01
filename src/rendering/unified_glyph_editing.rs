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
use std::collections::HashMap;

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

/// Z-levels for proper layering
const UNIFIED_HANDLE_Z: f32 = 7.0; // Behind outlines  
const UNIFIED_OUTLINE_Z: f32 = 8.0; // Above handles, behind points
const UNIFIED_POINT_Z: f32 = 10.0; // Unselected points
const UNIFIED_SELECTED_POINT_Z: f32 = 15.0; // Selected points - always above unselected

/// Unified system that renders points, outlines, and handles together for zero lag
#[allow(clippy::type_complexity)]
pub fn render_unified_glyph_editing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut unified_entities: ResMut<UnifiedGlyphEntities>,
    camera_scale: Res<CameraResponsiveScale>,
    // Include ALL active sorts (regular sorts and buffer sorts both use ActiveSort component)
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<ActiveSort>,
    >,
    point_query: Query<
        (
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
            Option<&Selected>,
        ),
        With<SortPointEntity>,
    >,
    app_state: Option<Res<crate::core::state::AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    existing_elements: Query<Entity, With<UnifiedGlyphElement>>,
    _mesh_outline_elements: Query<Entity, With<crate::rendering::mesh_glyph_outline::GlyphOutlineElement>>,
    mut mesh_outline_entities: ResMut<crate::rendering::mesh_glyph_outline::MeshOutlineEntities>,
    mut unified_rendering_sorts: ResMut<crate::rendering::outline_coordination::UnifiedRenderingSorts>,
    theme: Res<CurrentTheme>,
) {
    let active_sort_count = active_sort_query.iter().count();
    if active_sort_count == 0 {
        return; // No active sorts, nothing to render
    }

    // Check which sorts have visible points and should use unified rendering
    let mut sorts_with_points = Vec::new();
    for (sort_entity, sort, _) in active_sort_query.iter() {
        let has_points = point_query.iter().any(|(_, _, point_ref, _, _)| {
            point_ref.glyph_name == sort.glyph_name
        });
        
        if has_points {
            sorts_with_points.push(sort_entity);
        }
    }

    // Update the coordination resource
    unified_rendering_sorts.clear();
    for sort_entity in &sorts_with_points {
        unified_rendering_sorts.insert(*sort_entity);
        debug!("Unified system: Marking sort {:?} for unified rendering", sort_entity);
    }

    // ONLY clear and take over rendering if there are actual points visible
    // This prevents the unified system from interfering when no points are active
    if sorts_with_points.is_empty() {
        return; // Let mesh_glyph_outline handle basic outline rendering
    }

    // Clear existing unified elements only when we're taking over
    for entity in existing_elements.iter() {
        commands.entity(entity).despawn();
    }
    unified_entities.elements.clear();
    
    // CRITICAL: Clear mesh outline entities for sorts we're taking over
    // This prevents double rendering by removing the base outline
    for sort_entity in &sorts_with_points {
        // Remove from mesh outline tracking
        if let Some(entities) = mesh_outline_entities.path_segments.remove(sort_entity) {
            debug!("Unified system: Despawning {} path segment entities for sort {:?}", entities.len(), sort_entity);
            for entity in entities {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                } else {
                    debug!("Unified system: Entity {:?} already despawned", entity);
                }
            }
        }
        if let Some(entities) = mesh_outline_entities.control_handles.remove(sort_entity) {
            debug!("Unified system: Despawning {} control handle entities for sort {:?}", entities.len(), sort_entity);
            for entity in entities {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                } else {
                    debug!("Unified system: Entity {:?} already despawned", entity);
                }
            }
        }
    }

    // Process each active sort
    for (sort_entity, sort, sort_transform) in active_sort_query.iter() {
        let sort_position = sort_transform.translation.truncate();
        let mut element_entities = Vec::new();

        // Collect all points for this sort from the point query
        let mut sort_points = Vec::new();
        for (point_entity, point_transform, point_ref, point_type, selected) in
            point_query.iter()
        {
            if point_ref.glyph_name == sort.glyph_name {
                sort_points.push((
                    point_entity,
                    point_transform.translation.truncate(),
                    point_ref,
                    point_type,
                    selected.is_some(),
                ));
            }
        }

        if !sort_points.is_empty() {
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
                    Transform::from_translation(
                        position.extend(base_z),
                    ),
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
                    Transform::from_translation(
                        position.extend(base_z + 1.0),
                    ),
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
                    Transform::from_translation(
                        position.extend(base_z),
                    ),
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
                    Transform::from_translation(
                        position.extend(base_z + 1.0),
                    ),
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
                            materials.add(ColorMaterial::from_color(primary_color)),
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
                    Transform::from_translation(
                        position.extend(base_z + 3.0),
                    ),
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
                    Transform::from_translation(
                        position.extend(base_z + 3.0),
                    ),
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
                    // ClosePath doesn't add geometry
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
    let line_mesh = crate::rendering::mesh_glyph_outline::create_line_mesh(
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

/// Plugin for unified glyph editing rendering
pub struct UnifiedGlyphEditingPlugin;

impl Plugin for UnifiedGlyphEditingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnifiedGlyphEntities>()
           .add_systems(Update, render_unified_glyph_editing
               .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized)
               .after(crate::editing::selection::nudge::handle_nudge_input));
    }
}
