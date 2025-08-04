//! Mesh-based glyph outline rendering
//!
//! This module replaces gizmo-based rendering with mesh/sprite-based rendering
//! to allow proper z-ordering and layering control.
//!
//! Performance optimizations:
//! - Entity pooling to avoid constant spawn/despawn
//! - Batched mesh creation
//! - Efficient curve tessellation

#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::core::state::{ContourData, OutlineData, PointData, PointTypeData};
use crate::editing::sort::{ActiveSort, Sort};
use crate::rendering::entity_pools::{
    update_outline_entity, EntityPools, PooledEntityType,
};
use crate::rendering::mesh_cache::GlyphMeshCache;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;

// Lyon imports for filled glyph tessellation
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use lyon::geom::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex,
    VertexBuffers,
};
use std::collections::HashMap;

/// Component to mark entities as glyph outline elements
#[derive(Component)]
pub struct GlyphOutlineElement {
    pub element_type: OutlineElementType,
    pub sort_entity: Entity,
}

/// Types of glyph outline elements
#[derive(Debug, Clone)]
pub enum OutlineElementType {
    PathSegment,
    ControlHandle,
    FilledShape, // For filled glyph rendering (inactive sorts)
}

/// Resource to track mesh outline entities for performance (entity pooling)
#[derive(Resource, Default)]
pub struct MeshOutlineEntities {
    pub path_segments: HashMap<Entity, Vec<Entity>>, // sort_entity -> segment entities
    pub control_handles: HashMap<Entity, Vec<Entity>>, // sort_entity -> handle entities
}

/// Z-levels for proper layering
const GLYPH_OUTLINE_Z: f32 = 8.0; // Behind points (10.0) and point backgrounds (5.0)
const CONTROL_HANDLE_Z: f32 = 9.0; // Slightly above outlines

// HANDLE_LINE_WIDTH removed - now using camera-responsive scaling

/// System to render glyph outlines using meshes instead of gizmos
pub fn render_mesh_glyph_outline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut outline_entities: ResMut<MeshOutlineEntities>,
    mut entity_pools: ResMut<EntityPools>,
    mut mesh_cache: ResMut<GlyphMeshCache>,
    // Active sort query for outline rendering (INCLUDES buffer sorts for outline rendering)
    // CHANGE DETECTION: Process sorts when Sort/Transform change OR when they become active
    active_sort_query: Query<
        (Entity, &Sort, &Transform),
        (
            With<ActiveSort>,
            Or<(Changed<Sort>, Changed<Transform>, Added<ActiveSort>)>,
        ),
    >,
    // Buffer sort query for text editor sorts (INACTIVE buffer sorts only - filled rendering)
    // NO CHANGE DETECTION: Always process ALL inactive buffer sorts for consistent filled rendering
    // ONLY inactive buffer sorts should be filled - active buffer sorts get outline rendering
    buffer_sort_query: Query<
        (Entity, &Sort, &Transform),
        (
            With<crate::systems::text_editor_sorts::sort_entities::BufferSortIndex>,
            With<crate::editing::sort::InactiveSort>,
        ),
    >,
    // Inactive sort query for non-buffer sorts (filled rendering for inactive freeform sorts)
    // COMPONENT CHANGE DETECTION: Detect when sorts become inactive (InactiveSort component added)
    inactive_sort_query: Query<
        (Entity, &Sort, &Transform),
        (
            With<crate::editing::sort::InactiveSort>,
            Without<crate::systems::text_editor_sorts::sort_entities::BufferSortIndex>, // Exclude buffer sorts
            Or<(Added<crate::editing::sort::InactiveSort>, Changed<Sort>, Changed<Transform>)>,
        )
    >,
    // Camera-responsive scaling for proper zoom-aware rendering
    camera_scale: Res<
        crate::rendering::camera_responsive::CameraResponsiveScale,
    >,
    // Coordination with unified rendering system
    unified_rendering_sorts: Res<
        crate::rendering::outline_coordination::UnifiedRenderingSorts,
    >,
    app_state: Option<Res<crate::core::state::AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    _existing_outlines: Query<Entity, With<GlyphOutlineElement>>,
    // Add point query for live rendering
    point_query: Query<
        (
            Entity,
            &Transform,
            &crate::editing::selection::components::GlyphPointReference,
            &crate::editing::selection::components::PointType,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    _selected_query: Query<
        Entity,
        With<crate::editing::selection::components::Selected>,
    >,
) {
    // System runs frequently - only log when processing sorts
    debug!("MESH OUTLINE SYSTEM: render_mesh_glyph_outline system running");

    // CHANGE DETECTION: Early return only if no active sorts have changed AND no buffer/inactive sorts exist
    let active_sort_count = active_sort_query.iter().count();
    let buffer_sort_count = buffer_sort_query.iter().count();
    let inactive_sort_count = inactive_sort_query.iter().count();

    // Only skip rendering if there are no active sort changes AND no buffer/inactive sorts to render
    if active_sort_count == 0
        && buffer_sort_count == 0
        && inactive_sort_count == 0
    {
        debug!("Outline rendering skipped - no changed active sorts and no buffer/inactive sorts (active: {}, buffer: {}, inactive: {})", active_sort_count, buffer_sort_count, inactive_sort_count);
        return; // No sorts to render - performance optimization
    }

    debug!("Outline rendering proceeding - changed sorts detected (active: {}, buffer: {}, inactive: {})", active_sort_count, buffer_sort_count, inactive_sort_count);

    // Log inactive sort processing with detailed component analysis
    if inactive_sort_count > 0 {
        info!("GLYPH OUTLINE DEBUG: Rendering {} inactive sorts with filled glyphs", inactive_sort_count);
        for (entity, sort, _) in inactive_sort_query.iter() {
            info!("GLYPH OUTLINE DEBUG: Processing inactive sort {:?} (glyph: {}) - WILL RENDER FILLED", entity, sort.glyph_name);
        }
    } else {
        info!("GLYPH OUTLINE DEBUG: No inactive sorts found to render - this means previously active sorts won't show filled glyphs!");
    }

    // SELECTIVE PROCESSING: Collect changed sorts by type for selective rendering
    let mut changed_sort_entities = Vec::new();
    let mut changed_active_sorts = Vec::new();
    let mut changed_buffer_sorts = Vec::new();
    let mut changed_inactive_sorts = Vec::new();

    for sort_data in active_sort_query.iter() {
        changed_sort_entities.push(sort_data.0);
        changed_active_sorts.push(sort_data);
        info!("GLYPH OUTLINE DEBUG: Found ACTIVE sort {:?} (glyph: {}) - will render outline", sort_data.0, sort_data.1.glyph_name);
    }
    for sort_data in buffer_sort_query.iter() {
        changed_sort_entities.push(sort_data.0);
        changed_buffer_sorts.push(sort_data);
        warn!("FILLED RENDERING: Processing INACTIVE buffer sort {:?} (glyph: {}) for filled rendering", sort_data.0, sort_data.1.glyph_name);
    }

    // DEBUG: Log all inactive buffer sorts that should have filled rendering
    info!(
        "GLYPH OUTLINE DEBUG: Total inactive buffer sorts found: {}",
        changed_buffer_sorts.len()
    );
    for sort_data in inactive_sort_query.iter() {
        changed_sort_entities.push(sort_data.0);
        changed_inactive_sorts.push(sort_data);
        info!("GLYPH OUTLINE DEBUG: Found INACTIVE sort {:?} (glyph: {}) - will render filled", sort_data.0, sort_data.1.glyph_name);
    }

    // DISABLED: Entity pooling removed to eliminate race conditions
    // This was causing intermittent issues with root sort filled rendering
    // We can re-add performance optimizations later once the core functionality is stable
    // info!("GLYPH OUTLINE DEBUG: Returning entities to pool for {} changed sorts: {:?}", changed_sort_entities.len(), changed_sort_entities);
    // entity_pools.return_entities_for_changed_sorts(&mut commands, &changed_sort_entities);
    outline_entities.path_segments.clear();
    outline_entities.control_handles.clear();

    debug!(
        "Returned outline entities for {} changed sorts",
        changed_sort_entities.len()
    );

    // ENABLED: Mesh glyph outline rendering for proper z-ordering
    // Removed return to enable mesh-based outlines

    // SELECTIVE RENDERING: Only render changed active sorts not handled by unified system
    for (sort_entity, sort, transform) in changed_active_sorts {
        // Check if entity still exists before processing
        if commands.get_entity(sort_entity).is_err() {
            debug!(
                "Skipping active sort rendering for non-existent entity {:?}",
                sort_entity
            );
            continue;
        }

        // Skip if this sort is being handled by the unified rendering system
        if unified_rendering_sorts.contains(sort_entity) {
            debug!("Mesh system: Skipping sort {:?} - handled by unified rendering system", sort_entity);
            continue;
        }
        debug!(
            "Mesh system: Rendering sort {:?} - not in unified system",
            sort_entity
        );

        let position = transform.translation.truncate();

        // Check if there are any points visible for this sort (indicating active editing mode)
        let has_visible_points =
            point_query.iter().any(|(_, _, point_ref, _)| {
                point_ref.glyph_name == sort.glyph_name
            });

        // PRIORITY: Use live Transform positions when ANY points are visible for immediate sync
        // This ensures points and outline always move together without lag
        if has_visible_points {
            // Use live Transform positions for outline rendering
            if let Some(fontir_state) = fontir_app_state.as_ref() {
                if let Some(_paths) =
                    fontir_state.get_glyph_paths_with_edits(&sort.glyph_name)
                {
                    render_fontir_outline_live(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &mut outline_entities,
                        sort_entity,
                        &sort.glyph_name,
                        position,
                        &point_query,
                        fontir_state.as_ref(),
                        &camera_scale,
                    );
                }
            } else if let Some(state) = app_state.as_ref() {
                if let Some(glyph_data) =
                    state.workspace.font.get_glyph(&sort.glyph_name)
                {
                    if let Some(outline) = &glyph_data.outline {
                        render_ufo_outline_live(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut outline_entities,
                            sort_entity,
                            &sort.glyph_name,
                            position,
                            &point_query,
                            outline,
                            &camera_scale,
                        );
                    }
                }
            }
        } else {
            // Use static rendering from FontIR/UFO data
            if let Some(fontir_state) = fontir_app_state.as_ref() {
                if let Some(paths) =
                    fontir_state.get_glyph_paths_with_edits(&sort.glyph_name)
                {
                    render_fontir_outline(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &mut outline_entities,
                        &mut entity_pools,
                        sort_entity,
                        &paths,
                        position,
                        &camera_scale,
                        &unified_rendering_sorts,
                    );
                }
            } else if let Some(state) = app_state.as_ref() {
                if let Some(glyph_data) =
                    state.workspace.font.get_glyph(&sort.glyph_name)
                {
                    if let Some(outline) = &glyph_data.outline {
                        render_ufo_outline(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut outline_entities,
                            sort_entity,
                            outline,
                            position,
                            &camera_scale,
                        );
                    }
                }
            }
        }
    }

    // Render buffer sorts (text editor sorts) with FILLED rendering for performance
    if buffer_sort_count > 0 {
        info!(
            "Rendering {} buffer sorts with filled glyphs",
            buffer_sort_count
        );
    }
    for (sort_entity, sort, transform) in buffer_sort_query.iter() {
        // Check if entity still exists before processing
        if commands.get_entity(sort_entity).is_err() {
            debug!(
                "Skipping buffer sort rendering for non-existent entity {:?}",
                sort_entity
            );
            continue;
        }

        let position = transform.translation.truncate();

        // For text buffer sorts, use filled rendering (like text editors)
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            if let Some(paths) =
                fontir_state.get_glyph_paths_with_edits(&sort.glyph_name)
            {
                render_fontir_filled_outline(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut outline_entities,
                    &mut entity_pools,
                    &mut mesh_cache,
                    sort_entity,
                    &sort.glyph_name,
                    &paths,
                    position,
                    &camera_scale,
                );
            }
        }
    }

    // Render inactive sorts (non-buffer sorts) with FILLED rendering for visibility
    for (sort_entity, sort, transform) in changed_inactive_sorts {
        // Check if entity still exists before processing
        if commands.get_entity(sort_entity).is_err() {
            debug!(
                "Skipping inactive sort rendering for non-existent entity {:?}",
                sort_entity
            );
            continue;
        }

        let position = transform.translation.truncate();

        // For inactive freeform sorts, use filled rendering for visibility
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            if let Some(paths) =
                fontir_state.get_glyph_paths_with_edits(&sort.glyph_name)
            {
                info!("GLYPH OUTLINE DEBUG: Calling render_fontir_filled_outline for inactive sort {:?} (glyph: {})", sort_entity, sort.glyph_name);
                render_fontir_filled_outline(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut outline_entities,
                    &mut entity_pools,
                    &mut mesh_cache,
                    sort_entity,
                    &sort.glyph_name,
                    &paths,
                    position,
                    &camera_scale,
                );
            }
        }
    }

    // All sort types now have appropriate rendering
}

/// Create a line mesh between two points (coordinates relative to midpoint)
pub fn create_line_mesh(start: Vec2, end: Vec2, width: f32) -> Mesh {
    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;
    let midpoint = (start + end) * 0.5;

    // Make coordinates relative to midpoint
    let start_rel = start - midpoint;
    let end_rel = end - midpoint;

    let vertices = vec![
        [
            start_rel.x - perpendicular.x,
            start_rel.y - perpendicular.y,
            0.0,
        ], // Bottom left
        [
            start_rel.x + perpendicular.x,
            start_rel.y + perpendicular.y,
            0.0,
        ], // Top left
        [
            end_rel.x + perpendicular.x,
            end_rel.y + perpendicular.y,
            0.0,
        ], // Top right
        [
            end_rel.x - perpendicular.x,
            end_rel.y - perpendicular.y,
            0.0,
        ], // Bottom right
    ];

    let indices = vec![0, 1, 2, 0, 2, 3]; // Two triangles forming a rectangle
    let uvs = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

/// Convert multiple kurbo::BezPath contours to a single filled mesh using Lyon tessellation
/// This combines all contours of a glyph for proper winding rule application
pub fn create_filled_mesh_from_glyph_paths(
    bez_paths: &[kurbo::BezPath],
) -> Mesh {
    // Convert all kurbo::BezPaths to a single lyon::Path
    let mut lyon_path_builder = Path::builder();

    for bez_path in bez_paths {
        for element in bez_path.elements() {
            match element {
                kurbo::PathEl::MoveTo(pt) => {
                    lyon_path_builder.begin(point(pt.x as f32, pt.y as f32));
                }
                kurbo::PathEl::LineTo(pt) => {
                    lyon_path_builder.line_to(point(pt.x as f32, pt.y as f32));
                }
                kurbo::PathEl::CurveTo(c1, c2, pt) => {
                    lyon_path_builder.cubic_bezier_to(
                        point(c1.x as f32, c1.y as f32),
                        point(c2.x as f32, c2.y as f32),
                        point(pt.x as f32, pt.y as f32),
                    );
                }
                kurbo::PathEl::QuadTo(c, pt) => {
                    lyon_path_builder.quadratic_bezier_to(
                        point(c.x as f32, c.y as f32),
                        point(pt.x as f32, pt.y as f32),
                    );
                }
                kurbo::PathEl::ClosePath => {
                    lyon_path_builder.close();
                }
            }
        }
    }

    let lyon_path = lyon_path_builder.build();

    // Tessellate to triangles with proper winding rule for font rendering
    let mut tessellator = FillTessellator::new();
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();

    // Use non-zero winding rule (standard for font rendering)
    let fill_options = FillOptions::default().with_fill_rule(FillRule::NonZero);

    let result = tessellator.tessellate_path(
        &lyon_path,
        &fill_options,
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            [vertex.position().x, vertex.position().y, 0.0]
        }),
    );

    if result.is_err() {
        warn!("Failed to tessellate path for filled glyph rendering");
        // Return empty mesh on tessellation failure
        return Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
    }

    // Convert to Bevy mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    // Get vertex count and create UVs before moving buffers.vertices
    let vertex_count = buffers.vertices.len();
    let uvs = buffers
        .vertices
        .iter()
        .map(|v| [v[0], v[1]])
        .collect::<Vec<_>>();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffers.vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(buffers.indices));

    // Generate normals
    let normals = vec![[0.0, 0.0, 1.0]; vertex_count];

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

/// Render UFO outline using meshes
fn render_ufo_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    sort_entity: Entity,
    outline: &OutlineData,
    position: Vec2,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let mut path_entities = Vec::new();
    let mut handle_entities = Vec::new();

    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }

        // Render path segments
        render_contour_path_meshes(
            commands,
            meshes,
            materials,
            &mut path_entities,
            sort_entity,
            contour,
            position,
            camera_scale,
        );

        // Render control handles
        render_contour_handles_meshes(
            commands,
            meshes,
            materials,
            &mut handle_entities,
            sort_entity,
            contour,
            position,
            camera_scale,
        );
    }

    outline_entities
        .path_segments
        .insert(sort_entity, path_entities);
    outline_entities
        .control_handles
        .insert(sort_entity, handle_entities);
}

/// Render FontIR outline using meshes
fn render_fontir_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    entity_pools: &mut ResMut<EntityPools>,
    sort_entity: Entity,
    paths: &[kurbo::BezPath],
    position: Vec2,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
    unified_rendering_sorts: &crate::rendering::outline_coordination::UnifiedRenderingSorts,
) {
    render_fontir_outline_with_color_and_quality(
        commands,
        meshes,
        materials,
        outline_entities,
        entity_pools,
        sort_entity,
        paths,
        position,
        PATH_STROKE_COLOR,
        camera_scale,
        false, // inactive/buffer sorts use lower quality
        unified_rendering_sorts,
    );
}

#[allow(dead_code)]
fn render_fontir_outline_with_color(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    entity_pools: &mut ResMut<EntityPools>,
    sort_entity: Entity,
    paths: &[kurbo::BezPath],
    position: Vec2,
    color: Color,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
    unified_rendering_sorts: &crate::rendering::outline_coordination::UnifiedRenderingSorts,
) {
    render_fontir_outline_with_color_and_quality(
        commands,
        meshes,
        materials,
        outline_entities,
        entity_pools,
        sort_entity,
        paths,
        position,
        color,
        camera_scale,
        true, // active sorts use high quality
        unified_rendering_sorts,
    );
}

fn render_fontir_outline_with_color_and_quality(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    entity_pools: &mut ResMut<EntityPools>,
    sort_entity: Entity,
    paths: &[kurbo::BezPath],
    position: Vec2,
    color: Color,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
    high_quality: bool,
    unified_rendering_sorts: &crate::rendering::outline_coordination::UnifiedRenderingSorts,
) {
    let _path_material = materials.add(ColorMaterial::from(color));
    let mut segment_entities = Vec::new();
    let line_width = camera_scale.adjusted_line_width();

    // Process each path (contour)
    for path in paths {
        let elements: Vec<_> = path.elements().iter().collect();
        let mut current_pos = None;

        for element in elements {
            match element {
                kurbo::PathEl::MoveTo(pt) => {
                    current_pos =
                        Some(Vec2::new(pt.x as f32, pt.y as f32) + position);
                }
                kurbo::PathEl::LineTo(pt) => {
                    if let Some(start) = current_pos {
                        let end =
                            Vec2::new(pt.x as f32, pt.y as f32) + position;
                        let entity = get_or_update_line_mesh(
                            commands,
                            meshes,
                            materials,
                            entity_pools,
                            start,
                            end,
                            line_width,
                            color,
                            GLYPH_OUTLINE_Z,
                            sort_entity,
                            OutlineElementType::PathSegment,
                        );
                        segment_entities.push(entity);
                        current_pos = Some(end);
                    }
                }
                kurbo::PathEl::CurveTo(c1, c2, pt) => {
                    // CRITICAL: Check coordination resource before rendering curves
                    if unified_rendering_sorts.contains(sort_entity) {
                        debug!("Mesh system: Skipping CurveTo for sort {:?} - handled by unified system", sort_entity);
                        if let kurbo::PathEl::CurveTo(_, _, pt) = element {
                            current_pos = Some(
                                Vec2::new(pt.x as f32, pt.y as f32) + position,
                            );
                        }
                        continue;
                    }

                    if let Some(start) = current_pos {
                        let end =
                            Vec2::new(pt.x as f32, pt.y as f32) + position;
                        // Approximate cubic curve with multiple line segments for smooth appearance
                        // PERFORMANCE: Use lower quality for inactive sorts
                        let segments = if high_quality { 32 } else { 8 };
                        debug!("Mesh system: Rendering CurveTo for sort {:?} with {} segments", sort_entity, segments);
                        let mut last_pos = start;

                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let mt = 1.0 - t;
                            let mt2 = mt * mt;
                            let mt3 = mt2 * mt;

                            // Cubic Bezier formula
                            let c1_pos =
                                Vec2::new(c1.x as f32, c1.y as f32) + position;
                            let c2_pos =
                                Vec2::new(c2.x as f32, c2.y as f32) + position;
                            let curve_pos = Vec2::new(
                                mt3 * start.x
                                    + 3.0 * mt2 * t * c1_pos.x
                                    + 3.0 * mt * t2 * c2_pos.x
                                    + t3 * end.x,
                                mt3 * start.y
                                    + 3.0 * mt2 * t * c1_pos.y
                                    + 3.0 * mt * t2 * c2_pos.y
                                    + t3 * end.y,
                            );

                            let entity = spawn_line_mesh(
                                commands,
                                meshes,
                                materials,
                                last_pos,
                                curve_pos,
                                line_width,
                                color,
                                GLYPH_OUTLINE_Z,
                                sort_entity,
                                OutlineElementType::PathSegment,
                            );
                            segment_entities.push(entity);
                            last_pos = curve_pos;
                        }
                        current_pos = Some(end);
                    }
                }
                kurbo::PathEl::QuadTo(c, pt) => {
                    // CRITICAL: Check coordination resource before rendering curves
                    if unified_rendering_sorts.contains(sort_entity) {
                        debug!("Mesh system: Skipping QuadTo for sort {:?} - handled by unified system", sort_entity);
                        if let kurbo::PathEl::QuadTo(_, pt) = element {
                            current_pos = Some(
                                Vec2::new(pt.x as f32, pt.y as f32) + position,
                            );
                        }
                        continue;
                    }

                    if let Some(start) = current_pos {
                        let end =
                            Vec2::new(pt.x as f32, pt.y as f32) + position;
                        // Approximate quadratic curve with line segments for smooth appearance
                        let segments = 24; // Increased for smoother curves
                        let mut last_pos = start;

                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let mt = 1.0 - t;

                            // Quadratic Bezier formula
                            let c_pos =
                                Vec2::new(c.x as f32, c.y as f32) + position;
                            let curve_pos = Vec2::new(
                                mt * mt * start.x
                                    + 2.0 * mt * t * c_pos.x
                                    + t * t * end.x,
                                mt * mt * start.y
                                    + 2.0 * mt * t * c_pos.y
                                    + t * t * end.y,
                            );

                            let entity = spawn_line_mesh(
                                commands,
                                meshes,
                                materials,
                                last_pos,
                                curve_pos,
                                line_width,
                                color,
                                GLYPH_OUTLINE_Z,
                                sort_entity,
                                OutlineElementType::PathSegment,
                            );
                            segment_entities.push(entity);
                            last_pos = curve_pos;
                        }
                        current_pos = Some(end);
                    }
                }
                kurbo::PathEl::ClosePath => {
                    // ClosePath doesn't add geometry - handled by path structure
                }
            }
        }
    }

    // Store entities for cleanup
    outline_entities
        .path_segments
        .insert(sort_entity, segment_entities);
}

/// Render FontIR outline using live Transform positions during nudging
fn render_fontir_outline_live(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    sort_entity: Entity,
    glyph_name: &str,
    sort_position: Vec2,
    point_query: &Query<
        (
            Entity,
            &Transform,
            &crate::editing::selection::components::GlyphPointReference,
            &crate::editing::selection::components::PointType,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    fontir_state: &crate::core::state::FontIRAppState,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    // Build a map of live positions from Transform components
    let mut live_positions = std::collections::HashMap::new();
    for (_entity, transform, point_ref, point_type) in point_query.iter() {
        if point_ref.glyph_name == glyph_name {
            let world_pos = transform.translation.truncate();
            let relative_pos = world_pos - sort_position;
            live_positions.insert(
                (point_ref.contour_index, point_ref.point_index),
                (relative_pos, point_type.is_on_curve),
            );
        }
    }

    // Get original paths to understand structure
    if let Some(original_paths) =
        fontir_state.get_glyph_paths_with_edits(glyph_name)
    {
        let mut segment_entities = Vec::new();
        let line_width = camera_scale.adjusted_line_width();

        // Process each contour
        for (contour_idx, original_path) in original_paths.iter().enumerate() {
            let elements: Vec<_> = original_path.elements().iter().collect();
            let mut element_point_index = 0;
            let mut current_pos = None;

            for element in elements {
                match element {
                    kurbo::PathEl::MoveTo(_) => {
                        // Get live position if available, otherwise use original
                        if let Some((live_pos, _)) = live_positions
                            .get(&(contour_idx, element_point_index))
                        {
                            current_pos = Some(*live_pos + sort_position);
                        } else if let kurbo::PathEl::MoveTo(pt) = element {
                            current_pos = Some(
                                Vec2::new(pt.x as f32, pt.y as f32)
                                    + sort_position,
                            );
                        }
                        element_point_index += 1;
                    }
                    kurbo::PathEl::LineTo(_) => {
                        if let Some(start) = current_pos {
                            // Get live end position
                            let end = if let Some((live_pos, _)) =
                                live_positions
                                    .get(&(contour_idx, element_point_index))
                            {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::LineTo(pt) = element {
                                Vec2::new(pt.x as f32, pt.y as f32)
                                    + sort_position
                            } else {
                                start // fallback
                            };

                            let entity = spawn_line_mesh(
                                commands,
                                meshes,
                                materials,
                                start,
                                end,
                                line_width,
                                PATH_STROKE_COLOR,
                                GLYPH_OUTLINE_Z,
                                sort_entity,
                                OutlineElementType::PathSegment,
                            );
                            segment_entities.push(entity);
                            current_pos = Some(end);
                        }
                        element_point_index += 1;
                    }
                    kurbo::PathEl::CurveTo(_, _, _) => {
                        if let Some(start) = current_pos {
                            // Get live control points and end point
                            let cp1 = if let Some((live_pos, _)) =
                                live_positions
                                    .get(&(contour_idx, element_point_index))
                            {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::CurveTo(c1, _, _) =
                                element
                            {
                                Vec2::new(c1.x as f32, c1.y as f32)
                                    + sort_position
                            } else {
                                start
                            };

                            let cp2 = if let Some((live_pos, _)) =
                                live_positions.get(&(
                                    contour_idx,
                                    element_point_index + 1,
                                )) {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::CurveTo(_, c2, _) =
                                element
                            {
                                Vec2::new(c2.x as f32, c2.y as f32)
                                    + sort_position
                            } else {
                                start
                            };

                            let end = if let Some((live_pos, _)) =
                                live_positions.get(&(
                                    contour_idx,
                                    element_point_index + 2,
                                )) {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::CurveTo(_, _, pt) =
                                element
                            {
                                Vec2::new(pt.x as f32, pt.y as f32)
                                    + sort_position
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

                                let entity = spawn_line_mesh(
                                    commands,
                                    meshes,
                                    materials,
                                    last_pos,
                                    curve_pos,
                                    line_width,
                                    PATH_STROKE_COLOR,
                                    GLYPH_OUTLINE_Z,
                                    sort_entity,
                                    OutlineElementType::PathSegment,
                                );
                                segment_entities.push(entity);
                                last_pos = curve_pos;
                            }
                            current_pos = Some(end);
                        }
                        element_point_index += 3; // CurveTo has 3 points
                    }
                    kurbo::PathEl::QuadTo(_, _) => {
                        if let Some(start) = current_pos {
                            // Similar to CurveTo but with only one control point
                            let cp = if let Some((live_pos, _)) = live_positions
                                .get(&(contour_idx, element_point_index))
                            {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::QuadTo(c, _) = element
                            {
                                Vec2::new(c.x as f32, c.y as f32)
                                    + sort_position
                            } else {
                                start
                            };

                            let end = if let Some((live_pos, _)) =
                                live_positions.get(&(
                                    contour_idx,
                                    element_point_index + 1,
                                )) {
                                *live_pos + sort_position
                            } else if let kurbo::PathEl::QuadTo(_, pt) = element
                            {
                                Vec2::new(pt.x as f32, pt.y as f32)
                                    + sort_position
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

                                let entity = spawn_line_mesh(
                                    commands,
                                    meshes,
                                    materials,
                                    last_pos,
                                    curve_pos,
                                    line_width,
                                    PATH_STROKE_COLOR,
                                    GLYPH_OUTLINE_Z,
                                    sort_entity,
                                    OutlineElementType::PathSegment,
                                );
                                segment_entities.push(entity);
                                last_pos = curve_pos;
                            }
                            current_pos = Some(end);
                        }
                        element_point_index += 2; // QuadTo has 2 points
                    }
                    kurbo::PathEl::ClosePath => {
                        // ClosePath doesn't add geometry
                    }
                }
            }
        }

        // Store entities for cleanup
        outline_entities
            .path_segments
            .insert(sort_entity, segment_entities);
    }
}

/// Render FontIR outline as filled shapes (for inactive text sorts)
/// This provides text editor performance for displaying inactive characters
fn render_fontir_filled_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    entity_pools: &mut ResMut<EntityPools>,
    mesh_cache: &mut ResMut<GlyphMeshCache>,
    sort_entity: Entity,
    glyph_name: &str,
    paths: &[kurbo::BezPath],
    position: Vec2,
    _camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    info!("FILLED RENDER DEBUG: Starting render_fontir_filled_outline for sort {:?} glyph '{}' with {} paths at {:?}", 
          sort_entity, glyph_name, paths.len(), position);

    if paths.is_empty() {
        warn!("FILLED RENDER DEBUG: No paths for glyph '{}' - skipping filled rendering", glyph_name);
        return;
    }

    let fill_material = materials.add(ColorMaterial::from(FILLED_GLYPH_COLOR));
    info!(
        "FILLED RENDER DEBUG: Created fill material for sort {:?}",
        sort_entity
    );

    // PERFORMANCE OPTIMIZATION: Try to get cached mesh first
    let mesh_handle =
        if let Some(cached_mesh) = mesh_cache.get_filled_mesh(glyph_name) {
            info!(
                "FILLED RENDER DEBUG: Using cached mesh for glyph '{}'",
                glyph_name
            );
            // Use cached mesh - avoid expensive tessellation
            cached_mesh
        } else {
            // Create single filled mesh for entire glyph (all contours combined)
            // This allows proper winding rule application for counters/holes
            let filled_mesh = create_filled_mesh_from_glyph_paths(paths);
            let mesh_handle = meshes.add(filled_mesh);

            // Cache the mesh for future use
            mesh_cache
                .cache_filled_mesh(glyph_name.to_string(), mesh_handle.clone());
            mesh_handle
        };

    // Get filled entity from pool instead of spawning
    let entity = entity_pools.get_outline_entity(commands, sort_entity);
    info!(
        "FILLED RENDER DEBUG: Got entity {:?} from pool for sort {:?}",
        entity, sort_entity
    );

    // Update the pooled entity with filled mesh components using the safe helper
    let outline_component = GlyphOutlineElement {
        element_type: OutlineElementType::FilledShape,
        sort_entity,
    };

    // Use the helper function which has entity existence checks
    update_outline_entity(
        commands,
        entity,
        mesh_handle,
        fill_material,
        Transform::from_xyz(position.x, position.y, FILLED_GLYPH_Z),
        outline_component,
    );
    info!("FILLED RENDER DEBUG: Updated entity {:?} with filled mesh at position {:?}", entity, position);

    debug!(
        "Created filled glyph entity {:?} for sort {:?} with {} contours",
        entity,
        sort_entity,
        paths.len()
    );

    // Store single entity for cleanup
    outline_entities
        .path_segments
        .insert(sort_entity, vec![entity]);
}

/// Render UFO outline using live Transform positions during nudging
fn render_ufo_outline_live(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    sort_entity: Entity,
    glyph_name: &str,
    sort_position: Vec2,
    point_query: &Query<
        (
            Entity,
            &Transform,
            &crate::editing::selection::components::GlyphPointReference,
            &crate::editing::selection::components::PointType,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    outline: &crate::core::state::OutlineData,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    // Build a map of live positions from Transform components
    let mut live_positions = std::collections::HashMap::new();
    for (_entity, transform, point_ref, _point_type) in point_query.iter() {
        if point_ref.glyph_name == glyph_name {
            let world_pos = transform.translation.truncate();
            let relative_pos = world_pos - sort_position;
            live_positions.insert(
                (point_ref.contour_index, point_ref.point_index),
                relative_pos,
            );
        }
    }

    let mut path_entities = Vec::new();
    let mut handle_entities = Vec::new();

    for (contour_idx, contour) in outline.contours.iter().enumerate() {
        if contour.points.is_empty() {
            continue;
        }

        // Render path segments using live positions
        render_contour_path_meshes_live(
            commands,
            meshes,
            materials,
            &mut path_entities,
            sort_entity,
            contour,
            sort_position,
            &live_positions,
            contour_idx,
            camera_scale,
        );

        // Render control handles using live positions
        render_contour_handles_meshes_live(
            commands,
            meshes,
            materials,
            &mut handle_entities,
            sort_entity,
            contour,
            sort_position,
            &live_positions,
            contour_idx,
            camera_scale,
        );
    }

    outline_entities
        .path_segments
        .insert(sort_entity, path_entities);
    outline_entities
        .control_handles
        .insert(sort_entity, handle_entities);
}

/// Render contour path segments as meshes
fn render_contour_path_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    path_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    contour: &ContourData,
    offset: Vec2,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Convert contour to line segments (similar to glyph_outline.rs logic)
    let segments = extract_path_segments(contour);
    let line_width = camera_scale.adjusted_line_width();

    for segment in segments {
        match segment {
            PathSegment::Line { start, end } => {
                let start_pos =
                    Vec2::new(start.x as f32, start.y as f32) + offset;
                let end_pos = Vec2::new(end.x as f32, end.y as f32) + offset;

                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    start_pos,
                    end_pos,
                    line_width,
                    PATH_STROKE_COLOR,
                    GLYPH_OUTLINE_Z,
                    sort_entity,
                    OutlineElementType::PathSegment,
                );
                path_entities.push(entity);
            }
            PathSegment::Curve {
                start,
                control1,
                control2,
                end,
            } => {
                // Tessellate curve into multiple line segments
                let start_pos =
                    Vec2::new(start.x as f32, start.y as f32) + offset;
                let cp1_pos =
                    Vec2::new(control1.x as f32, control1.y as f32) + offset;
                let cp2_pos =
                    Vec2::new(control2.x as f32, control2.y as f32) + offset;
                let end_pos = Vec2::new(end.x as f32, end.y as f32) + offset;

                let curve_points = tessellate_cubic_bezier(
                    start_pos, cp1_pos, cp2_pos, end_pos, 20,
                );

                // Create line segments for tessellated curve
                for i in 0..curve_points.len() - 1 {
                    let entity = spawn_line_mesh(
                        commands,
                        meshes,
                        materials,
                        curve_points[i],
                        curve_points[i + 1],
                        line_width,
                        PATH_STROKE_COLOR,
                        GLYPH_OUTLINE_Z,
                        sort_entity,
                        OutlineElementType::PathSegment,
                    );
                    path_entities.push(entity);
                }
            }
        }
    }
}

/// Render contour control handles as meshes
fn render_contour_handles_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    handle_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    contour: &ContourData,
    offset: Vec2,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Find control handle lines (similar to existing logic)
    let handle_line_width = camera_scale.adjusted_line_width();
    for (i, point) in points.iter().enumerate() {
        if !is_on_curve(point) {
            // Find nearest on-curve points to draw handles to
            let prev_on_curve = find_previous_on_curve(points, i);
            let next_on_curve = find_next_on_curve(points, i);

            let handle_pos = Vec2::new(point.x as f32, point.y as f32) + offset;

            if let Some(prev_idx) = prev_on_curve {
                let prev_pos = Vec2::new(
                    points[prev_idx].x as f32,
                    points[prev_idx].y as f32,
                ) + offset;
                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    prev_pos,
                    handle_pos,
                    handle_line_width,
                    HANDLE_LINE_COLOR,
                    CONTROL_HANDLE_Z,
                    sort_entity,
                    OutlineElementType::ControlHandle,
                );
                handle_entities.push(entity);
            }

            if let Some(next_idx) = next_on_curve {
                let next_pos = Vec2::new(
                    points[next_idx].x as f32,
                    points[next_idx].y as f32,
                ) + offset;
                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    handle_pos,
                    next_pos,
                    handle_line_width,
                    HANDLE_LINE_COLOR,
                    CONTROL_HANDLE_Z,
                    sort_entity,
                    OutlineElementType::ControlHandle,
                );
                handle_entities.push(entity);
            }
        }
    }
}

/// Helper to spawn a line mesh entity
fn spawn_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: Color,
    z: f32,
    sort_entity: Entity,
    element_type: OutlineElementType,
) -> Entity {
    let line_mesh = create_line_mesh(start, end, width);

    commands
        .spawn((
            GlyphOutlineElement {
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

/// ENTITY POOLING: Get or update a line mesh entity from the pool
fn get_or_update_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    entity_pools: &mut ResMut<EntityPools>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: Color,
    z: f32,
    sort_entity: Entity,
    element_type: OutlineElementType,
) -> Entity {
    // Get an entity from the pool
    let entity = entity_pools.get_outline_entity(commands, sort_entity);

    // Create the mesh and material
    let line_mesh = create_line_mesh(start, end, width);
    let mesh_handle = meshes.add(line_mesh);
    let material_handle = materials.add(ColorMaterial::from_color(color));

    // Calculate transform
    let transform = Transform::from_xyz(
        (start.x + end.x) * 0.5,
        (start.y + end.y) * 0.5,
        z,
    );

    // Update the entity using the helper function
    let outline_component = GlyphOutlineElement {
        element_type,
        sort_entity,
    };

    update_outline_entity(
        commands,
        entity,
        mesh_handle,
        material_handle,
        transform,
        outline_component,
    );

    debug!(
        "Updated pooled outline entity {:?} for sort {:?}",
        entity, sort_entity
    );
    entity
}

/// Tessellate a cubic bezier curve into line segments with adaptive subdivision
pub fn tessellate_cubic_bezier(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    segments: u32,
) -> Vec<Vec2> {
    let mut points = Vec::with_capacity((segments + 1) as usize);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;

        // Cubic bezier formula: (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let one_minus_t_cu = one_minus_t_sq * one_minus_t;
        let t_sq = t * t;
        let t_cu = t_sq * t;

        let point = p0 * one_minus_t_cu
            + p1 * (3.0 * one_minus_t_sq * t)
            + p2 * (3.0 * one_minus_t * t_sq)
            + p3 * t_cu;

        points.push(point);
    }

    points
}

/// Path segment types for efficient rendering
#[derive(Debug)]
#[allow(dead_code)]
enum PathSegment {
    Line {
        start: PointData,
        end: PointData,
    },
    Curve {
        start: PointData,
        control1: PointData,
        control2: PointData,
        end: PointData,
    },
}

/// Extract path segments from contour (similar to glyph_outline.rs logic)
fn extract_path_segments(contour: &ContourData) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let points = &contour.points;

    if points.is_empty() {
        return segments;
    }

    // Simplified segment extraction (can be optimized further)
    for i in 0..points.len() {
        let current = &points[i];
        let next = &points[(i + 1) % points.len()];

        if is_on_curve(current) && is_on_curve(next) {
            segments.push(PathSegment::Line {
                start: current.clone(),
                end: next.clone(),
            });
        }
        // TODO: Add proper curve segment detection
    }

    segments
}

/// Helper functions (from glyph_outline.rs)
fn is_on_curve(point: &PointData) -> bool {
    matches!(
        point.point_type,
        PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve
    )
}

fn find_previous_on_curve(
    points: &[PointData],
    start_idx: usize,
) -> Option<usize> {
    for i in 1..points.len() {
        let idx = (start_idx + points.len() - i) % points.len();
        if is_on_curve(&points[idx]) {
            return Some(idx);
        }
    }
    None
}

fn find_next_on_curve(points: &[PointData], start_idx: usize) -> Option<usize> {
    for i in 1..points.len() {
        let idx = (start_idx + i) % points.len();
        if is_on_curve(&points[idx]) {
            return Some(idx);
        }
    }
    None
}

/// Plugin for mesh-based glyph outline rendering
pub struct MeshGlyphOutlinePlugin;

/// Render contour path segments using live Transform positions
fn render_contour_path_meshes_live(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    path_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    contour: &ContourData,
    sort_position: Vec2,
    live_positions: &std::collections::HashMap<(usize, usize), Vec2>,
    contour_idx: usize,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Extract path segments using live positions
    let segments =
        extract_path_segments_live(contour, live_positions, contour_idx);
    let line_width = camera_scale.adjusted_line_width();

    for segment in segments {
        match segment {
            PathSegmentLive::Line { start, end } => {
                let start_pos = start + sort_position;
                let end_pos = end + sort_position;

                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    start_pos,
                    end_pos,
                    line_width,
                    PATH_STROKE_COLOR,
                    GLYPH_OUTLINE_Z,
                    sort_entity,
                    OutlineElementType::PathSegment,
                );
                path_entities.push(entity);
            }
            PathSegmentLive::Curve {
                start,
                control1,
                control2,
                end,
            } => {
                // Tessellate curve into multiple line segments
                let start_pos = start + sort_position;
                let cp1_pos = control1 + sort_position;
                let cp2_pos = control2 + sort_position;
                let end_pos = end + sort_position;

                let curve_points = tessellate_cubic_bezier(
                    start_pos, cp1_pos, cp2_pos, end_pos, 20,
                );

                // Create line segments for tessellated curve
                for i in 0..curve_points.len() - 1 {
                    let entity = spawn_line_mesh(
                        commands,
                        meshes,
                        materials,
                        curve_points[i],
                        curve_points[i + 1],
                        line_width,
                        PATH_STROKE_COLOR,
                        GLYPH_OUTLINE_Z,
                        sort_entity,
                        OutlineElementType::PathSegment,
                    );
                    path_entities.push(entity);
                }
            }
        }
    }
}

/// Render contour handles using live Transform positions
fn render_contour_handles_meshes_live(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    handle_entities: &mut Vec<Entity>,
    sort_entity: Entity,
    contour: &ContourData,
    sort_position: Vec2,
    live_positions: &std::collections::HashMap<(usize, usize), Vec2>,
    contour_idx: usize,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Find control handle lines using live positions
    let handle_line_width = camera_scale.adjusted_line_width();
    for (i, point) in points.iter().enumerate() {
        if !is_on_curve(point) {
            // Get live position for this off-curve point
            let handle_pos =
                if let Some(live_pos) = live_positions.get(&(contour_idx, i)) {
                    *live_pos + sort_position
                } else {
                    Vec2::new(point.x as f32, point.y as f32) + sort_position
                };

            // Find nearest on-curve points to draw handles to
            let prev_on_curve = find_previous_on_curve(points, i);
            let next_on_curve = find_next_on_curve(points, i);

            if let Some(prev_idx) = prev_on_curve {
                let prev_pos = if let Some(live_pos) =
                    live_positions.get(&(contour_idx, prev_idx))
                {
                    *live_pos + sort_position
                } else {
                    Vec2::new(
                        points[prev_idx].x as f32,
                        points[prev_idx].y as f32,
                    ) + sort_position
                };
                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    prev_pos,
                    handle_pos,
                    handle_line_width,
                    HANDLE_LINE_COLOR,
                    CONTROL_HANDLE_Z,
                    sort_entity,
                    OutlineElementType::ControlHandle,
                );
                handle_entities.push(entity);
            }

            if let Some(next_idx) = next_on_curve {
                let next_pos = if let Some(live_pos) =
                    live_positions.get(&(contour_idx, next_idx))
                {
                    *live_pos + sort_position
                } else {
                    Vec2::new(
                        points[next_idx].x as f32,
                        points[next_idx].y as f32,
                    ) + sort_position
                };
                let entity = spawn_line_mesh(
                    commands,
                    meshes,
                    materials,
                    handle_pos,
                    next_pos,
                    handle_line_width,
                    HANDLE_LINE_COLOR,
                    CONTROL_HANDLE_Z,
                    sort_entity,
                    OutlineElementType::ControlHandle,
                );
                handle_entities.push(entity);
            }
        }
    }
}

/// Path segment types for live rendering with Vec2 positions
#[derive(Debug)]
#[allow(dead_code)]
enum PathSegmentLive {
    Line {
        start: Vec2,
        end: Vec2,
    },
    Curve {
        start: Vec2,
        control1: Vec2,
        control2: Vec2,
        end: Vec2,
    },
}

/// Extract path segments from contour using live positions
fn extract_path_segments_live(
    contour: &ContourData,
    live_positions: &std::collections::HashMap<(usize, usize), Vec2>,
    contour_idx: usize,
) -> Vec<PathSegmentLive> {
    let mut segments = Vec::new();
    let points = &contour.points;

    if points.is_empty() {
        return segments;
    }

    // Simplified segment extraction using live positions
    for i in 0..points.len() {
        let current = &points[i];
        let next = &points[(i + 1) % points.len()];

        // Get live positions or fall back to original
        let current_pos = live_positions
            .get(&(contour_idx, i))
            .copied()
            .unwrap_or_else(|| Vec2::new(current.x as f32, current.y as f32));
        let next_pos = live_positions
            .get(&(contour_idx, (i + 1) % points.len()))
            .copied()
            .unwrap_or_else(|| Vec2::new(next.x as f32, next.y as f32));

        if is_on_curve(current) && is_on_curve(next) {
            segments.push(PathSegmentLive::Line {
                start: current_pos,
                end: next_pos,
            });
        }
        // TODO: Add proper curve segment detection using live positions
    }

    segments
}

/// System to clean up filled mesh entities when sorts become active
/// This prevents old filled meshes from showing on active sorts
fn cleanup_filled_meshes_on_activation(
    mut commands: Commands,
    mut outline_entities: ResMut<MeshOutlineEntities>,
    newly_active_sorts: Query<Entity, Added<ActiveSort>>,
    filled_entities_query: Query<
        (Entity, &GlyphOutlineElement),
        With<GlyphOutlineElement>,
    >,
) {
    for sort_entity in newly_active_sorts.iter() {
        debug!("CLEANUP: Sort {:?} became active - clearing any old filled mesh entities", sort_entity);

        // Find and despawn any filled mesh entities for this sort
        let mut entities_to_despawn = Vec::new();

        for (entity, outline_element) in filled_entities_query.iter() {
            if outline_element.sort_entity == sort_entity {
                match outline_element.element_type {
                    OutlineElementType::FilledShape => {
                        entities_to_despawn.push(entity);
                    }
                    _ => {} // Keep other types (line segments, handles)
                }
            }
        }

        if !entities_to_despawn.is_empty() {
            debug!("CLEANUP: Despawning {} filled mesh entities for newly active sort {:?}", entities_to_despawn.len(), sort_entity);
            for entity in entities_to_despawn {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                } else {
                    debug!(
                        "CLEANUP: Filled mesh entity {:?} already despawned",
                        entity
                    );
                }
            }

            // Also remove from tracking (filled entities are stored in path_segments for now)
            outline_entities.path_segments.remove(&sort_entity);
        }
    }
}

impl Plugin for MeshGlyphOutlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshOutlineEntities>()
           .add_systems(Update, (
               cleanup_filled_meshes_on_activation,
               render_mesh_glyph_outline
                   .after(crate::rendering::sort_visuals::handle_sort_selection_and_drag_start),
           ).chain());
    }
}
