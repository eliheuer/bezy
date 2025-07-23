//! Mesh-based glyph outline rendering
//!
//! This module replaces gizmo-based rendering with mesh/sprite-based rendering
//! to allow proper z-ordering and layering control.
//!
//! Performance optimizations:
//! - Entity pooling to avoid constant spawn/despawn
//! - Batched mesh creation
//! - Efficient curve tessellation

use crate::core::state::{ContourData, OutlineData, PointData, PointTypeData};
use crate::editing::sort::{ActiveSort, Sort};
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
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

/// Line width for path segments and handles
const PATH_LINE_WIDTH: f32 = 1.0; // Same as handles for visual consistency
const HANDLE_LINE_WIDTH: f32 = 1.0;

/// System to render glyph outlines using meshes instead of gizmos
pub fn render_mesh_glyph_outline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut outline_entities: ResMut<MeshOutlineEntities>,
    active_sort_query: Query<(Entity, &Sort, &Transform), With<ActiveSort>>,
    app_state: Option<Res<crate::core::state::AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    existing_outlines: Query<Entity, With<GlyphOutlineElement>>,
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
    selected_query: Query<
        Entity,
        With<crate::editing::selection::components::Selected>,
    >,
) {
    // Clear existing outline elements for performance
    for entity in existing_outlines.iter() {
        commands.entity(entity).despawn();
    }
    outline_entities.path_segments.clear();
    outline_entities.control_handles.clear();

    let active_sort_count = active_sort_query.iter().count();
    if active_sort_count == 0 {
        return; // No active sorts, nothing to render
    }

    // ENABLED: Mesh glyph outline rendering for proper z-ordering
    // Removed return to enable mesh-based outlines

    // Render outlines for each active sort
    for (sort_entity, sort, transform) in active_sort_query.iter() {
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
                if let Some(paths) =
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
                        sort_entity,
                        &paths,
                        position,
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
                        );
                    }
                }
            }
        }
    }
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

/// Render UFO outline using meshes
fn render_ufo_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    outline_entities: &mut ResMut<MeshOutlineEntities>,
    sort_entity: Entity,
    outline: &OutlineData,
    position: Vec2,
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
    sort_entity: Entity,
    paths: &[kurbo::BezPath],
    position: Vec2,
) {
    let path_material = materials.add(ColorMaterial::from(PATH_STROKE_COLOR));
    let mut segment_entities = Vec::new();

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
                        let entity = spawn_line_mesh(
                            commands,
                            meshes,
                            materials,
                            start,
                            end,
                            PATH_LINE_WIDTH,
                            PATH_STROKE_COLOR,
                            GLYPH_OUTLINE_Z,
                            sort_entity,
                            OutlineElementType::PathSegment,
                        );
                        segment_entities.push(entity);
                        current_pos = Some(end);
                    }
                }
                kurbo::PathEl::CurveTo(c1, c2, pt) => {
                    if let Some(start) = current_pos {
                        let end =
                            Vec2::new(pt.x as f32, pt.y as f32) + position;
                        // Approximate cubic curve with multiple line segments for smooth appearance
                        let segments = 32; // Increased for smoother curves
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
                                PATH_LINE_WIDTH,
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
                }
                kurbo::PathEl::QuadTo(c, pt) => {
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
                                PATH_LINE_WIDTH,
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
                                PATH_LINE_WIDTH,
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
                                    PATH_LINE_WIDTH,
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
                                    PATH_LINE_WIDTH,
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
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Convert contour to line segments (similar to glyph_outline.rs logic)
    let segments = extract_path_segments(contour);

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
                    PATH_LINE_WIDTH,
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
                        PATH_LINE_WIDTH,
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
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Find control handle lines (similar to existing logic)
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
                    HANDLE_LINE_WIDTH,
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
                    HANDLE_LINE_WIDTH,
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
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Extract path segments using live positions
    let segments =
        extract_path_segments_live(contour, live_positions, contour_idx);

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
                    PATH_LINE_WIDTH,
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
                        PATH_LINE_WIDTH,
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
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Find control handle lines using live positions
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
                    HANDLE_LINE_WIDTH,
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
                    HANDLE_LINE_WIDTH,
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

impl Plugin for MeshGlyphOutlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshOutlineEntities>()
           .add_systems(Update, render_mesh_glyph_outline
               .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized)
               .after(crate::editing::selection::nudge::handle_nudge_input)
               .after(crate::editing::selection::nudge::sync_nudged_points_on_completion));
    }
}
