//! Shapes Tool - Geometric shape drawing tool
//!
//! This tool allows users to draw basic geometric shapes like rectangles,
//! ellipses, and rounded rectangles by clicking and dragging.

#![allow(dead_code)]

use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::BezySettings;
use crate::editing::selection::systems::AppStateChanged;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

/// Resource to track if shapes mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct ShapesModeActive(pub bool);

pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "shapes"
    }
    
    fn name(&self) -> &'static str {
        "Shapes"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E016}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('r')
    }
    
    fn default_order(&self) -> i32 {
        30 // After pen, before text
    }
    
    fn description(&self) -> &'static str {
        "Draw geometric shapes"
    }
    
    fn update(&self, commands: &mut Commands) {
        // Activate shapes mode
        commands.insert_resource(ShapesModeActive(true));
        debug!("ShapesTool::update() called - activating shapes mode");
    }
    
    fn on_enter(&self) {
        info!("✅ SHAPES TOOL: Entered Shapes tool");
    }
    
    fn on_exit(&self) {
        info!("❌ SHAPES TOOL: Exited Shapes tool");
    }
}

/// Types of shapes that can be drawn
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum ShapeType {
    #[default]
    Rectangle,
    Ellipse,
    RoundedRectangle,
}

/// Resource to track the currently selected shape type
#[derive(Resource, Default)]
pub struct CurrentShapeType(pub ShapeType);

/// Active drawing state for shapes
#[derive(Resource, Default)]
pub struct ActiveShapeDrawing {
    pub is_drawing: bool,
    pub shape_type: ShapeType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
}

impl ActiveShapeDrawing {
    /// Get the rectangle from the current drawing state
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start_position, self.current_position) {
            let min_x = start.x.min(current.x);
            let min_y = start.y.min(current.y);
            let max_x = start.x.max(current.x);
            let max_y = start.y.max(current.y);

            Some(Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        } else {
            None
        }
    }
}

/// Resource to store the current corner radius for rounded rectangles
#[derive(Resource)]
pub struct CurrentCornerRadius(pub f32);

impl Default for CurrentCornerRadius {
    fn default() -> Self {
        Self(10.0)
    }
}

/// Plugin for the shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapesModeActive>()
           .init_resource::<CurrentShapeType>()
           .init_resource::<ActiveShapeDrawing>()
           .init_resource::<CurrentCornerRadius>()
           .add_systems(Startup, register_shapes_tool)
           .add_systems(
               Update,
               (
                   handle_shape_mouse_events,
                   render_active_shape_drawing,
                   reset_shapes_mode_when_inactive,
               ),
           );
    }
}

fn register_shapes_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(ShapesTool));
}

/// Handle mouse events for shape drawing
pub fn handle_shape_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_shape_type: Res<CurrentShapeType>,
    mut active_drawing: ResMut<ActiveShapeDrawing>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mut app_state: ResMut<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    corner_radius: Res<CurrentCornerRadius>,
    shapes_mode: Option<Res<ShapesModeActive>>,
) {
    // Only handle input if shapes tool is active
    if let Some(shapes_mode) = shapes_mode {
        if !shapes_mode.0 {
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
    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        // Apply grid snapping
        let settings = BezySettings::default();
        let snapped_position = settings.apply_grid_snap(world_position);
        
        // Handle mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            debug!("SHAPES TOOL: Starting to draw {:?} at ({:.1}, {:.1})", current_shape_type.0, snapped_position.x, snapped_position.y);
            active_drawing.is_drawing = true;
            active_drawing.shape_type = current_shape_type.0;
            active_drawing.start_position = Some(snapped_position);
            active_drawing.current_position = Some(snapped_position);
        }
        
        // Handle mouse movement during drawing
        if active_drawing.is_drawing {
            active_drawing.current_position = Some(snapped_position);
        }
        
        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) && active_drawing.is_drawing {
            if let Some(rect) = active_drawing.get_rect() {
                debug!("SHAPES TOOL: Completing {:?} shape with rect: ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       active_drawing.shape_type, rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                
                // Create the shape in the current glyph
                create_shape(
                    rect,
                    active_drawing.shape_type,
                    corner_radius.0,
                    &glyph_navigation,
                    &mut app_state,
                    &mut app_state_changed,
                );
            }
            
            // Reset drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
            active_drawing.current_position = None;
        }
    }
}

/// Render the shape being drawn
pub fn render_active_shape_drawing(
    mut gizmos: Gizmos,
    active_drawing: Res<ActiveShapeDrawing>,
    shapes_mode: Option<Res<ShapesModeActive>>,
) {
    // Only render if shapes tool is active
    if let Some(shapes_mode) = shapes_mode {
        if !shapes_mode.0 {
            return;
        }
    } else {
        return;
    }

    if !active_drawing.is_drawing {
        return;
    }

    if let Some(rect) = active_drawing.get_rect() {
        let preview_color = Color::srgba(0.8, 0.8, 0.8, 0.6);
        
        match active_drawing.shape_type {
            ShapeType::Rectangle | ShapeType::RoundedRectangle => {
                draw_dashed_rectangle(&mut gizmos, rect, preview_color);
            }
            ShapeType::Ellipse => {
                draw_dashed_ellipse(&mut gizmos, rect, preview_color);
            }
        }
    }
}

/// Reset shapes mode when another tool is selected
pub fn reset_shapes_mode_when_inactive(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut active_drawing: ResMut<ActiveShapeDrawing>,
) {
    if current_tool.get_current() != Some("shapes") {
        // Cancel any active drawing
        if active_drawing.is_drawing {
            debug!("SHAPES TOOL: Cancelling active drawing - switching tools");
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
            active_drawing.current_position = None;
        }
        
        // Mark shapes mode as inactive
        commands.insert_resource(ShapesModeActive(false));
    }
}

/// Create a shape in the current glyph
fn create_shape(
    rect: Rect,
    shape_type: ShapeType,
    corner_radius: f32,
    glyph_navigation: &GlyphNavigation,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    let Some(glyph_name) = glyph_navigation.find_glyph(app_state) else {
        warn!("No current glyph selected for shape creation");
        return;
    };
    
    // Create contour points based on shape type
    let points = match shape_type {
        ShapeType::Rectangle => create_rectangle_points(rect),
        ShapeType::Ellipse => create_ellipse_points(rect),
        ShapeType::RoundedRectangle => create_rounded_rectangle_points(rect, corner_radius),
    };
    
    // Add the contour to the glyph
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get_mut(&glyph_name) {
        if glyph_data.outline.is_none() {
            glyph_data.outline = Some(crate::core::state::OutlineData {
                contours: Vec::new(),
            });
        }
        
        if let Some(outline) = &mut glyph_data.outline {
            outline.contours.push(crate::core::state::ContourData { points });
            
            info!("Created {} shape in glyph '{}'", 
                  match shape_type {
                      ShapeType::Rectangle => "rectangle",
                      ShapeType::Ellipse => "ellipse", 
                      ShapeType::RoundedRectangle => "rounded rectangle",
                  }, 
                  glyph_name);
            
            app_state_changed.write(AppStateChanged);
        }
    }
}

/// Create points for a rectangle
fn create_rectangle_points(rect: Rect) -> Vec<crate::core::state::PointData> {
    vec![
        crate::core::state::PointData {
            x: rect.min.x as f64,
            y: rect.min.y as f64,
            point_type: crate::core::state::PointTypeData::Move,
        },
        crate::core::state::PointData {
            x: rect.max.x as f64,
            y: rect.min.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
        crate::core::state::PointData {
            x: rect.max.x as f64,
            y: rect.max.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
        crate::core::state::PointData {
            x: rect.min.x as f64,
            y: rect.max.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
    ]
}

/// Create points for an ellipse (simplified as octagon)
fn create_ellipse_points(rect: Rect) -> Vec<crate::core::state::PointData> {
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;
    
    let mut points = Vec::new();
    
    // Create 8 points for a simplified ellipse
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
        let x = center_x + radius_x * angle.cos();
        let y = center_y + radius_y * angle.sin();
        
        points.push(crate::core::state::PointData {
            x: x as f64,
            y: y as f64,
            point_type: if i == 0 { 
                crate::core::state::PointTypeData::Move 
            } else { 
                crate::core::state::PointTypeData::Line 
            },
        });
    }
    
    points
}

/// Create points for a rounded rectangle (simplified)
fn create_rounded_rectangle_points(rect: Rect, _radius: f32) -> Vec<crate::core::state::PointData> {
    // For now, just create a regular rectangle
    // TODO: Implement proper rounded corners
    create_rectangle_points(rect)
}

/// Draw a dashed rectangle preview
fn draw_dashed_rectangle(gizmos: &mut Gizmos, rect: Rect, color: Color) {
    let corners = [
        Vec2::new(rect.min.x, rect.min.y),
        Vec2::new(rect.max.x, rect.min.y),
        Vec2::new(rect.max.x, rect.max.y),
        Vec2::new(rect.min.x, rect.max.y),
    ];
    
    for i in 0..4 {
        let start = corners[i];
        let end = corners[(i + 1) % 4];
        draw_dashed_line(gizmos, start, end, 8.0, 4.0, color);
    }
}

/// Draw a dashed ellipse preview (simplified as octagon)
fn draw_dashed_ellipse(gizmos: &mut Gizmos, rect: Rect, color: Color) {
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;
    
    let mut points = Vec::new();
    
    // Create 16 points for smoother ellipse preview
    for i in 0..16 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 16.0;
        let x = center_x + radius_x * angle.cos();
        let y = center_y + radius_y * angle.sin();
        points.push(Vec2::new(x, y));
    }
    
    for i in 0..16 {
        let start = points[i];
        let end = points[(i + 1) % 16];
        draw_dashed_line(gizmos, start, end, 4.0, 2.0, color);
    }
}

/// Draw a dashed line
fn draw_dashed_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, dash_length: f32, gap_length: f32, color: Color) {
    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;
    
    let mut current_pos = 0.0;
    
    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;
        
        gizmos.line_2d(dash_start, dash_end, color);
        
        current_pos += segment_length;
    }
} 