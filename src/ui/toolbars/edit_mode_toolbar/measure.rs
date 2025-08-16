use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;
use kurbo::ParamCurve;

/// Resource to track if measure mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct MeasureModeActive(pub bool);

pub struct MeasureTool;

impl EditTool for MeasureTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "measure"
    }

    fn name(&self) -> &'static str {
        "Measure"
    }

    fn icon(&self) -> &'static str {
        "\u{E015}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }

    fn default_order(&self) -> i32 {
        60 // After text tool, before pan
    }

    fn description(&self) -> &'static str {
        "Measure distances and dimensions"
    }

    fn update(&self, commands: &mut Commands) {
        info!("📏 MEASURE_TOOL: update() called - setting measure mode active and input mode to Measure");
        commands.insert_resource(MeasureModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Measure);
    }

    fn on_enter(&self) {
        info!("Entered Measure tool");
    }

    fn on_exit(&self) {
        info!("Exited Measure tool");
    }
}

/// Plugin for the Measure tool
pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureModeActive>()
            .add_systems(Startup, register_measure_tool)
            .add_systems(
                Update,
                (
                    manage_measure_mode_state,
                    render_measure_preview.after(manage_measure_mode_state),
                ),
            );
    }
}

fn register_measure_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(MeasureTool));
}

/// System to manage measure mode activation/deactivation
pub fn manage_measure_mode_state(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    measure_mode: Option<Res<MeasureModeActive>>,
) {
    let is_measure_active = current_tool.get_current() == Some("measure");
    let current_mode = measure_mode.as_ref().map(|m| m.0).unwrap_or(false);
    
    if is_measure_active && !current_mode {
        // Measure tool is active but mode is not set - activate it
        commands.insert_resource(MeasureModeActive(true));
        info!("📏 MANAGE_MEASURE_MODE: Activating measure mode");
    } else if !is_measure_active && current_mode {
        // Measure tool is not active but mode is set - deactivate it
        commands.insert_resource(MeasureModeActive(false));
        info!("📏 MANAGE_MEASURE_MODE: Deactivating measure mode");
    }
}

/// Render the measure tool preview
#[allow(clippy::too_many_arguments)]
pub fn render_measure_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<bevy::render::mesh::Mesh>>,
    mut materials: ResMut<Assets<bevy::sprite::ColorMaterial>>,
    measure_consumer: Res<crate::systems::input_consumer::MeasureInputConsumer>,
    measure_mode: Option<Res<MeasureModeActive>>,
    camera_scale: Res<crate::rendering::camera_responsive::CameraResponsiveScale>,
    mut measure_entities: Local<Vec<Entity>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut update_tracker: Local<Option<crate::systems::input_consumer::MeasureGestureState>>,
    fontir_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    // Check if tool is active
    let is_measure_active = current_tool.get_current() == Some("measure") && 
                           measure_mode.as_ref().map(|m| m.0).unwrap_or(false);
    
    // Debug current state
    info!("📏 RENDER_MEASURE_PREVIEW: current_tool={:?}, measure_mode={:?}, is_measure_active={}", 
          current_tool.get_current(), measure_mode.as_ref().map(|m| m.0), is_measure_active);
    info!("📏 RENDER_MEASURE_PREVIEW: measure_consumer.gesture={:?}", measure_consumer.gesture);
    
    // Only update if gesture state has changed, measure tool became active, or cleanup needed
    let gesture_changed = update_tracker.as_ref() != Some(&measure_consumer.gesture);
    let cleanup_needed = !measure_entities.is_empty() && !is_measure_active;
    let tool_became_active = is_measure_active && measure_entities.is_empty();
    let needs_update = gesture_changed || cleanup_needed || tool_became_active;
    
    info!("📏 RENDER_MEASURE_PREVIEW: gesture_changed={}, cleanup_needed={}, tool_became_active={}, needs_update={}, measure_entities.len()={}", 
          gesture_changed, cleanup_needed, tool_became_active, needs_update, measure_entities.len());
    
    if !needs_update {
        return; // Early exit for performance
    }
    
    // Update tracking state
    *update_tracker = Some(measure_consumer.gesture);
    
    // Clean up previous measure entities
    for entity in measure_entities.drain(..) {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if measure tool is active
    if current_tool.get_current() != Some("measure") {
        return;
    }

    // Also check measure mode resource
    if let Some(measure_mode) = measure_mode {
        if !measure_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Draw the measuring line
    if let Some((start, end)) = measure_consumer.get_measuring_line() {
        debug!("📏 RENDER_MEASURE_PREVIEW: Drawing measuring line from {:?} to {:?}", start, end);
        let line_color = theme.theme().action_color(); // Use action color for measure line
        let line_width = camera_scale.adjusted_line_width();
        
        // Create solid line for measuring
        let line_entity = spawn_measure_line_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            start,
            end,
            line_width,
            line_color,
            18.0, // z-order (below intersection points but above other elements)
        );
        measure_entities.push(line_entity);

        // Draw start point (green circle)
        let start_color = theme.theme().selected_color();
        let point_size = camera_scale.adjusted_point_size(4.0);
        let point_entity = spawn_measure_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            start,
            point_size,
            start_color,
            19.0, // z-order above line but below intersection points
        );
        measure_entities.push(point_entity);
        
        // Draw end point (orange circle)
        let end_color = theme.theme().action_color();
        let end_point_entity = spawn_measure_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            end,
            point_size,
            end_color,
            19.0, // z-order above line but below intersection points
        );
        measure_entities.push(end_point_entity);
        
        debug!("📏 RENDER_MEASURE_PREVIEW: Created {} visual entities for measure preview", measure_entities.len());
    } else {
        // Log when we're not drawing
        if matches!(measure_consumer.gesture, crate::systems::input_consumer::MeasureGestureState::Ready) {
            debug!("📏 RENDER_MEASURE_PREVIEW: No measuring line to draw (Ready state)");
        }
    }

    // Calculate and draw intersection points from actual glyph data
    if let Some((start, end)) = measure_consumer.get_measuring_line() {
        let intersections = calculate_measure_intersections(start, end, &fontir_state);
        
        let intersection_color = theme.theme().helper_color(); // Use helper color for intersection points
        
        for &intersection in &intersections {
            let cross_size = theme.theme().knife_cross_size() * camera_scale.scale_factor;
            let cross_width = camera_scale.adjusted_line_width();
            
            // Create + mark with two perpendicular lines
            // Horizontal line
            let horizontal_entity = spawn_measure_line_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(intersection.x - cross_size, intersection.y),
                Vec2::new(intersection.x + cross_size, intersection.y),
                cross_width,
                intersection_color,
                20.0, // z-order above everything else
            );
            measure_entities.push(horizontal_entity);
            
            // Vertical line
            let vertical_entity = spawn_measure_line_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(intersection.x, intersection.y - cross_size),
                Vec2::new(intersection.x, intersection.y + cross_size),
                cross_width,
                intersection_color,
                20.0, // z-order above everything else
            );
            measure_entities.push(vertical_entity);
        }
        
        // Calculate and display distance measurement
        if intersections.len() >= 2 {
            let distance = intersections[0].distance(intersections[1]);
            let _midpoint = (intersections[0] + intersections[1]) * 0.5;
            
            // TODO: Add text label rendering for distance at midpoint
            info!("📏 MEASURE: Distance between intersection points: {:.1} units", distance);
        }
    }
}

/// Calculate intersections between measure line and current glyph contours
fn calculate_measure_intersections(
    start: Vec2, 
    end: Vec2, 
    fontir_state: &Option<Res<crate::core::state::FontIRAppState>>
) -> Vec<Vec2> {
    let mut intersections = Vec::new();
    
    // Convert measuring line to kurbo Line for intersection testing
    let measuring_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    // Try FontIR state first (preferred)
    if let Some(fontir_state) = fontir_state {
        if let Some(ref current_glyph) = fontir_state.current_glyph {
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(current_glyph) {
                debug!("📏 CALCULATE_MEASURE_INTERSECTIONS: Found {} paths for glyph '{}'", paths.len(), current_glyph);
                for path in &paths {
                    let path_intersections = find_measure_path_intersections(path, &measuring_line);
                    for intersection in path_intersections {
                        intersections.push(Vec2::new(intersection.x as f32, intersection.y as f32));
                    }
                }
                debug!("📏 CALCULATE_MEASURE_INTERSECTIONS: Total intersections found: {}", intersections.len());
                return intersections;
            } else {
                info!("📏 CALCULATE_MEASURE_INTERSECTIONS: No paths found for glyph '{}'", current_glyph);
            }
        } else {
            info!("📏 CALCULATE_MEASURE_INTERSECTIONS: No current glyph selected");
        }
    } else {
        info!("📏 CALCULATE_MEASURE_INTERSECTIONS: No FontIR state available");
    }
    
    intersections
}

/// Find intersections between a measuring line and a path (reuse knife tool logic)
fn find_measure_path_intersections(path: &kurbo::BezPath, measuring_line: &kurbo::Line) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let mut current_point = kurbo::Point::ZERO;
    
    for element in path.elements() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            kurbo::PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection_point) = line_line_intersection_simple(measuring_line, &segment) {
                    intersections.push(intersection_point);
                }
                current_point = *end;
            }
            kurbo::PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_intersections = curve_line_intersections_simple(&curve, measuring_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            kurbo::PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_intersections = quad_line_intersections_simple(&curve, measuring_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            kurbo::PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection_point) = line_line_intersection_simple(measuring_line, &segment) {
                        intersections.push(intersection_point);
                    }
                }
            }
        }
    }
    
    intersections.dedup_by(|a, b| a.distance(*b) < 5.0);
    intersections
}

/// Helper functions from knife tool (reused for measure tool)
fn line_line_intersection_simple(line1: &kurbo::Line, line2: &kurbo::Line) -> Option<kurbo::Point> {
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
        Some(kurbo::Point::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    } else {
        None
    }
}

fn curve_line_intersections_simple(curve: &kurbo::CubicBez, line: &kurbo::Line) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let curve_seg = kurbo::PathSeg::Cubic(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        let point = curve.eval(intersection.segment_t);
        intersections.push(point);
    }
    
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn quad_line_intersections_simple(curve: &kurbo::QuadBez, line: &kurbo::Line) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let curve_seg = kurbo::PathSeg::Quad(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);
    
    for intersection in curve_intersections {
        let point = curve.eval(intersection.segment_t);
        intersections.push(point);
    }
    
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn get_path_start_point(path: &kurbo::BezPath) -> Option<kurbo::Point> {
    for element in path.elements() {
        if let kurbo::PathEl::MoveTo(pt) = element {
            return Some(*pt);
        }
    }
    None
}

/// Spawn a line mesh for the measure tool
fn spawn_measure_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: bevy::color::Color,
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

    let mut mesh = bevy::render::mesh::Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(bevy::sprite::ColorMaterial::from(color));

    commands.spawn((
        bevy::render::mesh::Mesh2d(mesh_handle),
        bevy::sprite::MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, z)),
    )).id()
}

/// Spawn a point (circle) mesh for the measure tool
fn spawn_measure_point_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: bevy::color::Color,
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

    let mut mesh = bevy::render::mesh::Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(bevy::sprite::ColorMaterial::from(color));

    commands.spawn((
        bevy::render::mesh::Mesh2d(mesh_handle),
        bevy::sprite::MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(0.0, 0.0, z)),
    )).id()
}
