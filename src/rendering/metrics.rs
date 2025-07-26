//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

use crate::core::state::font_metrics::FontMetrics;
use crate::core::state::fontir_app_state::FontIRMetrics;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::ui::theme::METRICS_GUIDE_COLOR;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::ParamCurve;

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

/// Component to mark entities as preview metrics (temporary entities)
#[derive(Component)]
pub struct PreviewMetricsLine;

/// Component to mark entities as preview glyph outline elements
#[derive(Component)]
pub struct PreviewGlyphOutline;

/// Resource to track preview metrics entities that need to be cleaned up
#[derive(Resource, Default)]
pub struct PreviewMetricsEntities {
    pub entities: Vec<Entity>,
}

/// Resource to store current preview state for mesh-based preview system
#[derive(Resource, Default)]
pub struct PreviewMetricsState {
    pub active: bool,
    pub position: Vec2,
    pub glyph_name: String,
    pub advance_width: f32,
    pub color: Color,
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
    let line_mesh = crate::rendering::mesh_glyph_outline::create_line_mesh(
        start, end, line_width,
    );

    commands
        .spawn((
            MetricsLine {
                sort_entity,
                line_type,
            },
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(
                (start.x + end.x) * 0.5,
                (start.y + end.y) * 0.5,
                METRICS_LINE_Z,
            ),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id()
}

/// System to render mesh-based metrics lines for all active sorts
pub fn render_mesh_metrics_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut metrics_entities: ResMut<MetricsLineEntities>,
    sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        With<crate::editing::sort::ActiveSort>,
    >,
    // Add query for ACTIVE text buffer sorts (text roots)
    active_buffer_sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        (
            With<crate::systems::text_editor_sorts::sort_entities::BufferSortIndex>,
            With<crate::editing::sort::ActiveSort>,
        ),
    >,
    // Add query for INACTIVE text buffer sorts (typed characters)
    inactive_buffer_sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        (
            With<crate::systems::text_editor_sorts::sort_entities::BufferSortIndex>,
            With<crate::editing::sort::InactiveSort>,
        ),
    >,
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
            let advance_width =
                fontir_state.get_glyph_advance_width(&sort.glyph_name);
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
            let bottom_right =
                Vec2::new(position.x + advance_width, descender_y);

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

        // Render metrics for ACTIVE buffer sorts (text roots) with green color
        for (sort_entity, sort_transform, sort) in active_buffer_sort_query.iter() {
            let position = sort_transform.translation.truncate();
            let advance_width =
                fontir_state.get_glyph_advance_width(&sort.glyph_name);
            let color = crate::ui::theme::SORT_ACTIVE_METRICS_COLOR; // Green for active buffer sorts (text roots)

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

            // Draw bounding box lines (4 lines for rectangle) with more transparency
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right =
                Vec2::new(position.x + advance_width, descender_y);

            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color.with_alpha(0.3), // More subtle for text
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
                color.with_alpha(0.3),
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
                color.with_alpha(0.3),
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
                color.with_alpha(0.3),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);

            metrics_entities.lines.insert(sort_entity, line_entities);
        }

        // Render metrics for INACTIVE buffer sorts (typed characters) with gray color
        for (sort_entity, sort_transform, sort) in inactive_buffer_sort_query.iter() {
            let position = sort_transform.translation.truncate();
            let advance_width =
                fontir_state.get_glyph_advance_width(&sort.glyph_name);
            let color = crate::ui::theme::SORT_INACTIVE_METRICS_COLOR; // Gray for inactive buffer sorts (typed characters)

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

            // Draw bounding box lines (4 lines for rectangle) with more transparency
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right =
                Vec2::new(position.x + advance_width, descender_y);

            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color.with_alpha(0.3), // More subtle for text
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
                color.with_alpha(0.3),
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
                color.with_alpha(0.3),
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
                color.with_alpha(0.3),
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);

            metrics_entities.lines.insert(sort_entity, line_entities);
        }
    }
}




/// System to manage mesh-based preview metrics
pub fn manage_preview_metrics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut preview_entities: ResMut<PreviewMetricsEntities>,
    preview_state: Res<PreviewMetricsState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clean up existing preview entities
    for entity in preview_entities.entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Only create new preview if active and we have fontir state
    if !preview_state.active || preview_state.glyph_name.is_empty() {
        return;
    }

    let Some(fontir_state) = fontir_app_state.as_ref() else {
        return;
    };

    let font_metrics = fontir_state.get_font_metrics();
    let upm = font_metrics.units_per_em;
    let ascender = font_metrics.ascender.unwrap_or(upm * 0.8);
    let descender = font_metrics.descender.unwrap_or(upm * -0.2);
    let x_height = font_metrics.x_height.unwrap_or(upm * 0.5);
    let cap_height = font_metrics.cap_height.unwrap_or(upm * 0.7);
    
    let temp_entity = Entity::from_raw(0); // Placeholder for preview
    let position = preview_state.position;
    let advance_width = preview_state.advance_width;
    let color = preview_state.color;

    // Baseline (most important)
    let baseline_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        Vec2::new(position.x + advance_width, position.y),
        color,
        temp_entity,
        MetricsLineType::Baseline,
        &camera_scale,
    );
    preview_entities.entities.push(baseline_entity);

    // x-height
    let x_height_y = position.y + x_height;
    let x_height_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, x_height_y),
        Vec2::new(position.x + advance_width, x_height_y),
        color,
        temp_entity,
        MetricsLineType::XHeight,
        &camera_scale,
    );
    preview_entities.entities.push(x_height_entity);

    // cap-height
    let cap_height_y = position.y + cap_height;
    let cap_height_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, cap_height_y),
        Vec2::new(position.x + advance_width, cap_height_y),
        color,
        temp_entity,
        MetricsLineType::CapHeight,
        &camera_scale,
    );
    preview_entities.entities.push(cap_height_entity);

    // ascender
    let ascender_y = position.y + ascender;
    let ascender_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, ascender_y),
        Vec2::new(position.x + advance_width, ascender_y),
        color,
        temp_entity,
        MetricsLineType::Ascender,
        &camera_scale,
    );
    preview_entities.entities.push(ascender_entity);

    // descender
    let descender_y = position.y + descender;
    let descender_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, descender_y),
        Vec2::new(position.x + advance_width, descender_y),
        color,
        temp_entity,
        MetricsLineType::Descender,
        &camera_scale,
    );
    preview_entities.entities.push(descender_entity);

    // Advance width line (vertical)
    let advance_width_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x + advance_width, descender_y),
        Vec2::new(position.x + advance_width, ascender_y),
        color,
        temp_entity,
        MetricsLineType::AdvanceWidth,
        &camera_scale,
    );
    preview_entities.entities.push(advance_width_entity);

    // Draw bounding box lines (4 lines for rectangle)
    let top_left = Vec2::new(position.x, position.y + upm);
    let bottom_right = Vec2::new(position.x + advance_width, descender_y);

    // Top line
    let top_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        top_left,
        Vec2::new(bottom_right.x, top_left.y),
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(top_entity);

    // Right line
    let right_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(bottom_right.x, top_left.y),
        bottom_right,
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(right_entity);

    // Bottom line
    let bottom_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        bottom_right,
        Vec2::new(top_left.x, bottom_right.y),
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(bottom_entity);

    // Left line
    let left_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(top_left.x, bottom_right.y),
        top_left,
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(left_entity);

    // Add glyph outline preview
    if let Some(glyph_paths) = fontir_state.get_glyph_paths_with_edits(&preview_state.glyph_name) {
        for path in &glyph_paths {
            let outline_entities = create_preview_glyph_outline(
                &mut commands,
                &mut meshes,
                &mut materials,
                path,
                position,
                color.with_alpha(0.6), // Slightly transparent for preview
                &camera_scale,
            );
            preview_entities.entities.extend(outline_entities);
        }
    }
}

/// Helper function to spawn preview metrics lines with special component
fn spawn_preview_metrics_line(
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
    // Use the existing spawn_metrics_line function but add PreviewMetricsLine component
    let entity = spawn_metrics_line(
        commands,
        meshes,
        materials,
        start,
        end,
        color,
        sort_entity,
        line_type,
        camera_scale,
    );
    
    // Add the preview component for identification
    commands.entity(entity).insert(PreviewMetricsLine);
    
    entity
}

/// Create preview glyph outline entities from a kurbo path with dashed lines
fn create_preview_glyph_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    path: &kurbo::BezPath,
    position: Vec2,
    color: Color,
    camera_scale: &CameraResponsiveScale,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let line_width = camera_scale.adjusted_line_width();
    let preview_z = 6.0; // Between metrics (5.0) and glyph outlines (8.0)
    
    // Dashing parameters (same as selection marquee)
    let dash_length = 8.0;
    let gap_length = 4.0;

    // Convert kurbo path to line segments for mesh rendering
    let mut current_pos: Option<kurbo::Point> = None;
    let mut segment_start: Option<kurbo::Point> = None;

    // Flatten the path into line segments
    let tolerance = 0.5; // Tessellation tolerance
    for element in path.iter() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                current_pos = Some(pt);
                segment_start = Some(pt);
            }
            kurbo::PathEl::LineTo(pt) => {
                if let Some(start) = current_pos {
                    let start_world = Vec2::new(start.x as f32 + position.x, start.y as f32 + position.y);
                    let end_world = Vec2::new(pt.x as f32 + position.x, pt.y as f32 + position.y);
                    
                    // Create dashed line segments
                    let dashed_entities = create_dashed_line_meshes(
                        commands, meshes, materials,
                        start_world, end_world, line_width, 
                        dash_length, gap_length, color, preview_z
                    );
                    entities.extend(dashed_entities);
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::QuadTo(cp, pt) => {
                if let Some(start) = current_pos {
                    // Tessellate quadratic curve into line segments
                    let quad = kurbo::QuadBez::new(start, cp, pt);
                    let mut prev_pt = start;
                    
                    // Simple tessellation - could be improved
                    for i in 1..=8 {
                        let t = i as f64 / 8.0;
                        let curr_pt = quad.eval(t);
                        
                        let start_world = Vec2::new(prev_pt.x as f32 + position.x, prev_pt.y as f32 + position.y);
                        let end_world = Vec2::new(curr_pt.x as f32 + position.x, curr_pt.y as f32 + position.y);
                        
                        // Create dashed line segments for tessellated curves
                        let dashed_entities = create_dashed_line_meshes(
                            commands, meshes, materials,
                            start_world, end_world, line_width, 
                            dash_length, gap_length, color, preview_z
                        );
                        entities.extend(dashed_entities);
                        prev_pt = curr_pt;
                    }
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::CurveTo(cp1, cp2, pt) => {
                if let Some(start) = current_pos {
                    // Tessellate cubic curve into line segments
                    let cubic = kurbo::CubicBez::new(start, cp1, cp2, pt);
                    let mut prev_pt = start;
                    
                    // Simple tessellation - could be improved
                    for i in 1..=12 {
                        let t = i as f64 / 12.0;
                        let curr_pt = cubic.eval(t);
                        
                        let start_world = Vec2::new(prev_pt.x as f32 + position.x, prev_pt.y as f32 + position.y);
                        let end_world = Vec2::new(curr_pt.x as f32 + position.x, curr_pt.y as f32 + position.y);
                        
                        // Create dashed line segments for tessellated curves
                        let dashed_entities = create_dashed_line_meshes(
                            commands, meshes, materials,
                            start_world, end_world, line_width, 
                            dash_length, gap_length, color, preview_z
                        );
                        entities.extend(dashed_entities);
                        prev_pt = curr_pt;
                    }
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::ClosePath => {
                if let (Some(current), Some(start)) = (current_pos, segment_start) {
                    // Close the path by connecting back to the start
                    let start_world = Vec2::new(current.x as f32 + position.x, current.y as f32 + position.y);
                    let end_world = Vec2::new(start.x as f32 + position.x, start.y as f32 + position.y);
                    
                    // Create dashed line segments for closing path
                    let dashed_entities = create_dashed_line_meshes(
                        commands, meshes, materials,
                        start_world, end_world, line_width, 
                        dash_length, gap_length, color, preview_z
                    );
                    entities.extend(dashed_entities);
                }
                current_pos = segment_start;
            }
        }
    }

    entities
}

/// Create dashed line mesh entities between two points
fn create_dashed_line_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    line_width: f32,
    dash_length: f32,
    gap_length: f32,
    color: Color,
    z: f32,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    
    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;
    
    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start_pos = current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        
        let dash_start = start + direction * dash_start_pos;
        let dash_end = start + direction * dash_end_pos;
        
        // Create mesh for this dash segment
        let line_mesh = crate::rendering::mesh_glyph_outline::create_line_mesh(
            dash_start, dash_end, line_width,
        );

        let entity = commands
            .spawn((
                PreviewGlyphOutline,
                bevy::render::mesh::Mesh2d(meshes.add(line_mesh)),
                bevy::sprite::MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_xyz(
                    (dash_start.x + dash_end.x) * 0.5,
                    (dash_start.y + dash_end.y) * 0.5,
                    z,
                ),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();
        entities.push(entity);
        
        current_pos += segment_length;
    }
    
    entities
}

/// Plugin for mesh-based metrics line rendering
pub struct MetricsRenderingPlugin;

impl Plugin for MetricsRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetricsLineEntities>()
            .init_resource::<PreviewMetricsEntities>()
            .init_resource::<PreviewMetricsState>()
            .add_systems(Update, (
                render_mesh_metrics_lines
                    .after(crate::rendering::mesh_glyph_outline::render_mesh_glyph_outline)
                    .after(crate::systems::text_editor_sorts::sort_entities::update_buffer_sort_positions),
                manage_preview_metrics
                    .after(render_mesh_metrics_lines)
            ));
    }
}
