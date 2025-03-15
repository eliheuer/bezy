use crate::edit_mode_toolbar::primitives::tools::ellipse::EllipsePrimitive;
use crate::edit_mode_toolbar::primitives::tools::rectangle::RectanglePrimitive;
use crate::edit_mode_toolbar::primitives::tools::rounded_rectangle::RoundedRectanglePrimitive;
use crate::edit_mode_toolbar::CurrentPrimitiveType;
use crate::edit_mode_toolbar::PrimitiveType;
use bevy::prelude::*;
use kurbo::Shape;

// Common components and systems for primitives
#[derive(Component)]
#[allow(dead_code)]
pub struct PrimitiveShape;

// Common trait for all primitive shapes
#[allow(dead_code)]
pub trait PrimitiveShapeTool: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn update(&self, commands: &mut Commands);
    fn on_enter(&self);
    fn on_exit(&self);

    // Drawing methods
    fn begin_draw(&mut self, position: Vec2);
    fn update_draw(&mut self, position: Vec2);
    fn end_draw(&mut self, position: Vec2);
    fn cancel_draw(&mut self);

    // Method to access shift_locked property
    fn set_shift_locked(&mut self, locked: bool);
}

// Active drawing state for primitives
#[derive(Resource)]
pub struct ActivePrimitiveDrawing {
    pub is_drawing: bool,
    pub tool_type: PrimitiveType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
}

impl Default for ActivePrimitiveDrawing {
    fn default() -> Self {
        Self {
            is_drawing: false,
            tool_type: PrimitiveType::Rectangle,
            start_position: None,
            current_position: None,
        }
    }
}

impl ActivePrimitiveDrawing {
    /// Get the rectangle from the current drawing state
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) =
            (self.start_position, self.current_position)
        {
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

// Systems for handling primitive shape drawing
pub fn handle_primitive_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    current_primitive_type: Res<CurrentPrimitiveType>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut active_drawing: ResMut<ActivePrimitiveDrawing>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
    mut app_state: ResMut<crate::data::AppState>,
    cli_args: Res<crate::cli::CliArgs>,
    corner_radius: Res<
        crate::edit_mode_toolbar::primitives::ui::CurrentCornerRadius,
    >,
    ui_state: Res<crate::edit_mode_toolbar::primitives::ui::UiInteractionState>,
) {
    // Only handle events when in primitives mode
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Primitives {
        return;
    }

    // Don't process drawing events when interacting with UI
    if ui_state.is_interacting_with_ui {
        debug!("Skipping primitive drawing - UI is being interacted with");
        return;
    }

    // Get the current window and cursor position
    // Find the primary camera or any camera that is the main 2D camera
    let camera_entity = camera_q.iter().find(|(camera, _)| {
        // Just use any active camera for now
        camera.is_active
    });

    // Get the main window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(e) => {
            warn!("Could not get window for primitive drawing: {}", e);
            return;
        }
    };

    // Only proceed if we found a valid camera
    let Some((camera, camera_transform)) = camera_entity else {
        warn!("No primary camera found for primitive drawing");
        return;
    };

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            let world_position = match camera
                .viewport_to_world_2d(camera_transform, cursor_pos)
            {
                Ok(pos) => pos,
                Err(e) => {
                    debug!(
                        "Error converting cursor to world position: {:?}",
                        e
                    );
                    continue;
                }
            };

            active_drawing.current_position = Some(world_position);

            // If we're already drawing, update the current drawing
            if active_drawing.is_drawing {
                let mut tool = get_primitive_tool(current_primitive_type.0);
                tool.update_draw(world_position);
            }
        }
    }

    // Handle mouse button input
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = active_drawing.current_position {
            // Start a new drawing
            active_drawing.is_drawing = true;
            active_drawing.tool_type = current_primitive_type.0;
            active_drawing.start_position = Some(cursor_pos);

            // Create and initialize the appropriate primitive tool
            let mut tool = get_primitive_tool(current_primitive_type.0);
            tool.begin_draw(cursor_pos);
            debug!(
                "Started drawing primitive: {:?} at {:?}",
                active_drawing.tool_type, cursor_pos
            );
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        if active_drawing.is_drawing {
            if let Some(cursor_pos) = active_drawing.current_position {
                // Get the current tool
                let mut tool = get_primitive_tool(active_drawing.tool_type);

                // Finish the drawing
                tool.end_draw(cursor_pos);
                debug!(
                    "Finished drawing primitive: {:?}",
                    active_drawing.tool_type
                );

                // Create and add contour based on the tool type
                if let Some(rect) = active_drawing.get_rect() {
                    // Get the glyph name first
                    if let Some(glyph_name) =
                        cli_args.find_glyph(&app_state.workspace.font.ufo)
                    {
                        let glyph_name = glyph_name.clone(); // Clone the glyph name

                        // Get mutable access to the font
                        let font_obj = app_state.workspace.font_mut();

                        // Get the current glyph
                        if let Some(default_layer) =
                            font_obj.ufo.get_default_layer_mut()
                        {
                            if let Some(glyph) =
                                default_layer.get_glyph_mut(&glyph_name)
                            {
                                // Get or create the outline
                                let outline =
                                    glyph.outline.get_or_insert_with(|| {
                                        norad::glyph::Outline {
                                            contours: Vec::new(),
                                            components: Vec::new(),
                                        }
                                    });

                                match active_drawing.tool_type {
                                    PrimitiveType::Rectangle => {
                                        // Create a path for the rectangle
                                        let mut path = kurbo::BezPath::new();
                                        path.move_to(kurbo::Point::new(
                                            rect.min.x as f64,
                                            rect.min.y as f64,
                                        ));
                                        path.line_to(kurbo::Point::new(
                                            rect.max.x as f64,
                                            rect.min.y as f64,
                                        ));
                                        path.line_to(kurbo::Point::new(
                                            rect.max.x as f64,
                                            rect.max.y as f64,
                                        ));
                                        path.line_to(kurbo::Point::new(
                                            rect.min.x as f64,
                                            rect.max.y as f64,
                                        ));
                                        path.close_path();

                                        // Convert the path to a contour
                                        let contour_result: Result<
                                            norad::Contour,
                                            &'static str,
                                        > = {
                                            use kurbo::PathEl;

                                            let mut points = Vec::new();
                                            let mut current_point = None;

                                            for el in path.elements() {
                                                match el {
                                                    PathEl::MoveTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Move, false));
                                                    }
                                                    PathEl::LineTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Line, false));
                                                    }
                                                    PathEl::QuadTo(p1, p2) => {
                                                        // Convert quadratic bezier to cubic (not ideal but works for now)
                                                        if let Some(p0) =
                                                            current_point
                                                        {
                                                            let cp1 = kurbo::Point::new(
                                                                p0.x + 2.0/3.0 * (p1.x - p0.x),
                                                                p0.y + 2.0/3.0 * (p1.y - p0.y),
                                                            );
                                                            let cp2 = kurbo::Point::new(
                                                                p2.x + 2.0/3.0 * (p1.x - p2.x),
                                                                p2.y + 2.0/3.0 * (p1.y - p2.y),
                                                            );

                                                            points.push(create_point(cp1.x as f32, cp1.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(cp2.x as f32, cp2.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::Curve, true));

                                                            current_point =
                                                                Some(p2);
                                                        } else {
                                                            // If there's no current point, we can't create a quadratic curve
                                                            warn!("QuadTo without a current point");
                                                            continue;
                                                        }
                                                    }
                                                    PathEl::CurveTo(
                                                        p1,
                                                        p2,
                                                        p3,
                                                    ) => {
                                                        points.push(create_point(p1.x as f32, p1.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p3.x as f32, p3.y as f32, norad::PointType::Curve, true));

                                                        current_point =
                                                            Some(p3);
                                                    }
                                                    PathEl::ClosePath => {
                                                        // No need to add a point for close path
                                                    }
                                                }
                                            }

                                            // Create the contour with the points
                                            Ok(norad::Contour::new(
                                                points, None, None,
                                            ))
                                        };

                                        if let Ok(contour) = contour_result {
                                            outline.contours.push(contour);
                                            info!("Added rectangle contour to glyph {}", glyph_name);
                                        } else {
                                            warn!("Failed to convert rectangle to contour");
                                        }
                                    }
                                    PrimitiveType::Ellipse => {
                                        // Create a path for the ellipse
                                        let center_x =
                                            (rect.min.x + rect.max.x) / 2.0;
                                        let center_y =
                                            (rect.min.y + rect.max.y) / 2.0;
                                        let radius_x =
                                            (rect.max.x - rect.min.x) / 2.0;
                                        let radius_y =
                                            (rect.max.y - rect.min.y) / 2.0;

                                        let ellipse = kurbo::Ellipse::new(
                                            kurbo::Point::new(
                                                center_x as f64,
                                                center_y as f64,
                                            ),
                                            kurbo::Vec2::new(
                                                radius_x as f64,
                                                radius_y as f64,
                                            ),
                                            0.0,
                                        );

                                        let path = ellipse.to_path(0.1);

                                        // Convert the path to a contour
                                        let contour_result: Result<
                                            norad::Contour,
                                            &'static str,
                                        > = {
                                            use kurbo::PathEl;

                                            let mut points = Vec::new();
                                            let mut current_point = None;

                                            for el in path.elements() {
                                                match el {
                                                    PathEl::MoveTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Move, false));
                                                    }
                                                    PathEl::LineTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Line, false));
                                                    }
                                                    PathEl::QuadTo(p1, p2) => {
                                                        // Convert quadratic bezier to cubic (not ideal but works for now)
                                                        if let Some(p0) =
                                                            current_point
                                                        {
                                                            let cp1 = kurbo::Point::new(
                                                                p0.x + 2.0/3.0 * (p1.x - p0.x),
                                                                p0.y + 2.0/3.0 * (p1.y - p0.y),
                                                            );
                                                            let cp2 = kurbo::Point::new(
                                                                p2.x + 2.0/3.0 * (p1.x - p2.x),
                                                                p2.y + 2.0/3.0 * (p1.y - p2.y),
                                                            );

                                                            points.push(create_point(cp1.x as f32, cp1.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(cp2.x as f32, cp2.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::Curve, true));

                                                            current_point =
                                                                Some(p2);
                                                        } else {
                                                            // If there's no current point, we can't create a quadratic curve
                                                            warn!("QuadTo without a current point");
                                                            continue;
                                                        }
                                                    }
                                                    PathEl::CurveTo(
                                                        p1,
                                                        p2,
                                                        p3,
                                                    ) => {
                                                        points.push(create_point(p1.x as f32, p1.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p3.x as f32, p3.y as f32, norad::PointType::Curve, true));

                                                        current_point =
                                                            Some(p3);
                                                    }
                                                    PathEl::ClosePath => {
                                                        // No need to add a point for close path
                                                    }
                                                }
                                            }

                                            // Create the contour with the points
                                            Ok(norad::Contour::new(
                                                points, None, None,
                                            ))
                                        };

                                        if let Ok(contour) = contour_result {
                                            outline.contours.push(contour);
                                            info!("Added ellipse contour to glyph {}", glyph_name);
                                        } else {
                                            warn!("Failed to convert ellipse to contour");
                                        }
                                    }
                                    PrimitiveType::RoundedRectangle => {
                                        // Calculate corner radius - ensure it's appropriate for the rectangle size
                                        let width = rect.width();
                                        let height = rect.height();
                                        let min_dimension =
                                            width.min(height) / 2.0;
                                        let radius = (corner_radius.0 as f64)
                                            .min(min_dimension as f64);

                                        // Create a rounded rectangle with the specified corner radius
                                        let rounded_rect =
                                            kurbo::RoundedRect::new(
                                                rect.min.x as f64,
                                                rect.min.y as f64,
                                                rect.max.x as f64,
                                                rect.max.y as f64,
                                                radius,
                                            );

                                        // Convert to path
                                        let path = rounded_rect.to_path(0.1);

                                        // Convert the path to a contour
                                        let contour_result: Result<
                                            norad::Contour,
                                            &'static str,
                                        > = {
                                            use kurbo::PathEl;

                                            let mut points = Vec::new();
                                            let mut current_point = None;

                                            for el in path.elements() {
                                                match el {
                                                    PathEl::MoveTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Move, false));
                                                    }
                                                    PathEl::LineTo(p) => {
                                                        current_point = Some(p);
                                                        points.push(create_point(p.x as f32, p.y as f32, norad::PointType::Line, false));
                                                    }
                                                    PathEl::QuadTo(p1, p2) => {
                                                        // Convert quadratic bezier to cubic (not ideal but works for now)
                                                        if let Some(p0) =
                                                            current_point
                                                        {
                                                            let cp1 = kurbo::Point::new(
                                                                p0.x + 2.0/3.0 * (p1.x - p0.x),
                                                                p0.y + 2.0/3.0 * (p1.y - p0.y),
                                                            );
                                                            let cp2 = kurbo::Point::new(
                                                                p2.x + 2.0/3.0 * (p1.x - p2.x),
                                                                p2.y + 2.0/3.0 * (p1.y - p2.y),
                                                            );

                                                            points.push(create_point(cp1.x as f32, cp1.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(cp2.x as f32, cp2.y as f32, norad::PointType::OffCurve, false));
                                                            points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::Curve, true));

                                                            current_point =
                                                                Some(p2);
                                                        } else {
                                                            warn!("QuadTo without a current point");
                                                            continue;
                                                        }
                                                    }
                                                    PathEl::CurveTo(
                                                        p1,
                                                        p2,
                                                        p3,
                                                    ) => {
                                                        points.push(create_point(p1.x as f32, p1.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p2.x as f32, p2.y as f32, norad::PointType::OffCurve, false));
                                                        points.push(create_point(p3.x as f32, p3.y as f32, norad::PointType::Curve, true));

                                                        current_point =
                                                            Some(p3);
                                                    }
                                                    PathEl::ClosePath => {
                                                        // No need to add a point for close path
                                                    }
                                                }
                                            }

                                            // Create the contour with the points
                                            Ok(norad::Contour::new(
                                                points, None, None,
                                            ))
                                        };

                                        if let Ok(contour) = contour_result {
                                            outline.contours.push(contour);
                                            info!("Added rounded rectangle contour to glyph {}", glyph_name);
                                        } else {
                                            warn!("Failed to convert rounded rectangle to contour");
                                        }
                                    }
                                }

                                // Notify that the app state has changed
                                app_state_changed
                                    .send(crate::draw::AppStateChanged);
                            } else {
                                warn!(
                                    "Could not find glyph for contour creation"
                                );
                            }
                        } else {
                            warn!(
                                "No default layer found for contour creation"
                            );
                        }
                    } else {
                        warn!("No current glyph selected for contour creation");
                    }
                }
            } else {
                debug!("Mouse released but no current position available");
            }

            // Reset the drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
        }
    }

    // Handle keyboard input for canceling drawing
    if keyboard.just_pressed(KeyCode::Escape) {
        if active_drawing.is_drawing {
            // Cancel the drawing
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.cancel_draw();

            // Reset the drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
        }
    }

    // Handle shift key for constraining shapes
    if keyboard.just_pressed(KeyCode::ShiftLeft)
        || keyboard.just_pressed(KeyCode::ShiftRight)
    {
        if active_drawing.is_drawing {
            // Get the active primitive tool and toggle shift lock
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.set_shift_locked(true);
        }
    } else if keyboard.just_released(KeyCode::ShiftLeft)
        || keyboard.just_released(KeyCode::ShiftRight)
    {
        if active_drawing.is_drawing {
            // Get the active primitive tool and toggle shift lock off
            let mut tool = get_primitive_tool(active_drawing.tool_type);
            tool.set_shift_locked(false);
        }
    }
}

// System to render the active primitive drawing
pub fn render_active_primitive_drawing(
    mut gizmos: Gizmos,
    active_drawing: Res<ActivePrimitiveDrawing>,
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
) {
    // Only render when in primitives mode
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Primitives {
        return;
    }

    // Only render when actively drawing
    if !active_drawing.is_drawing {
        return;
    }

    // Get the rectangle from active_drawing
    if let Some(rect) = active_drawing.get_rect() {
        // Define the orange color to match selected buttons (similar to PRESSED_BUTTON in theme.rs)
        let orange_color = Color::srgb(1.0, 0.6, 0.1);
        
        // Draw based on the primitive type
        match active_drawing.tool_type {
            PrimitiveType::Rectangle => {
                draw_dashed_rectangle(&mut gizmos, rect, orange_color);
            },
            PrimitiveType::Ellipse => {
                draw_dashed_ellipse(&mut gizmos, rect, orange_color);
            },
            PrimitiveType::RoundedRectangle => {
                // Use a default corner radius of 10
                draw_dashed_rounded_rectangle(&mut gizmos, rect, 10.0, orange_color);
            },
        }
    }
}

// Helper function to draw a dashed rectangle
fn draw_dashed_rectangle(
    gizmos: &mut Gizmos,
    rect: Rect,
    color: Color,
) {
    let dash_length = 10.0;
    let gap_length = 5.0;
    
    // Get the corner points
    let min_x = rect.min.x;
    let min_y = rect.min.y;
    let max_x = rect.max.x;
    let max_y = rect.max.y;
    
    // Draw dashed lines for each side of the rectangle
    draw_dashed_line(gizmos, 
        Vec2::new(min_x, min_y), 
        Vec2::new(max_x, min_y), 
        dash_length, gap_length, color);
        
    draw_dashed_line(gizmos, 
        Vec2::new(max_x, min_y), 
        Vec2::new(max_x, max_y), 
        dash_length, gap_length, color);
        
    draw_dashed_line(gizmos, 
        Vec2::new(max_x, max_y), 
        Vec2::new(min_x, max_y), 
        dash_length, gap_length, color);
        
    draw_dashed_line(gizmos, 
        Vec2::new(min_x, max_y), 
        Vec2::new(min_x, min_y), 
        dash_length, gap_length, color);
}

// Helper function to draw a dashed ellipse
fn draw_dashed_ellipse(
    gizmos: &mut Gizmos,
    rect: Rect,
    color: Color,
) {
    // Calculate ellipse center and radii
    let center = Vec2::new(
        (rect.min.x + rect.max.x) / 2.0,
        (rect.min.y + rect.max.y) / 2.0,
    );
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;
    
    // Number of segments for the ellipse approximation
    let segments = 32;
    let segment_angle = std::f32::consts::PI * 2.0 / segments as f32;
    
    // Draw segments to approximate the ellipse
    for i in 0..segments {
        let start_angle = i as f32 * segment_angle;
        let end_angle = (i + 1) as f32 * segment_angle;
        
        let start_point = Vec2::new(
            center.x + radius_x * start_angle.cos(),
            center.y + radius_y * start_angle.sin(),
        );
        
        let end_point = Vec2::new(
            center.x + radius_x * end_angle.cos(),
            center.y + radius_y * end_angle.sin(),
        );
        
        // Skip every other segment to create a dashed effect
        if i % 2 == 0 {
            gizmos.line_2d(start_point, end_point, color);
        }
    }
}

// Helper function to draw a dashed rounded rectangle
fn draw_dashed_rounded_rectangle(
    gizmos: &mut Gizmos,
    rect: Rect,
    radius: f32,
    color: Color,
) {
    // Get the corner points
    let min_x = rect.min.x;
    let min_y = rect.min.y;
    let max_x = rect.max.x;
    let max_y = rect.max.y;
    
    // Limit the radius to half the smallest dimension
    let max_radius = ((max_x - min_x) / 2.0).min((max_y - min_y) / 2.0);
    let radius = radius.min(max_radius);
    
    if radius <= 0.1 {
        // If radius is very small, draw a regular rectangle
        draw_dashed_rectangle(gizmos, rect, color);
        return;
    }
    
    // Draw the straight segments with dashed lines
    // Top line
    draw_dashed_line(gizmos, 
        Vec2::new(min_x + radius, min_y), 
        Vec2::new(max_x - radius, min_y), 
        10.0, 5.0, color);
    
    // Right line
    draw_dashed_line(gizmos, 
        Vec2::new(max_x, min_y + radius), 
        Vec2::new(max_x, max_y - radius), 
        10.0, 5.0, color);
    
    // Bottom line
    draw_dashed_line(gizmos, 
        Vec2::new(max_x - radius, max_y), 
        Vec2::new(min_x + radius, max_y), 
        10.0, 5.0, color);
    
    // Left line
    draw_dashed_line(gizmos, 
        Vec2::new(min_x, max_y - radius), 
        Vec2::new(min_x, min_y + radius), 
        10.0, 5.0, color);
    
    // Draw the curved corners
    // Top-left corner
    draw_dashed_corner(gizmos, Vec2::new(min_x + radius, min_y + radius), radius, 180.0, 270.0, color);
    
    // Top-right corner
    draw_dashed_corner(gizmos, Vec2::new(max_x - radius, min_y + radius), radius, 270.0, 360.0, color);
    
    // Bottom-right corner
    draw_dashed_corner(gizmos, Vec2::new(max_x - radius, max_y - radius), radius, 0.0, 90.0, color);
    
    // Bottom-left corner
    draw_dashed_corner(gizmos, Vec2::new(min_x + radius, max_y - radius), radius, 90.0, 180.0, color);
}

// Helper function to draw a dashed corner arc
fn draw_dashed_corner(
    gizmos: &mut Gizmos,
    center: Vec2,
    radius: f32,
    start_angle_deg: f32,
    end_angle_deg: f32,
    color: Color,
) {
    let start_angle = start_angle_deg * std::f32::consts::PI / 180.0;
    let end_angle = end_angle_deg * std::f32::consts::PI / 180.0;
    
    // Number of segments for the corner approximation
    let segments = 8;
    let angle_per_segment = (end_angle - start_angle) / segments as f32;
    
    for i in 0..segments {
        if i % 2 == 0 {  // Skip every other segment for dashed effect
            let seg_start_angle = start_angle + i as f32 * angle_per_segment;
            let seg_end_angle = start_angle + (i + 1) as f32 * angle_per_segment;
            
            let start_point = Vec2::new(
                center.x + radius * seg_start_angle.cos(),
                center.y + radius * seg_start_angle.sin(),
            );
            
            let end_point = Vec2::new(
                center.x + radius * seg_end_angle.cos(),
                center.y + radius * seg_end_angle.sin(),
            );
            
            gizmos.line_2d(start_point, end_point, color);
        }
    }
}

// Helper function to draw a dashed line between two points
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    dash_length: f32,
    gap_length: f32,
    color: Color,
) {
    let direction = (end - start).normalize_or_zero();
    if direction == Vec2::ZERO {
        return; // Cannot draw a line with zero length
    }
    
    let total_length = start.distance(end);
    
    let segment_length = dash_length + gap_length;
    let num_segments = (total_length / segment_length).ceil() as usize;
    
    for i in 0..num_segments {
        let segment_start = start + direction * (i as f32 * segment_length);
        let raw_segment_end = segment_start + direction * dash_length;
        
        // Make sure we don't go past the end point
        let segment_end = if raw_segment_end.distance(start) > total_length {
            end
        } else {
            raw_segment_end
        };
        
        gizmos.line_2d(segment_start, segment_end, color);
    }
}

#[allow(dead_code)]
impl<'a> dyn PrimitiveShapeTool + 'a {
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        // This is unsafe because we're bypassing the compiler's type checking
        // It's safe only if we correctly check the type with TypeId
        unsafe {
            let type_id = std::any::TypeId::of::<T>();

            if self.type_id() == type_id {
                Some(&*(self as *const dyn PrimitiveShapeTool as *const T))
            } else {
                None
            }
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        // This is unsafe for the same reasons as above
        unsafe {
            let type_id = std::any::TypeId::of::<T>();

            if self.type_id() == type_id {
                Some(&mut *(self as *mut dyn PrimitiveShapeTool as *mut T))
            } else {
                None
            }
        }
    }

    fn type_id(&self) -> std::any::TypeId {
        // Use the name as a proxy for type_id (this is just a demonstration)
        // In a real implementation, you'd need proper type reflection
        match self.name() {
            "Rectangle" => std::any::TypeId::of::<RectanglePrimitive>(),
            "Ellipse" => std::any::TypeId::of::<EllipsePrimitive>(),
            "RoundedRectangle" => {
                std::any::TypeId::of::<RoundedRectanglePrimitive>()
            }
            _ => std::any::TypeId::of::<()>(), // Default for unknown types
        }
    }
}

// Util function to get current tool based on primitive type
pub fn get_primitive_tool(
    primitive_type: PrimitiveType,
) -> Box<dyn PrimitiveShapeTool> {
    match primitive_type {
        PrimitiveType::Rectangle => Box::new(RectanglePrimitive::default()),
        PrimitiveType::Ellipse => Box::new(EllipsePrimitive::default()),
        PrimitiveType::RoundedRectangle => {
            Box::new(RoundedRectanglePrimitive::default())
        }
    }
}

// Helper function to create a ContourPoint
fn create_point(
    x: f32,
    y: f32,
    typ: norad::PointType,
    smooth: bool,
) -> norad::ContourPoint {
    norad::ContourPoint::new(x, y, typ, smooth, None, None, None)
}
