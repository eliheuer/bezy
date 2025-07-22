//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use crate::core::state::font_metrics::FontMetrics;
use crate::core::state::fontir_app_state::FontIRMetrics;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::ui::theme::METRICS_GUIDE_COLOR;
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::render::mesh::Mesh2d;

/// Component to mark entities as metrics line visual elements
#[derive(Component)]
pub struct MetricsLine {
    pub sort_entity: Entity,
    pub line_type: MetricsLineType,
}

/// Types of metrics line elements
#[derive(Debug, Clone)]
pub enum MetricsLineType {
    Baseline,
    XHeight,
    CapHeight,
    Ascender,
    Descender,
    AdvanceWidth,
    BoundingBox,
}

/// Resource to track metrics line entities
#[derive(Resource, Default)]
pub struct MetricsLineEntities {
    pub lines: std::collections::HashMap<Entity, Vec<Entity>>, // sort_entity -> line entities
}

/// Z-levels for metrics lines
const METRICS_LINE_Z: f32 = 5.0; // Behind glyph editing elements

/// Helper to spawn a mesh-based metrics line
fn spawn_metrics_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    sort_entity: Entity,
    line_type: MetricsLineType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    let line_width = camera_scale.adjusted_line_width(); // Camera-responsive width
    let line_mesh = crate::rendering::mesh_glyph_outline::create_line_mesh(start, end, line_width);
    
    commands.spawn((
        MetricsLine { sort_entity, line_type },
        Mesh2d(meshes.add(line_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
        Transform::from_xyz((start.x + end.x) * 0.5, (start.y + end.y) * 0.5, METRICS_LINE_Z),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )).id()
}

/// System to render mesh-based metrics lines for all active sorts
pub fn render_mesh_metrics_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut metrics_entities: ResMut<MetricsLineEntities>,
    sort_query: Query<(Entity, &Transform, &crate::editing::sort::Sort), With<crate::editing::sort::ActiveSort>>,
    existing_metrics: Query<Entity, With<MetricsLine>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clear existing metrics lines
    for entity in existing_metrics.iter() {
        commands.entity(entity).despawn();
    }
    metrics_entities.lines.clear();

    if let Some(fontir_state) = fontir_app_state {
        let fontir_metrics = fontir_state.get_font_metrics();
        
        for (sort_entity, sort_transform, sort) in sort_query.iter() {
            let position = sort_transform.translation.truncate();
            let advance_width = fontir_state.get_glyph_advance_width(&sort.glyph_name);
            // Since query filters for ActiveSort, all sorts here are active - use active color
            let color = crate::ui::theme::SORT_ACTIVE_METRICS_COLOR;
            
            let mut line_entities = Vec::new();
            
            // Extract metrics values
            let upm = fontir_metrics.units_per_em;
            let ascender = fontir_metrics.ascender.unwrap_or(upm * 0.8);
            let descender = fontir_metrics.descender.unwrap_or(upm * -0.2);
            let x_height = fontir_metrics.x_height.unwrap_or(upm * 0.5);
            let cap_height = fontir_metrics.cap_height.unwrap_or(upm * 0.7);
            
            // Baseline (most important)
            let baseline_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                position,
                Vec2::new(position.x + advance_width, position.y),
                color,
                sort_entity,
                MetricsLineType::Baseline,
                &camera_scale,
            );
            line_entities.push(baseline_entity);
            
            // x-height
            let x_height_y = position.y + x_height;
            let x_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, x_height_y),
                Vec2::new(position.x + advance_width, x_height_y),
                color,
                sort_entity,
                MetricsLineType::XHeight,
                &camera_scale,
            );
            line_entities.push(x_height_entity);
            
            // cap-height
            let cap_height_y = position.y + cap_height;
            let cap_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, cap_height_y),
                Vec2::new(position.x + advance_width, cap_height_y),
                color,
                sort_entity,
                MetricsLineType::CapHeight,
                &camera_scale,
            );
            line_entities.push(cap_height_entity);
            
            // ascender
            let ascender_y = position.y + ascender;
            let ascender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, ascender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::Ascender,
                &camera_scale,
            );
            line_entities.push(ascender_entity);
            
            // descender
            let descender_y = position.y + descender;
            let descender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, descender_y),
                Vec2::new(position.x + advance_width, descender_y),
                color,
                sort_entity,
                MetricsLineType::Descender,
                &camera_scale,
            );
            line_entities.push(descender_entity);
            
            // Advance width line (vertical)
            let advance_width_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x + advance_width, descender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::AdvanceWidth,
                &camera_scale,
            );
            line_entities.push(advance_width_entity);
            
            // Draw bounding box lines (4 lines for rectangle)
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right = Vec2::new(position.x + advance_width, descender_y);
            
            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color.with_alpha(0.7),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(top_entity);
            
            // Right line
            let right_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(bottom_right.x, top_left.y),
                bottom_right,
                color.with_alpha(0.7),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(right_entity);
            
            // Bottom line
            let bottom_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                bottom_right,
                Vec2::new(top_left.x, bottom_right.y),
                color.with_alpha(0.7),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(bottom_entity);
            
            // Left line
            let left_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(top_left.x, bottom_right.y),
                top_left,
                color.with_alpha(0.7),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);
            
            metrics_entities.lines.insert(sort_entity, line_entities);
        }
    }
}

/// Draw complete font metrics for a glyph at a specific design-space position
/// DISABLED: Gizmo-based metrics rendering - now using mesh system
pub fn draw_metrics_at_position(
    _gizmos: &mut Gizmos,
    _advance_width: f32,
    _metrics: &FontMetrics,
    _position: Vec2,
    _color: Color,
) {
    // DISABLED: All gizmo-based metrics rendering - now using mesh system
    // let upm = metrics.units_per_em;
    // let ascender = metrics.ascender.unwrap_or(upm * 0.8) as f32;
    // let descender = metrics.descender.unwrap_or(upm * -0.2) as f32;
    // let x_height = metrics.x_height.unwrap_or(upm * 0.5) as f32;
    // let cap_height = metrics.cap_height.unwrap_or(upm * 0.7) as f32;

    // // Baseline (most important)
    // gizmos.line_2d(
    //     position,
    //     Vec2::new(position.x + advance_width, position.y),
    //     color,
    // );

    // // x-height
    // let x_height_y = position.y + x_height;
    // gizmos.line_2d(
    //     Vec2::new(position.x, x_height_y),
    //     Vec2::new(position.x + advance_width, x_height_y),
    //     color,
    // );

    // // cap-height
    // let cap_height_y = position.y + cap_height;
    // gizmos.line_2d(
    //     Vec2::new(position.x, cap_height_y),
    //     Vec2::new(position.x + advance_width, cap_height_y),
    //     color,
    // );

    // // ascender
    // let ascender_y = position.y + ascender;
    // gizmos.line_2d(
    //     Vec2::new(position.x, ascender_y),
    //     Vec2::new(position.x + advance_width, ascender_y),
    //     color,
    // );

    // // descender
    // let descender_y = position.y + descender;
    // gizmos.line_2d(
    //     Vec2::new(position.x, descender_y),
    //     Vec2::new(position.x + advance_width, descender_y),
    //     color,
    // );

    // // Draw bounding box from descender to UPM (units per em)
    // let top_left = Vec2::new(position.x, position.y + upm as f32);
    // let bottom_right = (position.x + advance_width, position.y + descender);
    // draw_rect(gizmos, top_left, bottom_right, color.with_alpha(0.7));
}

/// Draw a rectangle outline in design space, This is the sort bounding box
/// DISABLED: Gizmo-based rectangle drawing - now using mesh system
fn draw_rect(
    _gizmos: &mut Gizmos,
    _top_left: Vec2,
    _bottom_right: (f32, f32),
    _color: Color,
) {
    // DISABLED: Gizmo-based rectangle drawing - now using mesh system
    // let br: Vec2 = bottom_right.into();
    // gizmos.line_2d(top_left, Vec2::new(br.x, top_left.y), color);
    // gizmos.line_2d(Vec2::new(br.x, top_left.y), br, color);
    // gizmos.line_2d(br, Vec2::new(top_left.x, br.y), color);
    // gizmos.line_2d(Vec2::new(top_left.x, br.y), top_left, color);
}

/// Draw FontIR metrics for a glyph at a specific design-space position
/// DISABLED: Gizmo-based metrics rendering - now using mesh system
pub fn draw_fontir_metrics_at_position(
    _gizmos: &mut Gizmos,
    _advance_width: f32,
    _metrics: &FontIRMetrics,
    _position: Vec2,
    _color: Color,
) {
    // DISABLED: All gizmo-based metrics rendering - now using mesh system
    // let upm = metrics.units_per_em;
    // let ascender = metrics.ascender.unwrap_or(upm * 0.8);
    // let descender = metrics.descender.unwrap_or(upm * -0.2);
    // let x_height = metrics.x_height.unwrap_or(upm * 0.5);
    // let cap_height = metrics.cap_height.unwrap_or(upm * 0.7);
    
    // // Baseline (most important)
    // gizmos.line_2d(
    //     position,
    //     Vec2::new(position.x + advance_width, position.y),
    //     color,
    // );

    // // x-height
    // let x_height_y = position.y + x_height;
    // gizmos.line_2d(
    //     Vec2::new(position.x, x_height_y),
    //     Vec2::new(position.x + advance_width, x_height_y),
    //     color,
    // );

    // // cap-height
    // let cap_height_y = position.y + cap_height;
    // gizmos.line_2d(
    //     Vec2::new(position.x, cap_height_y),
    //     Vec2::new(position.x + advance_width, cap_height_y),
    //     color,
    // );

    // // Ascender line
    // gizmos.line_2d(
    //     Vec2::new(position.x, position.y + ascender),
    //     Vec2::new(position.x + advance_width, position.y + ascender),
    //     color,
    // );

    // // Descender line
    // gizmos.line_2d(
    //     Vec2::new(position.x, position.y + descender),
    //     Vec2::new(position.x + advance_width, position.y + descender),
    //     color,
    // );

    // // Advance width line (vertical)
    // gizmos.line_2d(
    //     Vec2::new(position.x + advance_width, position.y + descender),
    //     Vec2::new(position.x + advance_width, position.y + ascender),
    //     color,
    // );

    // // Draw bounding box from descender to UPM (units per em)
    // let top_left = Vec2::new(position.x, position.y + upm);
    // let bottom_right = (position.x + advance_width, position.y + descender);
    // draw_rect(gizmos, top_left, bottom_right, color.with_alpha(0.7));
}

/// Plugin for mesh-based metrics line rendering
pub struct MetricsRenderingPlugin;

impl Plugin for MetricsRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetricsLineEntities>()
           .add_systems(Update, render_mesh_metrics_lines);
    }
}
