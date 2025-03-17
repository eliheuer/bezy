use super::EditModeSystem;
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint, PointType};

/// Component to mark entities related to pen tool operations
#[derive(Component)]
pub struct PenToolComponent;

/// Resource to track the state of the pen tool
#[derive(Resource, Default)]
pub struct PenToolState {
    /// Whether the pen tool is active
    #[allow(dead_code)]
    pub active: bool,
    /// The current path being drawn
    pub current_path: Option<BezPath>,
    /// The point where the current drag started
    pub drag_start: Option<Vec2>,
    /// The point where the cursor currently is
    pub current_point: Option<Vec2>,
    /// Whether we're currently drawing a curve (drag) or a line (click)
    pub is_drawing_curve: bool,
    /// The entity ID of the current path being created
    pub current_path_entity: Option<Entity>,
    /// Points already placed in the current path
    pub points: Vec<Vec2>,
    /// Whether the last click closed the path
    pub path_closed: bool,
}

/// State of the pen tool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum PenState {
    /// Ready to place a point
    Ready,
    /// A point has been added
    PointAdded,
    /// Currently dragging to create a curve handle
    DraggingHandle,
}

/// Resource to track if pen mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct PenModeActive(pub bool);

/// Plugin to register pen mode systems
pub struct PenModePlugin;

impl Plugin for PenModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PenToolState>()
            .init_resource::<PenModeActive>()
            .add_systems(
                Update,
                (
                    handle_pen_mouse_events,
                    render_pen_preview,
                    handle_pen_keyboard_events,
                    reset_pen_mode_when_inactive,
                ),
            );
    }
}

/// Pen mode for drawing paths
pub struct PenMode;

impl EditModeSystem for PenMode {
    fn update(&self, commands: &mut Commands) {
        // Mark pen mode as active
        commands.insert_resource(PenModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entering Pen Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Pen Mode");
    }
}

/// System to handle deactivation of pen mode when another mode is selected
pub fn reset_pen_mode_when_inactive(
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
) {
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Pen {
        commands.insert_resource(PenModeActive(false));
    }
}

/// System to handle mouse events for the pen tool
pub fn handle_pen_mouse_events(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    app_state: Res<crate::data::AppState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
) {
    // Only handle events when in pen mode
    if let Some(pen_mode) = pen_mode {
        if !pen_mode.0 {
            return;
        }
    }

    // Early return if no window
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Find the primary camera
    let camera_entity = camera_q.iter().find(|(camera, _)| camera.is_active);

    // Early return if no camera
    let Some((camera, camera_transform)) = camera_entity else {
        warn!("No primary camera found for pen tool");
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
                Err(_) => continue,
            };

            // Update current cursor position
            pen_state.current_point = Some(world_position);
        }
    }

    // Handle mouse down
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = pen_state.current_point {
            // Get shift state for alignment
            let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight);

            // Get alt state for smooth/corner point (not used yet but kept for future)
            let _alt_pressed = keyboard.pressed(KeyCode::AltLeft)
                || keyboard.pressed(KeyCode::AltRight);

            // Adjust position based on shift (for axis alignment)
            let adjusted_pos = if shift_pressed && !pen_state.points.is_empty()
            {
                let last_point = pen_state.points.last().unwrap();
                axis_lock_position(world_pos, *last_point)
            } else {
                world_pos
            };

            // Initialize path if none exists
            if pen_state.current_path.is_none() {
                let mut path = BezPath::new();
                path.move_to((adjusted_pos.x as f64, adjusted_pos.y as f64));
                pen_state.current_path = Some(path);
                pen_state.points.push(adjusted_pos);

                // Create a new entity for this path
                let path_entity = commands
                    .spawn((
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        PenToolComponent,
                        Name::new("Pen Path"),
                    ))
                    .id();
                pen_state.current_path_entity = Some(path_entity);
            } else {
                // Check if clicking on start point to close the path
                if !pen_state.points.is_empty() && !pen_state.path_closed {
                    let start_point = pen_state.points[0];
                    let distance = start_point.distance(adjusted_pos);

                    if distance < 10.0 && pen_state.points.len() > 1 {
                        // Close the path
                        let mut path_clone = None;
                        if let Some(ref mut path) = pen_state.current_path {
                            path.close_path();
                            path_clone = Some(path.clone());
                        }

                        // Only continue if we have a path
                        if let Some(path) = path_clone {
                            pen_state.path_closed = true;

                            // Commit the path to the actual glyph
                            if let Some(active_glyph) =
                                app_state.workspace.selected.clone()
                            {
                                let contour = match bezpath_to_contour(&path) {
                                    Ok(contour) => contour,
                                    Err(e) => {
                                        error!("Failed to convert path to contour: {}", e);
                                        return;
                                    }
                                };

                                // Add path to glyph
                                info!("Adding path to glyph: {}", active_glyph);

                                // Since we can't directly modify the glyph through the Arc,
                                // we'll send an event with the contour data
                                app_state_changed
                                    .send(crate::draw::AppStateChanged);

                                // TODO: We need a proper event system for editing glyphs
                                // For now, just show info that path was created
                                info!(
                                    "Created contour with {} points",
                                    contour.points.len()
                                );
                            }

                            // Reset for next path
                            pen_state.current_path = None;
                            pen_state.points.clear();
                            pen_state.path_closed = false;
                            if let Some(entity) =
                                pen_state.current_path_entity.take()
                            {
                                commands.entity(entity).despawn_recursive();
                            }
                        }
                        return;
                    }
                }

                // Add a new point to the path
                {
                    // Check if we should create a line or a curve
                    let path_exists = pen_state.current_path.is_some();
                    pen_state.drag_start = Some(adjusted_pos);
                    pen_state.is_drawing_curve = false;

                    if path_exists {
                        if let Some(ref mut path) = pen_state.current_path {
                            // Add line segment by default
                            path.line_to((
                                adjusted_pos.x as f64,
                                adjusted_pos.y as f64,
                            ));
                            pen_state.points.push(adjusted_pos);
                        }
                    }
                }
            }
        }
    }

    // Handle mouse drag (for curve handles)
    if mouse_button_input.pressed(MouseButton::Left)
        && pen_state.drag_start.is_some()
    {
        if let (Some(start_pos), Some(current_pos)) =
            (pen_state.drag_start, pen_state.current_point)
        {
            let dist = start_pos.distance(current_pos);

            // Only start curve mode if we've dragged a sufficient distance
            if dist > 5.0
                && !pen_state.is_drawing_curve
                && pen_state.current_path.is_some()
            {
                pen_state.is_drawing_curve = true;

                // Remove the last line segment and replace with a curve
                let mut path_clone = None;
                let _points_clone = pen_state.points.clone(); // Only used if needed later

                if let Some(ref mut path) = pen_state.current_path {
                    // We need to pop the last line_to and replace with curve_to
                    // This is a bit hacky - we create a new path and copy all elements except the last
                    let mut new_path = BezPath::new();
                    let elements = path.elements();

                    // Only proceed if we have at least a move_to and a line_to
                    if elements.len() >= 2 {
                        // Copy all elements except the last line_to
                        for (i, el) in elements.iter().enumerate() {
                            if i < elements.len() - 1 {
                                match el {
                                    kurbo::PathEl::MoveTo(p) => {
                                        new_path.move_to((p.x, p.y))
                                    }
                                    kurbo::PathEl::LineTo(p) => {
                                        new_path.line_to((p.x, p.y))
                                    }
                                    kurbo::PathEl::QuadTo(p1, p2) => new_path
                                        .quad_to((p1.x, p1.y), (p2.x, p2.y)),
                                    kurbo::PathEl::CurveTo(p1, p2, p3) => {
                                        new_path.curve_to(
                                            (p1.x, p1.y),
                                            (p2.x, p2.y),
                                            (p3.x, p3.y),
                                        )
                                    }
                                    kurbo::PathEl::ClosePath => {
                                        new_path.close_path()
                                    }
                                }
                            }
                        }

                        path_clone = Some(new_path);
                    }
                }

                if let Some(new_path) = path_clone {
                    // Replace path
                    pen_state.current_path = Some(new_path);

                    // Remove the last point we added
                    if !pen_state.points.is_empty() {
                        pen_state.points.pop();
                    }
                }
            }

            // Update curve in progress
            if pen_state.is_drawing_curve && pen_state.current_path.is_some() {
                let points_clone = pen_state.points.clone();
                if let Some(ref mut path) = pen_state.current_path {
                    // Only proceed if we have points
                    if !points_clone.is_empty() {
                        // Get the last point
                        let last_idx = points_clone.len() - 1;
                        let last_point = points_clone[last_idx];

                        // Calculate control points for the curve
                        let shift_pressed = keyboard
                            .pressed(KeyCode::ShiftLeft)
                            || keyboard.pressed(KeyCode::ShiftRight);
                        let current = if shift_pressed {
                            axis_lock_position(current_pos, last_point)
                        } else {
                            current_pos
                        };

                        // Reflect control point for smooth curve
                        let control1 = last_point;
                        let control2 = current;

                        // Update the path with a cubic bezier curve
                        path.curve_to(
                            (control1.x as f64, control1.y as f64),
                            (control2.x as f64, control2.y as f64),
                            (start_pos.x as f64, start_pos.y as f64),
                        );

                        // Update points list
                        pen_state.points.push(start_pos);
                    }
                }
            }
        }
    }

    // Handle mouse up
    if mouse_button_input.just_released(MouseButton::Left)
        && pen_state.drag_start.is_some()
    {
        pen_state.drag_start = None;
        pen_state.is_drawing_curve = false;
    }
}

/// System to render a preview of the pen tool's current path
pub fn render_pen_preview(
    mut gizmos: Gizmos,
    pen_state: Res<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
) {
    // Only render when in pen mode
    if let Some(pen_mode) = pen_mode {
        if !pen_mode.0 {
            return;
        }
    }

    // Define colors
    let point_color = Color::srgb(1.0, 1.0, 0.0); // Yellow
    let line_color = Color::srgb(1.0, 1.0, 1.0); // White
    let handle_line_color = Color::srgb(0.7, 0.7, 0.7); // Gray
    let handle_point_color = Color::srgb(0.0, 0.5, 1.0); // Blue
    let preview_color = Color::srgba(1.0, 1.0, 1.0, 0.5); // Translucent white

    // Draw the current path
    if let Some(ref path) = pen_state.current_path {
        // Draw each segment
        let mut last_point = None;

        for el in path.elements() {
            match el {
                kurbo::PathEl::MoveTo(p) => {
                    let point = Vec2::new(p.x as f32, p.y as f32);
                    gizmos.circle_2d(point, 5.0, point_color);
                    last_point = Some(point);
                }
                kurbo::PathEl::LineTo(p) => {
                    let point = Vec2::new(p.x as f32, p.y as f32);
                    if let Some(last) = last_point {
                        gizmos.line_2d(last, point, line_color);
                    }
                    gizmos.circle_2d(point, 5.0, point_color);
                    last_point = Some(point);
                }
                kurbo::PathEl::CurveTo(p1, p2, p3) => {
                    let control1 = Vec2::new(p1.x as f32, p1.y as f32);
                    let control2 = Vec2::new(p2.x as f32, p2.y as f32);
                    let point = Vec2::new(p3.x as f32, p3.y as f32);

                    if let Some(last) = last_point {
                        // Draw curve
                        draw_bezier_curve(
                            &mut gizmos,
                            last,
                            control1,
                            control2,
                            point,
                            line_color,
                        );

                        // Draw control points and handles
                        gizmos.line_2d(last, control1, handle_line_color);
                        gizmos.line_2d(point, control2, handle_line_color);
                        gizmos.circle_2d(control1, 3.0, handle_point_color);
                        gizmos.circle_2d(control2, 3.0, handle_point_color);
                    }

                    gizmos.circle_2d(point, 5.0, point_color);
                    last_point = Some(point);
                }
                _ => {}
            }
        }

        // Draw preview line or curve from last point to current cursor
        if let (Some(last), Some(current)) =
            (last_point, pen_state.current_point)
        {
            if pen_state.is_drawing_curve {
                // Preview curve
                let control1 = last;
                let control2 = current;
                draw_bezier_curve(
                    &mut gizmos,
                    last,
                    control1,
                    control2,
                    current,
                    preview_color,
                );
            } else {
                // Preview line
                gizmos.line_2d(last, current, preview_color);
            }
        }
    }

    // Draw point at cursor position when no path is active
    if pen_state.current_path.is_none() && pen_state.current_point.is_some() {
        gizmos.circle_2d(pen_state.current_point.unwrap(), 5.0, point_color);
    }
}

/// System to handle keyboard events for the pen tool
pub fn handle_pen_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    mut commands: Commands,
    pen_mode: Option<Res<PenModeActive>>,
) {
    // Only handle events when in pen mode
    if let Some(pen_mode) = pen_mode {
        if !pen_mode.0 {
            return;
        }
    }

    // Handle Escape key to cancel current path
    if keyboard.just_pressed(KeyCode::Escape) {
        pen_state.current_path = None;
        pen_state.points.clear();
        pen_state.path_closed = false;
        if let Some(entity) = pen_state.current_path_entity.take() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Helper function to lock a position to the horizontal or vertical axis relative to another point
fn axis_lock_position(pos: Vec2, relative_to: Vec2) -> Vec2 {
    let dx = (pos.x - relative_to.x).abs();
    let dy = (pos.y - relative_to.y).abs();

    if dx >= dy {
        // Lock to horizontal axis
        Vec2::new(pos.x, relative_to.y)
    } else {
        // Lock to vertical axis
        Vec2::new(relative_to.x, pos.y)
    }
}

/// Helper function to draw a cubic bezier curve using line segments
fn draw_bezier_curve(
    gizmos: &mut Gizmos,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    color: Color,
) {
    // Number of segments to use when approximating the curve
    const SEGMENTS: usize = 20;

    let mut prev = p0;

    for i in 1..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        // Cubic Bezier formula
        let point =
            mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3;

        gizmos.line_2d(prev, point, color);
        prev = point;
    }
}

/// Convert a kurbo::BezPath to a norad::Contour
/// This is a local implementation used by the pen tool
fn bezpath_to_contour(
    path: &kurbo::BezPath,
) -> Result<norad::Contour, &'static str> {
    use kurbo::PathEl;

    let mut points = Vec::new();
    let mut current_point = None;

    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => {
                current_point = Some(p);
                points.push(create_point(
                    p.x as f32,
                    p.y as f32,
                    PointType::Move,
                    false,
                ));
            }
            PathEl::LineTo(p) => {
                current_point = Some(p);
                points.push(create_point(
                    p.x as f32,
                    p.y as f32,
                    PointType::Line,
                    false,
                ));
            }
            PathEl::QuadTo(p1, p2) => {
                // Convert quadratic bezier to cubic (not ideal but works for now)
                if let Some(p0) = current_point {
                    let cp1 = kurbo::Point::new(
                        p0.x + 2.0 / 3.0 * (p1.x - p0.x),
                        p0.y + 2.0 / 3.0 * (p1.y - p0.y),
                    );
                    let cp2 = kurbo::Point::new(
                        p2.x + 2.0 / 3.0 * (p1.x - p2.x),
                        p2.y + 2.0 / 3.0 * (p1.y - p2.y),
                    );

                    points.push(create_point(
                        cp1.x as f32,
                        cp1.y as f32,
                        PointType::OffCurve,
                        false,
                    ));
                    points.push(create_point(
                        cp2.x as f32,
                        cp2.y as f32,
                        PointType::OffCurve,
                        false,
                    ));
                    points.push(create_point(
                        p2.x as f32,
                        p2.y as f32,
                        PointType::Curve,
                        true,
                    ));

                    current_point = Some(p2);
                } else {
                    return Err("QuadTo without a current point");
                }
            }
            PathEl::CurveTo(p1, p2, p3) => {
                points.push(create_point(
                    p1.x as f32,
                    p1.y as f32,
                    PointType::OffCurve,
                    false,
                ));
                points.push(create_point(
                    p2.x as f32,
                    p2.y as f32,
                    PointType::OffCurve,
                    false,
                ));
                points.push(create_point(
                    p3.x as f32,
                    p3.y as f32,
                    PointType::Curve,
                    true,
                ));

                current_point = Some(p3);
            }
            PathEl::ClosePath => {
                // No need to add a point for close path
            }
        }
    }

    // Create the contour with the points
    let contour = Contour::new(points, None, None);

    Ok(contour)
}

/// Helper function to create a ContourPoint
fn create_point(x: f32, y: f32, typ: PointType, smooth: bool) -> ContourPoint {
    ContourPoint::new(x, y, typ, smooth, None, None, None)
}
