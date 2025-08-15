//! Knife Tool - Path cutting and slicing tool
//!
//! This tool allows users to cut paths by drawing a line across them.
//! The tool shows a preview of the cutting line and intersection points.

#![allow(unused_variables)]

use crate::core::state::AppState;
#[allow(unused_imports)]
use crate::core::state::GlyphNavigation;
use crate::editing::selection::systems::AppStateChanged;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::{BezPath, PathEl, Point};

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "knife"
    }

    fn name(&self) -> &'static str {
        "Knife"
    }

    fn icon(&self) -> &'static str {
        "\u{E013}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }

    fn default_order(&self) -> i32 {
        110 // Advanced tool, later in toolbar
    }

    fn description(&self) -> &'static str {
        "Cut and slice paths"
    }

    fn update(&self, commands: &mut Commands) {
        info!("🔪 KNIFE_TOOL: update() called - setting knife mode active and input mode to Knife");
        commands.insert_resource(KnifeModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Knife);
    }

    fn on_enter(&self) {
        info!("Entered Knife tool");
    }

    fn on_exit(&self) {
        info!("Exited Knife tool");
    }
}

/// The state of the knife gesture
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum KnifeGestureState {
    /// Ready to start cutting
    #[default]
    Ready,
    /// Currently dragging a cut line
    Cutting { start: Vec2, current: Vec2 },
}

/// Resource to track the state of the knife tool
#[derive(Resource, Default)]
pub struct KnifeToolState {
    /// The current gesture state
    pub gesture: KnifeGestureState,
    /// Whether shift key is pressed (for axis-aligned cuts)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
}

impl KnifeToolState {
    pub fn new() -> Self {
        Self {
            gesture: KnifeGestureState::Ready,
            shift_locked: false,
            intersections: Vec::new(),
        }
    }

    /// Get the cutting line with axis locking if shift is pressed
    pub fn get_cutting_line(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture {
            KnifeGestureState::Cutting { start, current } => {
                let actual_end = if self.shift_locked {
                    // Apply axis constraint for shift key
                    let delta = current - start;
                    if delta.x.abs() > delta.y.abs() {
                        // Horizontal line
                        Vec2::new(current.x, start.y)
                    } else {
                        // Vertical line
                        Vec2::new(start.x, current.y)
                    }
                } else {
                    current
                };
                Some((start, actual_end))
            }
            KnifeGestureState::Ready => None,
        }
    }
}

/// Plugin for the knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeModeActive>()
            .init_resource::<KnifeToolState>()
            .add_systems(Startup, register_knife_tool)
            .add_systems(
                Update,
                (
                    manage_knife_mode_state,
                    render_knife_preview.after(manage_knife_mode_state),
                ),
            );
    }
}

fn register_knife_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(KnifeTool));
}

/// Handle mouse events for the knife tool
#[allow(clippy::too_many_arguments)]
pub fn handle_knife_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert cursor position to world coordinates
    if let Ok(world_position) =
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        // Update shift lock state
        knife_state.shift_locked = keyboard.pressed(KeyCode::ShiftLeft)
            || keyboard.pressed(KeyCode::ShiftRight);

        // Handle mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            knife_state.gesture = KnifeGestureState::Cutting {
                start: world_position,
                current: world_position,
            };
            knife_state.intersections.clear();
            info!("🔪 KNIFE_DEBUG: Started cutting at {:?}", world_position);
        }

        // Handle mouse movement during cutting
        if let KnifeGestureState::Cutting { start, .. } = knife_state.gesture {
            knife_state.gesture = KnifeGestureState::Cutting {
                start,
                current: world_position,
            };

            // Update intersections for preview
            update_intersections(&mut knife_state, &app_state);
            debug!("🔪 KNIFE_DEBUG: Dragging to {:?}", world_position);
        }

        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) {
            if let Some((start, end)) = knife_state.get_cutting_line() {
                // Perform the cut
                perform_cut(start, end, &mut app_state, &mut app_state_changed);
            }

            // Reset state
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.intersections.clear();
        }
    }
}

/// Handle keyboard events for the knife tool
pub fn handle_knife_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to cancel current cut
    if keyboard.just_pressed(KeyCode::Escape) {
        knife_state.gesture = KnifeGestureState::Ready;
        knife_state.intersections.clear();
        info!("Cancelled knife cut");
    }
}

/// System to manage knife mode activation/deactivation
pub fn manage_knife_mode_state(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
) {
    let is_knife_active = current_tool.get_current() == Some("knife");
    let current_mode = knife_mode.as_ref().map(|m| m.0).unwrap_or(false);
    
    if is_knife_active && !current_mode {
        // Knife tool is active but mode is not set - activate it
        commands.insert_resource(KnifeModeActive(true));
        info!("🔪 MANAGE_KNIFE_MODE: Activating knife mode");
    } else if !is_knife_active && current_mode {
        // Knife tool is not active but mode is set - deactivate it
        *knife_state = KnifeToolState::new();
        commands.insert_resource(KnifeModeActive(false));
        info!("🔪 MANAGE_KNIFE_MODE: Deactivating knife mode");
    }
}

/// Render the knife tool preview
pub fn render_knife_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    knife_consumer: Res<crate::systems::input_consumer::KnifeInputConsumer>,
    knife_mode: Option<Res<KnifeModeActive>>,
    camera_scale: Res<crate::rendering::camera_responsive::CameraResponsiveScale>,
    mut knife_entities: Local<Vec<Entity>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Debug log when gesture changes OR every few frames when knife tool is active
    let should_log = knife_consumer.is_changed() || 
        (current_tool.get_current() == Some("knife") && 
         knife_mode.as_ref().map(|m| m.0).unwrap_or(false));
         
    if should_log {
        info!("🔪 RENDER_KNIFE_PREVIEW: Gesture state! current_tool={:?}, knife_mode={:?}, gesture={:?}", 
              current_tool.get_current(), 
              knife_mode.as_ref().map(|m| m.0),
              knife_consumer.gesture);
    }
    
    // Clean up previous knife entities
    for entity in knife_entities.drain(..) {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if knife tool is active
    if current_tool.get_current() != Some("knife") {
        return;
    }

    // Also check knife mode resource
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Draw the cutting line
    if let Some((start, end)) = knife_consumer.get_cutting_line() {
        info!("🔪 RENDER_KNIFE_PREVIEW: Drawing cutting line from {:?} to {:?}", start, end);
        let line_color = theme.theme().knife_line_color();
        
        // Create dashed line effect by drawing multiple segments
        let direction = (end - start).normalize();
        let total_length = start.distance(end);
        let dash_length = theme.theme().knife_dash_length() * camera_scale.scale_factor;
        let gap_length = theme.theme().knife_gap_length() * camera_scale.scale_factor;
        let segment_length = dash_length + gap_length;
        let line_width = camera_scale.adjusted_line_width();

        let mut current_pos = 0.0;
        while current_pos < total_length {
            let dash_start = start + direction * current_pos;
            let dash_end_pos = (current_pos + dash_length).min(total_length);
            let dash_end = start + direction * dash_end_pos;

            let entity = spawn_knife_line_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                dash_start,
                dash_end,
                line_width,
                line_color,
                1.0, // z-order
            );
            knife_entities.push(entity);

            current_pos += segment_length;
        }

        // Draw start point (green circle)
        let start_color = theme.theme().knife_start_point_color();
        let point_size = camera_scale.adjusted_point_size(4.0);
        let point_entity = spawn_knife_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            start,
            point_size,
            start_color,
            1.1, // z-order above line
        );
        knife_entities.push(point_entity);
        
        // Draw end point (orange cross)
        let end_color = theme.theme().action_color();
        let cross_size = theme.theme().knife_cross_size() * camera_scale.scale_factor;
        let cross_width = camera_scale.adjusted_line_width();
        
        // Horizontal line of cross
        let cross_h_entity = spawn_knife_line_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec2::new(end.x - cross_size, end.y),
            Vec2::new(end.x + cross_size, end.y),
            cross_width,
            end_color,
            1.2, // z-order above everything
        );
        knife_entities.push(cross_h_entity);
        
        // Vertical line of cross
        let cross_v_entity = spawn_knife_line_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec2::new(end.x, end.y - cross_size),
            Vec2::new(end.x, end.y + cross_size),
            cross_width,
            end_color,
            1.2, // z-order above everything
        );
        knife_entities.push(cross_v_entity);
        
        info!("🔪 RENDER_KNIFE_PREVIEW: Created {} visual entities for knife preview", knife_entities.len());
    } else {
        // Log when we're not drawing
        if matches!(knife_consumer.gesture, crate::systems::input_consumer::KnifeGestureState::Ready) {
            debug!("🔪 RENDER_KNIFE_PREVIEW: No cutting line to draw (Ready state)");
        }
    }

    // Draw intersection points
    let intersection_color = theme.theme().knife_intersection_color();
    for &intersection in &knife_consumer.intersections {
        let point_size = camera_scale.adjusted_point_size(3.0);
        let intersection_entity = spawn_knife_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            intersection,
            point_size,
            intersection_color,
            1.2, // z-order above everything else
        );
        knife_entities.push(intersection_entity);
    }
}

/// Update intersection points for preview
fn update_intersections(
    knife_state: &mut KnifeToolState,
    app_state: &AppState,
) {
    knife_state.intersections.clear();

    if let Some((start, end)) = knife_state.get_cutting_line() {
        // For now, just add a simple intersection point for testing
        // TODO: Implement proper path intersection with FontData when needed
        let mid_point = (start + end) * 0.5;
        knife_state.intersections.push(mid_point);
    }
}

/// Perform the actual cut operation
fn perform_cut(
    start: Vec2,
    end: Vec2,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    info!("Performing knife cut from {:?} to {:?}", start, end);

    // For now, just log the cut operation - actual cutting would require
    // integration with the font data structures
    // TODO: Implement actual path cutting with FontData
    
    app_state_changed.write(AppStateChanged);
    info!("Knife cut completed (placeholder implementation)");
}

/// Spawn a line mesh for the knife tool
fn spawn_knife_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;

    // Create quad vertices for the line
    let vertices = vec![
        [start.x - perpendicular.x, start.y - perpendicular.y, z],
        [start.x + perpendicular.x, start.y + perpendicular.y, z],
        [end.x + perpendicular.x, end.y + perpendicular.y, z],
        [end.x - perpendicular.x, end.y - perpendicular.y, z],
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, z)),
    )).id()
}

/// Spawn a point (circle) mesh for the knife tool
fn spawn_knife_point_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    // Create circle using triangle fan
    let segments = 16;
    let mut vertices = vec![[position.x, position.y, z]]; // Center vertex
    let mut indices = Vec::new();

    // Create vertices around the circle
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = position.x + radius * angle.cos();
        let y = position.y + radius * angle.sin();
        vertices.push([x, y, z]);
    }

    // Create triangle indices
    for i in 0..segments {
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, z)),
    )).id()
}
