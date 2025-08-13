//! # Pen Tool
//!
//! The pen tool allows users to draw vector paths by clicking points in sequence.
//! Click to place points, click near the start point to close the path, or right-click
//! to finish an open path. Hold Shift for axis-aligned drawing, press Escape to cancel.
//!
//! The tool converts placed points into UFO contours that are saved to the font file.

#![allow(clippy::too_many_arguments)]

use super::{EditTool, ToolInfo};
use crate::core::io::input::{helpers, InputEvent, InputMode, InputState};
use crate::core::io::pointer::PointerInfo;
use crate::core::state::{AppState, ContourData, PointData, PointTypeData};
use crate::editing::selection::events::AppStateChanged;
use crate::geometry::design_space::DPoint;
use crate::rendering::camera_responsive::CameraResponsiveScale;
use crate::systems::ui_interaction::UiHoverState;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::{BezPath, Point};

pub struct PenTool;

impl EditTool for PenTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "pen",
            display_name: "Pen",
            icon: "\u{E011}",
            tooltip: "Draw paths and contours",
            shortcut: Some(KeyCode::KeyP),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(true));
        commands.insert_resource(InputMode::Pen);
        info!("Entered Pen tool");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(false));
        commands.insert_resource(InputMode::Normal);
        info!("Exited Pen tool");
    }

    fn update(&self, _commands: &mut Commands) {
        // Pen tool behavior handled by dedicated systems
    }
}

// ================================================================
// CONSTANTS
// ================================================================

/// Distance threshold for closing a path by clicking near the start point
const CLOSE_PATH_THRESHOLD: f32 = 16.0;
/// Size of drawn points in the preview
const POINT_PREVIEW_SIZE: f32 = 4.0;
const CURSOR_INDICATOR_SIZE: f32 = 4.0;

// ================================================================
// RESOURCES AND STATE
// ================================================================

/// Resource to track if pen mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct PenModeActive(pub bool);

/// Input consumer for pen tool
#[derive(Resource)]
pub struct PenInputConsumer;

impl crate::systems::input_consumer::InputConsumer for PenInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Only handle input if pen mode is active
        if !helpers::is_input_mode(input_state, InputMode::Pen) {
            return false;
        }

        // Handle mouse events
        matches!(
            event,
            InputEvent::MouseClick { .. }
                | InputEvent::MouseRelease { .. }
                | InputEvent::KeyPress { .. }
        )
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        debug!("Pen tool handling input: {:?}", event);
        // Input handling is done in dedicated systems for better ECS integration
    }
}

/// Current state of the pen tool's path drawing
/// This is the shared state between input handling and rendering
#[derive(Resource, Default)]
pub struct PenToolState {
    /// Points that have been placed in the current path
    pub current_path: Vec<DPoint>,
    /// Whether the path should be closed (clicking near start point)
    pub should_close_path: bool,
    /// Whether we are currently placing a path
    pub is_drawing: bool,
}

/// Component to mark pen tool preview elements for cleanup
#[derive(Component)]
pub struct PenPreviewElement;

// ================================================================
// PLUGIN SETUP
// ================================================================

/// Bevy plugin that sets up the pen tool
pub struct PenToolPlugin;

impl Plugin for PenToolPlugin {
    fn build(&self, app: &mut App) {
        info!("🖊️ Registering PenToolPlugin systems");
        app.init_resource::<PenToolState>()
            .init_resource::<PenModeActive>()
            .add_systems(Startup, pen_tool_startup_log)
            .add_systems(
                Update,
                (
                    // handle_pen_mouse_events, // DISABLED - using InputConsumer system instead
                    handle_pen_keyboard_events,
                    render_pen_preview,
                    reset_pen_mode_when_inactive,
                    debug_pen_tool_state,
                ),
            );
    }
}

/// Startup system to confirm pen tool plugin loaded
fn pen_tool_startup_log() {
    info!("🖊️ PenToolPlugin successfully initialized with all systems");
}

/// Debug system to monitor pen tool state
fn debug_pen_tool_state(
    pen_state: Res<PenToolState>,
    time: Res<Time>,
) {
    static mut LAST_LOG: f32 = 0.0;
    let current_time = time.elapsed_secs();
    
    unsafe {
        if current_time - LAST_LOG > 2.0 && !pen_state.current_path.is_empty() {
            LAST_LOG = current_time;
            info!("🖊️ PEN STATE: {} points in path, drawing: {}", 
                  pen_state.current_path.len(), pen_state.is_drawing);
        }
    }
}

// ================================================================
// SYSTEMS
// ================================================================

/// System to handle mouse events for the pen tool
pub fn handle_pen_mouse_events(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Option<Res<PenModeActive>>,
    current_tool: Option<Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>>,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    pointer_info: Res<PointerInfo>,
    ui_hover_state: Res<UiHoverState>,
) {
    // Check if pen tool is active via multiple methods
    let pen_is_active = pen_mode_active.as_ref().is_some_and(|p| p.0) 
        || (current_tool.as_ref().and_then(|t| t.get_current()) == Some("pen"));
    
    // Debug: Always log current state during mouse button presses (ALWAYS LOG REGARDLESS OF PEN STATE)
    if mouse_button_input.just_pressed(MouseButton::Left) || mouse_button_input.just_pressed(MouseButton::Right) {
        info!("🖊️ PEN TOOL SYSTEM: Mouse button pressed - pen_is_active: {}, pen_mode_active: {:?}, current_tool: {:?}, ui_hovering: {}", 
              pen_is_active,
              pen_mode_active.as_ref().map(|p| p.0), 
              current_tool.as_ref().and_then(|t| t.get_current()),
              ui_hover_state.is_hovering_ui);
    }
    
    if !pen_is_active || ui_hover_state.is_hovering_ui {
        return;
    }

    info!("Pen tool: Mouse input system active and processing clicks");

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let click_position = pointer_info.design;
        info!("Pen tool: Left click detected at ({:.1}, {:.1})", click_position.x, click_position.y);

        // Check if we should close the path
        if pen_state.current_path.len() > 2 {
            if let Some(first_point) = pen_state.current_path.first() {
                let distance =
                    click_position.to_raw().distance(first_point.to_raw());
                if distance < CLOSE_PATH_THRESHOLD {
                    pen_state.should_close_path = true;
                    info!("Pen tool: Closing path - clicked near start point");
                    finalize_pen_path(
                        &mut pen_state,
                        &mut app_state,
                        &mut fontir_app_state,
                        &mut app_state_changed,
                    );
                    return;
                }
            }
        }

        // Add point to current path
        pen_state.current_path.push(click_position);
        pen_state.is_drawing = true;

        info!(
            "Pen tool: Added point at ({:.1}, {:.1}), total points: {}",
            click_position.x,
            click_position.y,
            pen_state.current_path.len()
        );
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        info!("Pen tool: Right click detected");
        // Finish open path
        if pen_state.current_path.len() > 1 {
            info!("Pen tool: Finishing open path with {} points", pen_state.current_path.len());
            finalize_pen_path(
                &mut pen_state,
                &mut app_state,
                &mut fontir_app_state,
                &mut app_state_changed,
            );
        }
    }
}

/// System to handle keyboard events for the pen tool
pub fn handle_pen_keyboard_events(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !pen_mode_active.0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Cancel current path
        pen_state.current_path.clear();
        pen_state.is_drawing = false;
        pen_state.should_close_path = false;
        info!("Pen tool: Cancelled current path");
    }
}

/// System to render the pen tool preview using unified mesh-based rendering
pub fn render_pen_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pen_state: Res<PenToolState>,
    pen_mode_active: Option<Res<PenModeActive>>,
    current_tool: Option<Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>>,
    pointer_info: Res<PointerInfo>,
    camera_scale: Res<CameraResponsiveScale>,
    existing_preview_query: Query<Entity, With<PenPreviewElement>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
) {
    // Clean up existing preview elements
    for entity in existing_preview_query.iter() {
        commands.entity(entity).despawn();
    }

    // Check if pen tool is active via multiple methods
    let pen_is_active = pen_mode_active.as_ref().is_some_and(|p| p.0) 
        || (current_tool.as_ref().and_then(|t| t.get_current()) == Some("pen"));
    
    if !pen_is_active {
        return;
    }

    // Debug: Log the current path state
    if !pen_state.current_path.is_empty() {
        info!("Pen tool preview: Rendering {} points", pen_state.current_path.len());
    }

    // Use theme colors: orange ACTION color for points and lines
    let action_color = theme.theme().action_color();
    let active_color = theme.theme().active_color(); // Green for closure indicator
    let line_width = camera_scale.adjusted_line_width() * 2.0;
    
    // Check if cursor is hovering over start point for closure
    let cursor_pos = Vec2::new(pointer_info.design.x, pointer_info.design.y);
    let hovering_start_point = if pen_state.current_path.len() > 2 {
        if let Some(first_point) = pen_state.current_path.first() {
            let first_pos = Vec2::new(first_point.x, first_point.y);
            let distance = cursor_pos.distance(first_pos);
            distance < CLOSE_PATH_THRESHOLD
        } else {
            false
        }
    } else {
        false
    };

    // Draw current path points
    for (i, &point) in pen_state.current_path.iter().enumerate() {
        let pos = Vec2::new(point.x, point.y);
        
        // Spawn point mesh using orange ACTION color
        spawn_pen_preview_point(
            &mut commands,
            &mut meshes,
            &mut materials,
            pos,
            action_color,
            camera_scale.adjusted_point_size(POINT_PREVIEW_SIZE),
        );

        // Draw dashed line to next point
        if i > 0 {
            let prev_point = pen_state.current_path[i - 1];
            let prev_pos = Vec2::new(prev_point.x, prev_point.y);
            spawn_pen_preview_dashed_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                prev_pos,
                pos,
                action_color,
                line_width,
            );
        }
    }

    // Draw dashed preview line to cursor if we have at least one point
    if let Some(&last_point) = pen_state.current_path.last() {
        let last_pos = Vec2::new(last_point.x, last_point.y);
        let cursor_pos = Vec2::new(pointer_info.design.x, pointer_info.design.y);
        spawn_pen_preview_dashed_line(
            &mut commands,
            &mut meshes,
            &mut materials,
            last_pos,
            cursor_pos,
            action_color.with_alpha(0.5),
            line_width * 0.5,
        );
    }

    // Draw green circle outline when hovering over start point for closure
    if hovering_start_point {
        if let Some(first_point) = pen_state.current_path.first() {
            let first_pos = Vec2::new(first_point.x, first_point.y);
            spawn_pen_closure_indicator(
                &mut commands,
                &mut meshes,
                &mut materials,
                first_pos,
                active_color,
                CLOSE_PATH_THRESHOLD,  // Use the threshold as radius
                camera_scale.adjusted_line_width(),  // Use standard zoom-aware line thickness
            );
        }
    }
    
    // Draw cursor indicator using orange ACTION color
    spawn_pen_preview_point(
        &mut commands,
        &mut meshes,
        &mut materials,
        cursor_pos,
        action_color.with_alpha(0.7),
        camera_scale.adjusted_point_size(CURSOR_INDICATOR_SIZE),
    );
}

/// System to reset pen mode when it becomes inactive
pub fn reset_pen_mode_when_inactive(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    if pen_mode_active.is_changed() && !pen_mode_active.0 {
        pen_state.current_path.clear();
        pen_state.is_drawing = false;
        pen_state.should_close_path = false;
        app_state_changed.write(AppStateChanged);
        debug!("Reset pen state due to mode change");
    }
}

/// Helper function to finalize the current pen path
fn finalize_pen_path(
    pen_state: &mut ResMut<PenToolState>,
    _app_state: &mut Option<ResMut<AppState>>,
    fontir_app_state: &mut Option<ResMut<crate::core::state::FontIRAppState>>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    if pen_state.current_path.len() < 2 {
        return;
    }

    // Try FontIR first, then fallback to AppState
    if fontir_app_state.is_some() {
        finalize_fontir_path(pen_state, fontir_app_state, app_state_changed);
    } else if let Some(_app_state) = _app_state.as_mut() {
        finalize_appstate_path(pen_state);
    } else {
        warn!(
            "Pen tool: No AppState or FontIR available for path finalization"
        );
    }

    // Reset state
    pen_state.current_path.clear();
    pen_state.is_drawing = false;
    pen_state.should_close_path = false;

    app_state_changed.write(AppStateChanged);
}

/// Helper function to finalize path using FontIR BezPath operations
fn finalize_fontir_path(
    pen_state: &mut ResMut<PenToolState>,
    fontir_app_state: &mut Option<ResMut<crate::core::state::FontIRAppState>>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    // Create a BezPath from the current path
    let mut bez_path = BezPath::new();

    if let Some(&first_point) = pen_state.current_path.first() {
        bez_path
            .move_to(Point::new(first_point.x as f64, first_point.y as f64));

        for &point in pen_state.current_path.iter().skip(1) {
            bez_path.line_to(Point::new(point.x as f64, point.y as f64));
        }

        if pen_state.should_close_path {
            bez_path.close_path();
        }
    }

    // Add the BezPath to the FontIR glyph data if available
    if let Some(ref mut fontir_state) = fontir_app_state.as_mut() {
        let current_glyph_name = fontir_state.current_glyph.clone();
        if let Some(current_glyph_name) = current_glyph_name {
            // Get the current location
            let location = fontir_state.current_location.clone();
            let key = (current_glyph_name.clone(), location);

            // Get or create a working copy
            let working_copy_exists = fontir_state.working_copies.contains_key(&key);
            
            if !working_copy_exists {
                // Create working copy from original FontIR data
                if let Some(fontir_glyph) = fontir_state.glyph_cache.get(&current_glyph_name) {
                    if let Some((_location, instance)) = fontir_glyph.sources().iter().next() {
                        let working_copy = crate::core::state::fontir_app_state::EditableGlyphInstance::from(instance);
                        fontir_state.working_copies.insert(key.clone(), working_copy);
                    }
                }
            }

            // Add the new contour to the working copy
            if let Some(working_copy) = fontir_state.working_copies.get_mut(&key) {
                working_copy.contours.push(bez_path.clone());
                working_copy.is_dirty = true;
                app_state_changed.write(AppStateChanged);
                
                info!("Pen tool (FontIR): Added contour with {} elements to glyph '{}'. Total contours: {}", 
                      bez_path.elements().len(), current_glyph_name, working_copy.contours.len());
            } else {
                warn!("Pen tool (FontIR): Could not create working copy for glyph '{}'", current_glyph_name);
            }
        } else {
            warn!("Pen tool (FontIR): No current glyph selected");
        }
    } else {
        warn!("Pen tool (FontIR): FontIR app state not available");
    }

    let path_info =
        format!("BezPath with {} elements", bez_path.elements().len());
    info!("Pen tool (FontIR): Created {} for current glyph", path_info);
}

/// Helper function to finalize path using traditional AppState operations
fn finalize_appstate_path(pen_state: &mut ResMut<PenToolState>) {
    // Convert path to ContourData
    let mut points = Vec::new();

    for (i, &point) in pen_state.current_path.iter().enumerate() {
        let point_type = if i == 0 {
            PointTypeData::Move
        } else {
            PointTypeData::Line
        };

        points.push(PointData {
            x: point.x as f64,
            y: point.y as f64,
            point_type,
        });
    }

    // Close path if requested
    if pen_state.should_close_path {
        points.push(PointData {
            x: pen_state.current_path[0].x as f64,
            y: pen_state.current_path[0].y as f64,
            point_type: PointTypeData::Line,
        });
    }

    let contour = ContourData { points };

    // Add contour to current glyph - this needs to be done through a proper system
    // For now just log that we would add it
    info!("Pen tool (AppState): Would add {} point contour to current glyph (contour has {} points)", 
          pen_state.current_path.len(), contour.points.len());
}

// ================================================================
// MESH-BASED PREVIEW HELPERS
// ================================================================

/// Create a mesh-based point for pen tool preview
fn spawn_pen_preview_point(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    color: Color,
    size: f32,
) {
    let mesh = Mesh::from(Circle::new(size));
    let material = ColorMaterial::from(color);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(material)),
        Transform::from_translation(position.extend(10.0)), // Z=10 to render above glyph
        PenPreviewElement,
    ));
}

/// Create a mesh-based line for pen tool preview
#[allow(dead_code)]
fn spawn_pen_preview_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    width: f32,
) {
    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x);
    let half_width = width * 0.5;
    
    // Calculate the center position for the transform
    let center = (start + end) * 0.5;
    
    // Create vertices relative to the center position
    let p1 = (start - center) + perpendicular * half_width;
    let p2 = (start - center) - perpendicular * half_width;
    let p3 = (end - center) - perpendicular * half_width;
    let p4 = (end - center) + perpendicular * half_width;
    
    let vertices = vec![
        [p1.x, p1.y, 0.0],
        [p2.x, p2.y, 0.0],
        [p3.x, p3.y, 0.0],
        [p4.x, p4.y, 0.0],
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    let material = ColorMaterial::from(color);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(material)),
        Transform::from_translation(center.extend(5.0)), // Z=5 to render above glyph but below points
        PenPreviewElement,
    ));
}

/// Create dashed line mesh entities for pen tool preview
fn spawn_pen_preview_dashed_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    width: f32,
) {
    let dash_length = 8.0;
    let gap_length = 4.0;
    
    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start_pos = current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);

        let dash_start = start + direction * dash_start_pos;
        let dash_end = start + direction * dash_end_pos;

        // Create mesh for this dash segment using the existing mesh utility
        let line_mesh = crate::rendering::mesh_utils::create_line_mesh(
            dash_start, dash_end, width,
        );

        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_xyz(
                (dash_start.x + dash_end.x) * 0.5,
                (dash_start.y + dash_end.y) * 0.5,
                5.0, // Z=5 to render above glyph but below points
            ),
            PenPreviewElement,
        ));

        current_pos += segment_length;
    }
}

/// Create a green circle outline indicator when hovering over start point for closure
fn spawn_pen_closure_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    color: Color,
    radius: f32,
    line_width: f32,
) {
    // Create a thin circle outline (ring) mesh using the line width for thickness
    let segments = 32;
    let outer_radius = radius;
    let inner_radius = radius - line_width; // Use line width to control ring thickness
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    
    // Create vertices for the ring
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // Outer vertex
        vertices.push([outer_radius * cos_a, outer_radius * sin_a, 0.0]);
        uvs.push([0.5 + 0.5 * cos_a, 0.5 + 0.5 * sin_a]);
        normals.push([0.0, 0.0, 1.0]);
        
        // Inner vertex
        vertices.push([inner_radius * cos_a, inner_radius * sin_a, 0.0]);
        uvs.push([0.5 + 0.5 * cos_a * (inner_radius / outer_radius), 0.5 + 0.5 * sin_a * (inner_radius / outer_radius)]);
        normals.push([0.0, 0.0, 1.0]);
    }
    
    // Create indices for the ring triangles
    for i in 0..segments {
        let base = i * 2;
        let next_base = ((i + 1) % (segments + 1)) * 2;
        
        // Triangle 1: outer[i] -> inner[i] -> outer[i+1]
        indices.push(base as u32);
        indices.push((base + 1) as u32);
        indices.push(next_base as u32);
        
        // Triangle 2: inner[i] -> inner[i+1] -> outer[i+1]
        indices.push((base + 1) as u32);
        indices.push((next_base + 1) as u32);
        indices.push(next_base as u32);
    }
    
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    let material = ColorMaterial::from(color);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(material)),
        Transform::from_translation(position.extend(12.0)), // Z=12 to render above points
        PenPreviewElement,
    ));
}
