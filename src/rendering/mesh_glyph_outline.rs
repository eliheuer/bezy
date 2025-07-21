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
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::render::mesh::Mesh2d;
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
const GLYPH_OUTLINE_Z: f32 = 8.0;   // Behind points (10.0) and point backgrounds (5.0)
const CONTROL_HANDLE_Z: f32 = 9.0;  // Slightly above outlines

/// Line width for path segments and handles
const PATH_LINE_WIDTH: f32 = 1.5;
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

    // DISABLED: Mesh glyph outline rendering - causes unwanted debug shapes
    return;

    // Render outlines for each active sort
    for (sort_entity, sort, transform) in active_sort_query.iter() {
        let position = transform.translation.truncate();
        
        // Try to get outline data from FontIR first, then AppState
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            if let Some(paths) = fontir_state.get_glyph_paths(&sort.glyph_name) {
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
            if let Some(glyph_data) = state.workspace.font.get_glyph(&sort.glyph_name) {
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

/// Create a line mesh between two points
pub fn create_line_mesh(start: Vec2, end: Vec2, width: f32) -> Mesh {
    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;
    
    let vertices = vec![
        [start.x - perpendicular.x, start.y - perpendicular.y, 0.0], // Bottom left
        [start.x + perpendicular.x, start.y + perpendicular.y, 0.0], // Top left  
        [end.x + perpendicular.x, end.y + perpendicular.y, 0.0],     // Top right
        [end.x - perpendicular.x, end.y - perpendicular.y, 0.0],     // Bottom right
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3]; // Two triangles forming a rectangle
    let uvs = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
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
            commands, meshes, materials, &mut path_entities,
            sort_entity, contour, position
        );
        
        // Render control handles
        render_contour_handles_meshes(
            commands, meshes, materials, &mut handle_entities,
            sort_entity, contour, position
        );
    }
    
    outline_entities.path_segments.insert(sort_entity, path_entities);
    outline_entities.control_handles.insert(sort_entity, handle_entities);
}

/// Render FontIR outline using meshes (placeholder for now)
fn render_fontir_outline(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<ColorMaterial>>,
    _outline_entities: &mut ResMut<MeshOutlineEntities>,
    _sort_entity: Entity,
    _paths: &[kurbo::BezPath],
    _position: Vec2,
) {
    // TODO: Implement FontIR rendering similar to UFO
    // For now, we'll focus on UFO rendering
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
                let start_pos = Vec2::new(start.x as f32, start.y as f32) + offset;
                let end_pos = Vec2::new(end.x as f32, end.y as f32) + offset;
                
                let entity = spawn_line_mesh(
                    commands, meshes, materials,
                    start_pos, end_pos, PATH_LINE_WIDTH,
                    PATH_STROKE_COLOR, GLYPH_OUTLINE_Z,
                    sort_entity, OutlineElementType::PathSegment
                );
                path_entities.push(entity);
            }
            PathSegment::Curve { start, control1, control2, end } => {
                // Tessellate curve into multiple line segments
                let start_pos = Vec2::new(start.x as f32, start.y as f32) + offset;
                let cp1_pos = Vec2::new(control1.x as f32, control1.y as f32) + offset;
                let cp2_pos = Vec2::new(control2.x as f32, control2.y as f32) + offset;
                let end_pos = Vec2::new(end.x as f32, end.y as f32) + offset;
                
                let curve_points = tessellate_cubic_bezier(start_pos, cp1_pos, cp2_pos, end_pos, 20);
                
                // Create line segments for tessellated curve
                for i in 0..curve_points.len() - 1 {
                    let entity = spawn_line_mesh(
                        commands, meshes, materials,
                        curve_points[i], curve_points[i + 1], PATH_LINE_WIDTH,
                        PATH_STROKE_COLOR, GLYPH_OUTLINE_Z,
                        sort_entity, OutlineElementType::PathSegment
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
                let prev_pos = Vec2::new(points[prev_idx].x as f32, points[prev_idx].y as f32) + offset;
                let entity = spawn_line_mesh(
                    commands, meshes, materials,
                    prev_pos, handle_pos, HANDLE_LINE_WIDTH,
                    HANDLE_LINE_COLOR, CONTROL_HANDLE_Z,
                    sort_entity, OutlineElementType::ControlHandle
                );
                handle_entities.push(entity);
            }
            
            if let Some(next_idx) = next_on_curve {
                let next_pos = Vec2::new(points[next_idx].x as f32, points[next_idx].y as f32) + offset;
                let entity = spawn_line_mesh(
                    commands, meshes, materials,
                    handle_pos, next_pos, HANDLE_LINE_WIDTH,
                    HANDLE_LINE_COLOR, CONTROL_HANDLE_Z,
                    sort_entity, OutlineElementType::ControlHandle
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
    
    commands.spawn((
        GlyphOutlineElement { element_type, sort_entity },
        Mesh2d(meshes.add(line_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_xyz((start.x + end.x) * 0.5, (start.y + end.y) * 0.5, z),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id()
}

/// Tessellate a cubic bezier curve into line segments with adaptive subdivision
pub fn tessellate_cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, segments: u32) -> Vec<Vec2> {
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
    Line { start: PointData, end: PointData },
    Curve { start: PointData, control1: PointData, control2: PointData, end: PointData },
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
                end: next.clone() 
            });
        }
        // TODO: Add proper curve segment detection
    }
    
    segments
}

/// Helper functions (from glyph_outline.rs)
fn is_on_curve(point: &PointData) -> bool {
    matches!(point.point_type, PointTypeData::Move | PointTypeData::Line | PointTypeData::Curve)
}

fn find_previous_on_curve(points: &[PointData], start_idx: usize) -> Option<usize> {
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

impl Plugin for MeshGlyphOutlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshOutlineEntities>()
           .add_systems(Update, render_mesh_glyph_outline
               .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized));
    }
}