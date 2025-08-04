//! Sort rendering for text editor sorts

#![allow(clippy::too_many_arguments)]

use crate::core::state::text_editor::{SortKind, SortLayoutMode};
use crate::core::state::{AppState, TextEditorState};
use crate::rendering::entity_pools::{
    update_cursor_entity, EntityPools, PooledEntityType,
};
use crate::ui::theme::*;
use crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode;
// TextPlacementMode import removed - not used in new mesh-based cursor
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

/// Component to mark text editor cursor entities
#[derive(Component)]
pub struct TextEditorCursor;

/// Resource to track cursor state for change detection
#[derive(Resource, Default)]
pub struct CursorRenderingState {
    pub last_cursor_position: Option<Vec2>,
    pub last_tool: Option<String>,
    pub last_placement_mode:
        Option<crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode>,
    pub last_buffer_cursor_position: Option<usize>,
    pub last_camera_scale: Option<f32>,
}

/// Text editor sorts are now rendered by the main mesh glyph outline system
/// This function exists for compatibility but the actual rendering happens
/// automatically through the ECS query in render_mesh_glyph_outline()
pub fn render_text_editor_sorts() {
    // Text editor sorts are rendered automatically by the mesh glyph outline system
    // since they are regular Sort entities with BufferSortIndex components.
    // No additional rendering logic needed here.
}

/// Render the visual cursor for Insert mode using zoom-aware mesh rendering
pub fn render_text_editor_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    camera_scale: Res<
        crate::rendering::camera_responsive::CameraResponsiveScale,
    >,
    _existing_cursors: Query<Entity, With<TextEditorCursor>>,
    mut cursor_state: ResMut<CursorRenderingState>,
    mut entity_pools: ResMut<EntityPools>,
) {
    info!(
        "CURSOR: System called - tool: {:?}, mode: {:?}",
        current_tool.get_current(),
        current_placement_mode.0
    );

    // Only render cursor when text tool is active and in Insert mode
    if current_tool.get_current() != Some("text") {
        info!(
            "CURSOR: Not rendering - text tool not active (current: {:?})",
            current_tool.get_current()
        );
        // Clear cursor entities when tool is not text
        entity_pools.return_cursor_entities(&mut commands);
        return;
    }

    // Only show cursor when in Insert mode
    if current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        info!(
            "CURSOR: Not rendering - not in Insert mode (current mode: {:?})",
            current_placement_mode.0
        );
        // Clear cursor entities when not in insert mode
        entity_pools.return_cursor_entities(&mut commands);
        return;
    }

    info!("CURSOR: Proceeding to render cursor (all checks passed)");

    // CHANGE DETECTION: Check if cursor needs updating
    let current_tool_name = current_tool.get_current();
    let current_placement_mode_value = current_placement_mode.0;
    let current_camera_scale = camera_scale.scale_factor;

    // Get current buffer cursor position
    let current_buffer_cursor_position =
        text_editor_state.as_ref().and_then(|state| {
            // Find active buffer root and get cursor position
            for i in 0..state.buffer.len() {
                if let Some(sort) = state.buffer.get(i) {
                    if sort.is_buffer_root && sort.is_active {
                        return sort.buffer_cursor_position;
                    }
                }
            }
            // Fallback: look for any buffer root with cursor position
            for i in 0..state.buffer.len() {
                if let Some(sort) = state.buffer.get(i) {
                    if sort.is_buffer_root
                        && sort.buffer_cursor_position.is_some()
                    {
                        return sort.buffer_cursor_position;
                    }
                }
            }
            None
        });

    // Calculate current cursor position
    let current_cursor_position =
        text_editor_state.as_ref().and_then(|state| {
            calculate_cursor_visual_position(
                state,
                &app_state,
                &fontir_app_state,
            )
        });

    // Check if anything changed
    let tool_changed = cursor_state.last_tool.as_deref() != current_tool_name;
    let placement_mode_changed =
        cursor_state.last_placement_mode != Some(current_placement_mode_value);
    let buffer_cursor_changed = cursor_state.last_buffer_cursor_position
        != current_buffer_cursor_position;
    let cursor_position_changed =
        cursor_state.last_cursor_position != current_cursor_position;
    let camera_scale_changed =
        cursor_state.last_camera_scale != Some(current_camera_scale);

    if !tool_changed
        && !placement_mode_changed
        && !buffer_cursor_changed
        && !cursor_position_changed
        && !camera_scale_changed
    {
        debug!("Cursor rendering skipped - no changes detected");
        return;
    }

    // ENTITY POOLING: Clear cursor entities before re-rendering
    entity_pools.return_cursor_entities(&mut commands);
    info!("CURSOR: Returned cursor entities to pool");

    // Update state tracking
    cursor_state.last_tool = current_tool_name.map(|s| s.to_string());
    cursor_state.last_placement_mode = Some(current_placement_mode_value);
    cursor_state.last_buffer_cursor_position = current_buffer_cursor_position;
    cursor_state.last_cursor_position = current_cursor_position;
    cursor_state.last_camera_scale = Some(current_camera_scale);

    debug!("Cursor rendering triggered - changes detected: tool={}, placement_mode={}, buffer_cursor={}, cursor_position={}, camera_scale={}", 
           tool_changed, placement_mode_changed, buffer_cursor_changed, cursor_position_changed, camera_scale_changed);

    debug!("Cursor mode: {:?}", current_placement_mode.0);

    debug!(
        "Cursor system running: text tool active, mode: {:?}",
        current_placement_mode.0
    );

    let Some(text_editor_state) = text_editor_state else {
        return;
    };

    // Calculate cursor visual position
    if let Some(cursor_world_pos) = calculate_cursor_visual_position(
        &text_editor_state,
        &app_state,
        &fontir_app_state,
    ) {
        // Get font metrics for proper cursor height - try FontIR first, then AppState
        let (upm, descender) =
            if let Some(fontir_state) = fontir_app_state.as_ref() {
                let metrics = fontir_state.get_font_metrics();
                (metrics.units_per_em, metrics.descender.unwrap_or(-256.0))
            } else if let Some(app_state) = app_state.as_ref() {
                let font_metrics = &app_state.workspace.info.metrics;
                (
                    font_metrics.units_per_em as f32,
                    font_metrics.descender.unwrap_or(-256.0) as f32,
                )
            } else {
                warn!(
                "Text cursor skipped - Neither FontIR nor AppState available"
            );
                return;
            };

        // Calculate cursor bounds based on font metrics
        let cursor_top = cursor_world_pos.y + upm; // UPM top
        let cursor_bottom = cursor_world_pos.y + descender; // Descender bottom
        let cursor_height = cursor_top - cursor_bottom;

        // Bright orange cursor color (like pre-refactor)
        let cursor_color = Color::srgb(1.0, 0.5, 0.0); // Bright orange

        // Create zoom-aware mesh-based cursor
        create_mesh_cursor(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut entity_pools,
            cursor_world_pos,
            cursor_top,
            cursor_bottom,
            cursor_color,
            &camera_scale,
        );

        debug!(
            "Text cursor rendered at ({:.1}, {:.1}), height: {:.1}",
            cursor_world_pos.x, cursor_world_pos.y, cursor_height
        );
    }
}

/// Calculate the visual world position of the cursor based on text buffer state
fn calculate_cursor_visual_position(
    text_editor_state: &TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<crate::core::state::FontIRAppState>>,
) -> Option<Vec2> {
    // Find the active buffer root
    let mut active_root_index = None;
    let mut cursor_pos_in_buffer = 0;
    let mut root_position = Vec2::ZERO;

    // Look for the active buffer root and get its cursor position
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            if sort.is_buffer_root && sort.is_active {
                active_root_index = Some(i);
                cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                root_position = sort.root_position;
                break;
            }
        }
    }

    // If no active root found, check for any buffer root with cursor position
    if active_root_index.is_none() {
        for i in 0..text_editor_state.buffer.len() {
            if let Some(sort) = text_editor_state.buffer.get(i) {
                if sort.is_buffer_root && sort.buffer_cursor_position.is_some()
                {
                    active_root_index = Some(i);
                    cursor_pos_in_buffer =
                        sort.buffer_cursor_position.unwrap_or(0);
                    root_position = sort.root_position;
                    break;
                }
            }
        }
    }

    let root_index = active_root_index?;

    // If cursor at position 0, place at root position
    if cursor_pos_in_buffer == 0 {
        return Some(root_position);
    }

    // Calculate position based on the glyphs in the buffer sequence, handling line breaks
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;
    let mut glyph_count = 0;

    // Get font metrics for line height calculation - try FontIR first, then AppState
    let (_upm, _descender, line_height) = if let Some(fontir_state) =
        fontir_app_state.as_ref()
    {
        let metrics = fontir_state.get_font_metrics();
        let upm = metrics.units_per_em;
        let descender = metrics.descender.unwrap_or(-256.0);
        (upm, descender, upm - descender)
    } else if let Some(app_state) = app_state.as_ref() {
        let font_metrics = &app_state.workspace.info.metrics;
        let upm = font_metrics.units_per_em as f32;
        let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
        (upm, descender, upm - descender)
    } else {
        warn!("Text cursor position calculation skipped - Neither FontIR nor AppState available");
        return Some(root_position); // Fallback to root position
    };

    // Start from the root and accumulate advances
    for i in root_index..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            // Stop if we hit another buffer root
            if i != root_index && sort.is_buffer_root {
                break;
            }

            // Count glyphs in this buffer sequence
            if i == root_index
                || sort.layout_mode == SortLayoutMode::LTRText
                || sort.layout_mode == SortLayoutMode::RTLText
            {
                // Handle different sort types
                match &sort.kind {
                    SortKind::LineBreak => {
                        // Apply line break first (move to next line)
                        x_offset = 0.0;
                        y_offset -= line_height;

                        // If cursor is positioned at this line break index, show it at the start of the new line
                        if glyph_count == cursor_pos_in_buffer {
                            return Some(Vec2::new(
                                root_position.x + x_offset,
                                root_position.y + y_offset,
                            ));
                        }
                    }
                    SortKind::Glyph {
                        glyph_name: _,
                        advance_width,
                    } => {
                        // If cursor is at this position, place it at the RIGHT edge of this glyph
                        if glyph_count == cursor_pos_in_buffer {
                            // For LTR: cursor goes at right edge (after advance)
                            // For RTL: cursor goes at left edge (before advance)
                            if sort.layout_mode == SortLayoutMode::RTLText {
                                return Some(Vec2::new(
                                    root_position.x + x_offset,
                                    root_position.y + y_offset,
                                ));
                            } else {
                                return Some(Vec2::new(
                                    root_position.x + x_offset + advance_width,
                                    root_position.y + y_offset,
                                ));
                            }
                        }

                        // Add advance width for positioning next characters
                        if sort.layout_mode == SortLayoutMode::RTLText {
                            x_offset -= advance_width;
                        } else {
                            x_offset += advance_width;
                        }
                    }
                }

                glyph_count += 1;
            }
        }
    }

    // Cursor is at or beyond the end, position after last glyph
    Some(Vec2::new(
        root_position.x + x_offset,
        root_position.y + y_offset,
    ))
}

/// Create a mesh-based cursor with triangular ends
fn create_mesh_cursor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    entity_pools: &mut ResMut<EntityPools>,
    cursor_pos: Vec2,
    cursor_top: f32,
    cursor_bottom: f32,
    cursor_color: Color,
    camera_scale: &crate::rendering::camera_responsive::CameraResponsiveScale,
) {
    let outline_width = camera_scale.adjusted_line_width();
    let cursor_width = outline_width * 4.0; // 4x the outline width
    let circle_size = cursor_width * 4.0;

    // Create main vertical line mesh
    let line_mesh = create_cursor_line_mesh(
        Vec2::new(cursor_pos.x, cursor_bottom),
        Vec2::new(cursor_pos.x, cursor_top),
        cursor_width,
    );

    // Create circle meshes for top and bottom
    let top_circle_mesh = create_circle_mesh(circle_size);
    let bottom_circle_mesh = create_circle_mesh(circle_size);

    let cursor_material = materials.add(ColorMaterial::from(cursor_color));
    let cursor_z = 15.0; // Above everything else

    // Get cursor line entity from pool
    let line_entity =
        entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        line_entity,
        meshes.add(line_mesh),
        cursor_material.clone(),
        Transform::from_xyz(
            cursor_pos.x,
            (cursor_top + cursor_bottom) * 0.5,
            cursor_z,
        ),
        TextEditorCursor,
    );

    debug!("Updated pooled cursor line entity: {:?}", line_entity);

    // Get top circle entity from pool
    let top_circle_entity =
        entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        top_circle_entity,
        meshes.add(top_circle_mesh),
        cursor_material.clone(),
        Transform::from_xyz(cursor_pos.x, cursor_top, cursor_z),
        TextEditorCursor,
    );

    debug!(
        "Updated pooled cursor top circle entity: {:?}",
        top_circle_entity
    );

    // Get bottom circle entity from pool
    let bottom_circle_entity =
        entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        bottom_circle_entity,
        meshes.add(bottom_circle_mesh),
        cursor_material,
        Transform::from_xyz(cursor_pos.x, cursor_bottom, cursor_z),
        TextEditorCursor,
    );

    debug!(
        "Updated pooled cursor bottom circle entity: {:?}",
        bottom_circle_entity
    );
}

/// Create a vertical line mesh for the cursor
fn create_cursor_line_mesh(start: Vec2, end: Vec2, width: f32) -> Mesh {
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

/// Create a circular mesh for cursor ends
fn create_circle_mesh(diameter: f32) -> Mesh {
    let radius = diameter * 0.5;
    let segments = 32; // Number of segments for circle smoothness

    let mut vertices = vec![[0.0, 0.0, 0.0]]; // Center vertex
    let mut uvs = vec![[0.5, 0.5]]; // Center UV
    let mut indices = Vec::new();

    // Create circle vertices around the perimeter
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = radius * angle.cos();
        let y = radius * angle.sin();

        vertices.push([x, y, 0.0]);

        // UV coordinates mapped from -1,1 to 0,1
        let u = (x / radius + 1.0) * 0.5;
        let v = (y / radius + 1.0) * 0.5;
        uvs.push([u, v]);

        // Create triangle indices (center, current, next)
        let next_i = (i + 1) % segments;
        indices.push(0); // Center
        indices.push((i + 1) as u32); // Current vertex
        indices.push((next_i + 1) as u32); // Next vertex
    }

    let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];

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
