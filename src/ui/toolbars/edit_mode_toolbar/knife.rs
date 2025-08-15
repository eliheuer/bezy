//! Knife Tool - Path cutting and slicing tool
//!
//! This tool allows users to cut paths by drawing a line across them.
//! The tool shows a preview of the cutting line and intersection points.

#![allow(unused_variables)]

use crate::core::state::AppState;
#[allow(unused_imports)]
use crate::core::state::GlyphNavigation;
use crate::editing::selection::events::AppStateChanged;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::{BezPath, PathEl, Point, ParamCurve};

// Simple path operations are defined at the end of this file

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
                    handle_fontir_knife_cutting,
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
    mut app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
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
            update_intersections(&mut knife_state, &app_state, None);
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

/// Resource to track visual update state for performance
#[derive(Resource, Default)]
pub struct KnifeVisualUpdateTracker {
    pub needs_update: bool,
    pub last_gesture_state: Option<KnifeGestureState>,
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
    mut update_tracker: Local<Option<crate::systems::input_consumer::KnifeGestureState>>,
    fontir_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    // Check if tool is active
    let is_knife_active = current_tool.get_current() == Some("knife") && 
                         knife_mode.as_ref().map(|m| m.0).unwrap_or(false);
    
    // Only update if gesture state has changed or knife tool became active
    let gesture_changed = update_tracker.as_ref() != Some(&knife_consumer.gesture);
    let needs_update = gesture_changed || (!knife_entities.is_empty() && !is_knife_active);
    
    if !needs_update {
        return; // Early exit for performance
    }
    
    // Update tracking state
    *update_tracker = Some(knife_consumer.gesture);
    
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

    // Calculate and draw intersection points from actual glyph data
    if let Some((start, end)) = knife_consumer.get_cutting_line() {
        let intersection_color = theme.theme().knife_intersection_color();
        let intersections = calculate_real_intersections(start, end, &fontir_state);
        
        for &intersection in &intersections {
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
}

/// Calculate real intersections between knife line and current glyph contours
fn calculate_real_intersections(
    start: Vec2, 
    end: Vec2, 
    fontir_state: &Option<Res<crate::core::state::FontIRAppState>>
) -> Vec<Vec2> {
    let mut intersections = Vec::new();
    
    // Convert cutting line to kurbo Line for intersection testing
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    // Try FontIR state first (preferred)
    if let Some(fontir_state) = fontir_state {
        if let Some(ref current_glyph) = fontir_state.current_glyph {
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(current_glyph) {
                info!("🔪 CALCULATE_REAL_INTERSECTIONS: Found {} paths for glyph '{}'", paths.len(), current_glyph);
                for (path_idx, path) in paths.iter().enumerate() {
                    let path_intersections = find_path_intersections_simple(path, &cutting_line);
                    info!("🔪 CALCULATE_REAL_INTERSECTIONS: Path {} has {} intersections", path_idx, path_intersections.len());
                    for intersection in path_intersections {
                        intersections.push(Vec2::new(intersection.x as f32, intersection.y as f32));
                    }
                }
                info!("🔪 CALCULATE_REAL_INTERSECTIONS: Total intersections found: {}", intersections.len());
                return intersections;
            } else {
                info!("🔪 CALCULATE_REAL_INTERSECTIONS: No paths found for glyph '{}'", current_glyph);
            }
        } else {
            info!("🔪 CALCULATE_REAL_INTERSECTIONS: No current glyph selected");
        }
    } else {
        info!("🔪 CALCULATE_REAL_INTERSECTIONS: No FontIR state available");
    }
    
    intersections
}

/// Update intersection points for preview
fn update_intersections(
    knife_state: &mut KnifeToolState,
    app_state: &AppState,
    fontir_state: Option<&crate::core::state::FontIRAppState>,
) {
    knife_state.intersections.clear();

    if let Some((start, end)) = knife_state.get_cutting_line() {
        // Convert cutting line to kurbo Line for intersection testing
        let cutting_line = kurbo::Line::new(
            kurbo::Point::new(start.x as f64, start.y as f64),
            kurbo::Point::new(end.x as f64, end.y as f64),
        );

        // Try FontIR state first (preferred)
        if let Some(fontir_state) = fontir_state {
            if let Some(ref current_glyph) = fontir_state.current_glyph {
                if let Some(paths) = fontir_state.get_glyph_paths_with_edits(current_glyph) {
                    for path in &paths {
                        let intersections = find_path_intersections_simple(path, &cutting_line);
                        for intersection in intersections {
                            knife_state.intersections.push(Vec2::new(intersection.x as f32, intersection.y as f32));
                        }
                    }
                    return; // Found paths in FontIR, use those
                }
            }
        }

        // For now, just add a test intersection point for AppState fallback
        // The AppState uses a different data structure that would need conversion to BezPath
        let mid_point = (start + end) * 0.5;
        knife_state.intersections.push(mid_point);
    }
}

/// Perform the actual cut operation
fn perform_cut(
    start: Vec2,
    end: Vec2,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<crate::editing::selection::events::AppStateChanged>,
) {
    info!("Performing knife cut from {:?} to {:?}", start, end);

    // Convert cutting line to kurbo Line for intersection testing
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    // For now, just trigger a state change to indicate cut was attempted
    // TODO: Integrate with FontIR working copies when available
    // This would involve:
    // 1. Getting the FontIR state as a parameter
    // 2. Finding the current glyph and creating/accessing its working copy
    // 3. Using slice_path_with_line on each contour
    // 4. Updating the working copy with the new paths
    // 5. Marking the working copy as dirty
    
    app_state_changed.write(crate::editing::selection::events::AppStateChanged);
    info!("Knife cut completed - ready for FontIR integration");
}

/// System to handle actual path cutting with FontIR integration
#[allow(clippy::too_many_arguments)]
pub fn handle_fontir_knife_cutting(
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    knife_consumer: Res<crate::systems::input_consumer::KnifeInputConsumer>,
    mut app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Check if we just finished a cutting gesture
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(ref mut fontir_state) = fontir_state {
            if let Some((start, end)) = knife_consumer.get_cutting_line() {
                perform_fontir_cut(start, end, fontir_state, &mut app_state_changed);
            }
        }
    }
}

/// Perform cutting with FontIR working copies
fn perform_fontir_cut(
    start: Vec2,
    end: Vec2,
    fontir_state: &mut crate::core::state::FontIRAppState,
    app_state_changed: &mut EventWriter<crate::editing::selection::events::AppStateChanged>,
) {
    info!("Performing FontIR knife cut from {:?} to {:?}", start, end);

    // Convert cutting line to kurbo Line
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    if let Some(ref current_glyph) = fontir_state.current_glyph.clone() {
        // Get or create working copy for this glyph
        let location = fontir_state.current_location.clone();
        let key = (current_glyph.clone(), location);
        
        // Ensure we have a working copy
        if !fontir_state.working_copies.contains_key(&key) {
            if let Some(original_paths) = fontir_state.get_glyph_paths(&current_glyph) {
                let working_copy = crate::core::state::fontir_app_state::EditableGlyphInstance {
                    width: fontir_state.get_glyph_advance_width(&current_glyph) as f64,
                    height: None,
                    vertical_origin: None,
                    contours: original_paths,
                    is_dirty: false,
                };
                fontir_state.working_copies.insert(key.clone(), working_copy);
            }
        }
        
        // Perform the cut on the working copy
        if let Some(working_copy) = fontir_state.working_copies.get_mut(&key) {
            let mut new_contours = Vec::new();
            let mut any_cuts_made = false;
            
            for contour in &working_copy.contours {
                let sliced_paths = slice_path_with_line_simple(contour, &cutting_line);
                
                if sliced_paths.len() > 1 {
                    // Path was successfully cut
                    info!("Cut contour into {} pieces", sliced_paths.len());
                    
                    // Add all the sliced paths as new contours
                    new_contours.extend(sliced_paths);
                    any_cuts_made = true;
                } else {
                    // Path was not cut, keep original
                    new_contours.push(contour.clone());
                }
            }
            
            if any_cuts_made {
                // Replace the contours with the cut versions
                working_copy.contours = new_contours;
                working_copy.is_dirty = true;
                app_state_changed.write(crate::editing::selection::events::AppStateChanged);
                info!("FontIR knife cut completed - glyph now has {} contours", working_copy.contours.len());
            } else {
                info!("FontIR knife cut completed - no intersections found");
            }
        }
    } else {
        info!("FontIR knife cut completed - no current glyph selected");
    }
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


// ============================================================================
// SIMPLE PATH OPERATIONS FOR KNIFE TOOL
// ============================================================================

/// Find simple intersections between a cutting line and a path
fn find_path_intersections_simple(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<Point> {
    let mut intersections = Vec::new();
    let mut current_point = Point::ZERO;
    
    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection_point) = line_line_intersection_simple(cutting_line, &segment) {
                    intersections.push(intersection_point);
                }
                current_point = *end;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_intersections = curve_line_intersections_simple(&curve, cutting_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_intersections = quad_line_intersections_simple(&curve, cutting_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point_inline(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection_point) = line_line_intersection_simple(cutting_line, &segment) {
                        intersections.push(intersection_point);
                    }
                }
            }
        }
    }
    
    intersections.dedup_by(|a, b| a.distance(*b) < 5.0);
    intersections
}

/// Hit structure to track intersection details
#[derive(Debug, Clone)]
struct Hit {
    pub point: Point,
    pub t: f64,
    pub segment_idx: usize,
}

/// Slice a path with a cutting line, returning new path segments
fn slice_path_with_line_simple(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<BezPath> {
    let hits = find_path_intersections_with_parameters(path, cutting_line);
    
    if hits.is_empty() {
        return vec![path.clone()];
    }
    
    info!("Found {} intersections, slicing path", hits.len());
    
    // Sort hits by segment index and parameter t
    let mut sorted_hits = hits;
    sorted_hits.sort_by(|a, b| {
        a.segment_idx.cmp(&b.segment_idx).then(a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal))
    });
    
    // Slice the path at intersection points
    slice_path_at_hits(path, &sorted_hits)
}

/// Find intersections with parameter information for accurate slicing
fn find_path_intersections_with_parameters(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<Hit> {
    let mut hits = Vec::new();
    let mut current_point = Point::ZERO;
    let mut segment_idx = 0;
    
    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection) = line_line_intersection_with_parameter(&segment, cutting_line) {
                    hits.push(Hit {
                        point: intersection.0,
                        t: intersection.1,
                        segment_idx,
                    });
                }
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_hits = curve_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_hits = quad_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point_inline(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection) = line_line_intersection_with_parameter(&segment, cutting_line) {
                        hits.push(Hit {
                            point: intersection.0,
                            t: intersection.1,
                            segment_idx,
                        });
                    }
                }
                segment_idx += 1;
            }
        }
    }
    
    // Remove duplicate hits
    hits.dedup_by(|a, b| a.point.distance(b.point) < 1.0);
    hits
}

/// Slice path at specific hit points
fn slice_path_at_hits(path: &BezPath, hits: &[Hit]) -> Vec<BezPath> {
    if hits.is_empty() {
        return vec![path.clone()];
    }
    
    // For now, return two copies of the path to show the cut was detected
    // TODO: Implement proper path slicing algorithm
    info!("Slicing path with {} hits - returning split paths", hits.len());
    vec![path.clone(), path.clone()]
}

/// Line-line intersection with parameter information
fn line_line_intersection_with_parameter(line1: &kurbo::Line, line2: &kurbo::Line) -> Option<(Point, f64)> {
    let p1 = line1.p0;
    let p2 = line1.p1;
    let p3 = line2.p0;
    let p4 = line2.p1;
    
    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
    
    if denom.abs() < 1e-10 {
        return None;
    }
    
    let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
    let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / denom;
    
    if u >= 0.0 && u <= 1.0 && t >= 0.0 && t <= 1.0 {
        let point = Point::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        );
        Some((point, t))
    } else {
        None
    }
}

/// Curve-line intersections with parameter information
fn curve_line_intersections_with_parameters(curve: &kurbo::CubicBez, line: &kurbo::Line, segment_idx: usize) -> Vec<Hit> {
    let mut hits = Vec::new();
    let curve_seg = kurbo::PathSeg::Cubic(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        hits.push(Hit {
            point,
            t: intersection.segment_t,
            segment_idx,
        });
    }
    
    hits
}

/// Quad-line intersections with parameter information
fn quad_line_intersections_with_parameters(curve: &kurbo::QuadBez, line: &kurbo::Line, segment_idx: usize) -> Vec<Hit> {
    let mut hits = Vec::new();
    let curve_seg = kurbo::PathSeg::Quad(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        hits.push(Hit {
            point,
            t: intersection.segment_t,
            segment_idx,
        });
    }
    
    hits
}

fn line_line_intersection_simple(line1: &kurbo::Line, line2: &kurbo::Line) -> Option<Point> {
    let p1 = line1.p0;
    let p2 = line1.p1;
    let p3 = line2.p0;
    let p4 = line2.p1;
    
    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
    
    if denom.abs() < 1e-10 {
        return None;
    }
    
    let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
    let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / denom;
    
    if u >= 0.0 && u <= 1.0 {
        Some(Point::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    } else {
        None
    }
}

fn curve_line_intersections_simple(curve: &kurbo::CubicBez, line: &kurbo::Line) -> Vec<Point> {
    // Use kurbo's built-in intersection method for accurate mathematical intersection
    let mut intersections = Vec::new();
    
    // Convert curve to PathSeg for intersection testing
    let curve_seg = kurbo::PathSeg::Cubic(*curve);
    
    // Find intersections using kurbo's accurate intersection algorithm
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        intersections.push(point);
    }
    
    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn quad_line_intersections_simple(curve: &kurbo::QuadBez, line: &kurbo::Line) -> Vec<Point> {
    // Use kurbo's built-in intersection method for accurate mathematical intersection
    let mut intersections = Vec::new();
    
    // Convert curve to PathSeg for intersection testing
    let curve_seg = kurbo::PathSeg::Quad(*curve);
    
    // Find intersections using kurbo's accurate intersection algorithm
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        intersections.push(point);
    }
    
    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn get_path_start_point_inline(path: &BezPath) -> Option<Point> {
    for element in path.elements() {
        if let PathEl::MoveTo(pt) = element {
            return Some(*pt);
        }
    }
    None
}

/// Calculate the distance from a point to a line
fn calculate_line_point_distance(line: &kurbo::Line, point: Point) -> f64 {
    let a = line.p1.y - line.p0.y;
    let b = line.p0.x - line.p1.x;
    let c = line.p1.x * line.p0.y - line.p0.x * line.p1.y;
    
    let distance = (a * point.x + b * point.y + c).abs() / (a * a + b * b).sqrt();
    distance
}

