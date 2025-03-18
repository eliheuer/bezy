use super::EditModeSystem;
use bevy::prelude::*;
use kurbo::{BezPath, Line, ParamCurve, ParamCurveNearest};

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

/// Plugin to register knife mode systems
pub struct KnifeModePlugin;

impl Plugin for KnifeModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeToolState>()
            .init_resource::<KnifeModeActive>()
            .add_systems(
                Update,
                (
                    handle_knife_mouse_events,
                    render_knife_preview,
                    handle_knife_keyboard_events,
                    reset_knife_mode_when_inactive,
                ),
            );
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
#[derive(Resource)]
pub struct KnifeToolState {
    /// Whether the knife tool is active
    pub active: bool,
    /// The current gesture state
    pub gesture: KnifeGestureState,
    /// Whether shift key is pressed (for axis-aligned cuts)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
}

impl Default for KnifeToolState {
    fn default() -> Self {
        Self {
            active: true,
            gesture: KnifeGestureState::default(),
            shift_locked: false,
            intersections: Vec::new(),
        }
    }
}

/// Knife mode for cutting paths
pub struct KnifeMode;

impl EditModeSystem for KnifeMode {
    fn update(&self, commands: &mut Commands) {
        // Mark knife mode as active
        commands.insert_resource(KnifeModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entering Knife Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Knife Mode");
    }
}

/// System to handle deactivation of knife mode when another mode is selected
pub fn reset_knife_mode_when_inactive(
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
    mut knife_state: ResMut<KnifeToolState>,
) {
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Knife {
        // Clear state and mark inactive
        *knife_state = KnifeToolState::default();
        knife_state.active = false;
        commands.insert_resource(KnifeModeActive(false));
    }
}

/// System to handle mouse events for the knife tool
#[allow(clippy::too_many_arguments)]
pub fn handle_knife_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    camera_q: Query<
        (&Camera, &GlobalTransform),
        With<crate::cameras::DesignCamera>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
    cli_args: Res<crate::cli::CliArgs>,
    mut app_state: ResMut<crate::data::AppState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
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
        warn!("No active camera found for knife tool");
        return;
    };

    // Check for shift key (for axis-constrained cuts)
    knife_state.shift_locked = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            if let Ok(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_pos)
            {
                // Update the knife line if currently cutting
                if let KnifeGestureState::Cutting {
                    start: _,
                    ref mut current,
                } = knife_state.gesture
                {
                    *current = world_position;
                    update_intersections(&mut knife_state, &app_state);
                }
            }
        }
    }

    // Handle mouse down - start knife cut
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_pos)
            {
                // Start a new cut
                knife_state.gesture = KnifeGestureState::Cutting {
                    start: world_position,
                    current: world_position,
                };
                knife_state.intersections.clear();
                info!("Started knife cut at: {:?}", world_position);
            }
        }
    }

    // Handle mouse up - complete the cut
    if mouse_button_input.just_released(MouseButton::Left) {
        if let KnifeGestureState::Cutting { start, current } =
            knife_state.gesture
        {
            info!("Completing knife cut from {:?} to {:?}", start, current);

            // Don't process extremely short cuts (likely accidental clicks)
            if start.distance(current) > 5.0 {
                // Perform the actual cutting operation here
                perform_cut(
                    &start,
                    &current,
                    knife_state.shift_locked,
                    &mut app_state,
                    &cli_args,
                );

                // Notify that we've made a change to the glyph
                app_state_changed.send(crate::draw::AppStateChanged);
            }

            // Reset to ready state
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.intersections.clear();
        }
    }
}

/// System to render a preview of the knife tool's current cut line
pub fn render_knife_preview(
    mut gizmos: Gizmos,
    knife_state: Res<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
) {
    // Only render when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    }

    // Define colors
    let line_color = Color::srgba(1.0, 0.3, 0.3, 0.9); // Reddish for cut line
    let intersection_color = Color::srgba(1.0, 1.0, 0.0, 1.0); // Yellow for intersections

    // Draw the current knife line
    if let KnifeGestureState::Cutting { start, current } = knife_state.gesture {
        let actual_end = if knife_state.shift_locked {
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

        // Draw the knife line with dashed style
        draw_dashed_line(&mut gizmos, start, actual_end, 8.0, 4.0, line_color);

        // Mark start point
        gizmos.circle_2d(start, 4.0, line_color);

        // Draw intersection points
        for point in &knife_state.intersections {
            gizmos.circle_2d(*point, 6.0, intersection_color);

            // Draw a small cross at each intersection
            let cross_size = 4.0;
            gizmos.line_2d(
                Vec2::new(point.x - cross_size, point.y),
                Vec2::new(point.x + cross_size, point.y),
                intersection_color,
            );
            gizmos.line_2d(
                Vec2::new(point.x, point.y - cross_size),
                Vec2::new(point.x, point.y + cross_size),
                intersection_color,
            );
        }
    }
}

/// Draw a dashed line between two points (helper for visualization)
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    dash_length: f32,
    gap_length: f32,
    color: Color,
) {
    let direction = (end - start).normalize();
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

/// System to handle keyboard events for the knife tool
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
    }

    // Handle Escape key to cancel current cut
    if keyboard.just_pressed(KeyCode::Escape) {
        knife_state.gesture = KnifeGestureState::Ready;
        knife_state.intersections.clear();
        info!("Cancelled current knife cut with Escape key");
    }

    // Update shift key state
    let new_shift_state = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if knife_state.shift_locked != new_shift_state {
        knife_state.shift_locked = new_shift_state;
    }
}

/// Update the list of intersections with the current glyphs
fn update_intersections(
    knife_state: &mut KnifeToolState,
    app_state: &crate::data::AppState,
) {
    // Clear previous intersections
    knife_state.intersections.clear();

    if let KnifeGestureState::Cutting { start, current } = knife_state.gesture {
        // Apply axis constraint if shift is pressed
        let actual_end = if knife_state.shift_locked {
            let delta = current - start;
            if delta.x.abs() > delta.y.abs() {
                // Horizontal constraint
                Vec2::new(current.x, start.y)
            } else {
                // Vertical constraint
                Vec2::new(start.x, current.y)
            }
        } else {
            current
        };

        // Create a Line from the knife cut
        let line = Line::new(
            (start.x as f64, start.y as f64),
            (actual_end.x as f64, actual_end.y as f64),
        );

        // Get the font from the workspace
        let font = &app_state.workspace.font;

        // Get the selected glyph (if any)
        if let Some(selected) = &app_state.workspace.selected {
            // Try to get the default layer
            if let Some(layer) = font.ufo.get_default_layer() {
                // Try to get the glyph
                if let Some(glyph) = layer.get_glyph(selected) {
                    // Process the outline if it exists
                    if let Some(outline) = &glyph.outline {
                        for contour in &outline.contours {
                            // Convert contour to BezPath and find intersections
                            if let Some(bez_path) =
                                convert_contour_to_bezpath(contour)
                            {
                                find_intersections(
                                    &bez_path,
                                    line,
                                    &mut knife_state.intersections,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper function to convert a contour to BezPath for intersection checking
fn convert_contour_to_bezpath(contour: &norad::Contour) -> Option<BezPath> {
    let mut path = BezPath::new();

    if contour.points.is_empty() {
        return None;
    }

    // Start with the first point
    let first_point = &contour.points[0];
    path.move_to((first_point.x as f64, first_point.y as f64));

    // Add the remaining points
    for i in 1..contour.points.len() {
        let point = &contour.points[i];

        match point.typ {
            norad::PointType::Move => {
                path.move_to((point.x as f64, point.y as f64));
            }
            norad::PointType::Line => {
                path.line_to((point.x as f64, point.y as f64));
            }
            norad::PointType::Curve => {
                // Cubic Bezier curves need control points
                // This is a simplification; real code would need to handle this properly
                // by looking at previous points for control points
                if i >= 3 {
                    let cp1 = &contour.points[i - 2];
                    let cp2 = &contour.points[i - 1];
                    path.curve_to(
                        (cp1.x as f64, cp1.y as f64),
                        (cp2.x as f64, cp2.y as f64),
                        (point.x as f64, point.y as f64),
                    );
                } else {
                    // Fallback if we don't have enough points
                    path.line_to((point.x as f64, point.y as f64));
                }
            }
            norad::PointType::QCurve => {
                // Quadratic Bezier curve
                if i >= 2 {
                    let cp = &contour.points[i - 1];
                    path.quad_to(
                        (cp.x as f64, cp.y as f64),
                        (point.x as f64, point.y as f64),
                    );
                } else {
                    // Fallback
                    path.line_to((point.x as f64, point.y as f64));
                }
            }
            _ => {
                // Unknown or unsupported point type
                path.line_to((point.x as f64, point.y as f64));
            }
        }
    }

    // Close the path if the contour is closed
    if !contour.points.is_empty() {
        path.close_path();
    }

    Some(path)
}

/// Find intersections between a BezPath and a Line
fn find_intersections(
    path: &BezPath,
    line: Line,
    intersections: &mut Vec<Vec2>,
) {
    // Iterate through each segment in the path
    for seg in path.segments() {
        // Check for intersections with this segment
        for hit in seg.intersect_line(line) {
            // Convert the intersection point to Vec2
            let point = line.eval(hit.line_t);
            intersections.push(Vec2::new(point.x as f32, point.y as f32));
        }
    }
}

/// Perform the actual cut operation
fn perform_cut(
    start: &Vec2,
    end: &Vec2,
    shift_locked: bool,
    app_state: &mut crate::data::AppState,
    cli_args: &crate::cli::CliArgs,
) {
    // Apply axis constraint if shift is pressed
    let actual_end = if shift_locked {
        let delta = *end - *start;
        if delta.x.abs() > delta.y.abs() {
            // Horizontal constraint
            Vec2::new(end.x, start.y)
        } else {
            // Vertical constraint
            Vec2::new(start.x, end.y)
        }
    } else {
        *end
    };

    // Create a Line from the knife cut
    let line = Line::new(
        (start.x as f64, start.y as f64),
        (actual_end.x as f64, actual_end.y as f64),
    );

    // Get the current glyph that needs to be cut
    if let Some(glyph_name_str) =
        cli_args.find_glyph(&app_state.workspace.font.ufo)
    {
        let glyph_name = glyph_name_str.clone();

        // Get mutable access to the font
        let font_obj = app_state.workspace.font_mut();

        // Get the current glyph
        if let Some(default_layer) = font_obj.ufo.get_default_layer_mut() {
            if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
                // Get the outline
                if let Some(outline) = glyph.outline.as_mut() {
                    // Process each contour for cutting
                    let original_contours =
                        std::mem::take(&mut outline.contours);

                    // Count how many contours we modified
                    let mut contours_cut = 0;

                    // For each contour, try to cut it
                    for contour in original_contours {
                        // Convert contour to BezPath for intersection checking
                        if let Some(bez_path) =
                            convert_contour_to_bezpath(&contour)
                        {
                            // Find intersections with the knife line
                            let mut intersections = Vec::new();

                            // Collect all intersection points
                            for seg in bez_path.segments() {
                                for hit in seg.intersect_line(line) {
                                    let point = line.eval(hit.line_t);
                                    let vec2_point = Vec2::new(
                                        point.x as f32,
                                        point.y as f32,
                                    );

                                    // Add this intersection if it's not a duplicate
                                    if !intersections.iter().any(|p: &Vec2| {
                                        is_point_near(*p, vec2_point, 0.01)
                                    }) {
                                        intersections.push(vec2_point);
                                    }
                                }
                            }

                            // Perform the cut if we have enough intersections
                            if intersections.len() >= 2 {
                                let cut_contours =
                                    cut_contour(&contour, &intersections, line);
                                let num_parts = cut_contours.len();
                                if num_parts > 1 {
                                    // If we generated new contours, add them
                                    outline.contours.extend(cut_contours);
                                    contours_cut += 1;
                                    info!(
                                        "Cut contour into {} parts",
                                        num_parts
                                    );
                                } else {
                                    // If cutting failed, keep the original
                                    outline.contours.push(contour);
                                    info!("Cutting failed, keeping original contour");
                                }
                            } else {
                                // Not enough intersections, keep original contour
                                outline.contours.push(contour);
                                info!("Not enough intersections ({}) to cut contour", intersections.len());
                            }
                        } else {
                            // Could not convert to BezPath, keep original
                            outline.contours.push(contour);
                            info!("Failed to convert contour to BezPath for cutting");
                        }
                    }

                    info!("Knife operation completed: cut {} contours in glyph {}", contours_cut, glyph_name);
                }
            }
        }
    }
}

/// Cut a contour at the intersection points with a line
fn cut_contour(
    contour: &norad::Contour,
    intersections: &[Vec2],
    _line: Line,
) -> Vec<norad::Contour> {
    // Need at least 2 intersections to make a cut
    if intersections.len() < 2 {
        return vec![contour.clone()];
    }

    info!(
        "Cutting contour with {} intersection points",
        intersections.len()
    );

    // 1. Convert intersections to point-parameter pairs on the contour
    let mut contour_parameters: Vec<(usize, f64, Vec2)> = Vec::new();

    // Create a BezPath for parameter calculation
    let bez_path = match convert_contour_to_bezpath(contour) {
        Some(path) => path,
        None => return vec![contour.clone()],
    };

    // For each segment in the path, find where it intersects
    let path_segments: Vec<_> = bez_path.segments().collect();

    // For each intersection point, find which segment it belongs to
    // and its parameter value along that segment
    for &intersection_point in intersections {
        let point = kurbo::Point::new(
            intersection_point.x as f64,
            intersection_point.y as f64,
        );

        // Find the segment and parameter value for this intersection
        for (seg_idx, segment) in path_segments.iter().enumerate() {
            // Find closest point on this segment
            let closest = segment.nearest(point, 0.01);

            // If this is a very close match (likely an intersection)
            if closest.distance_sq < 1.0 {
                // Get the point at parameter t
                let point_at_t = segment.eval(closest.t);
                // Check distance to the intersection point
                if (point_at_t - point).length() < 1.0 {
                    // Store (segment index, parameter, point)
                    contour_parameters.push((
                        seg_idx,
                        closest.t,
                        intersection_point,
                    ));
                }
            }
        }
    }

    // Sort parameters by segment index and parameter value
    contour_parameters.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    // We need at least 2 parameters to perform a cut
    if contour_parameters.len() < 2 {
        return vec![contour.clone()];
    }

    // Create the first contour (from start to first intersection)
    let mut result_contours = Vec::new();

    // 2. Create two new contours, one for each side of the cut
    let mut contour_a_points = Vec::new();
    let mut contour_b_points = Vec::new();

    // Get the intersection points in the order they are encountered along the contour
    let intersection_points: Vec<Vec2> =
        contour_parameters.iter().map(|(_, _, pt)| *pt).collect();

    // Create the two new contours - we'll cut the contour between pairs of intersections
    // First let's handle the simple case with exactly 2 intersections
    if intersection_points.len() == 2 {
        // Add the first intersection point to both contours
        let first_intersection = intersection_points[0];
        let second_intersection = intersection_points[1];

        // Start both contours with the Move points at the intersections
        contour_a_points.push(create_point_at_position(
            first_intersection.x as f64,
            first_intersection.y as f64,
            norad::PointType::Move,
        ));

        contour_b_points.push(create_point_at_position(
            second_intersection.x as f64,
            second_intersection.y as f64,
            norad::PointType::Move,
        ));

        // Find the closest original points to our intersections for proper replacement
        let mut closest_to_first: Option<(usize, f32)> = None;
        let mut closest_to_second: Option<(usize, f32)> = None;

        // Find the closest original points to the intersections
        for (idx, point) in contour.points.iter().enumerate() {
            let point_vec = Vec2::new(point.x as f32, point.y as f32);

            let dist_to_first = point_vec.distance(first_intersection);
            if closest_to_first.map_or(true, |(_, dist)| dist_to_first < dist) {
                closest_to_first = Some((idx, dist_to_first));
            }

            let dist_to_second = point_vec.distance(second_intersection);
            if closest_to_second.map_or(true, |(_, dist)| dist_to_second < dist)
            {
                closest_to_second = Some((idx, dist_to_second));
            }
        }

        // Process all contour points, splitting at the intersections
        if let (Some((first_idx, _)), Some((second_idx, _))) =
            (closest_to_first, closest_to_second)
        {
            // Determine which index comes first in the contour
            let (earlier_idx, earlier_point, later_idx, later_point) =
                if first_idx <= second_idx {
                    (
                        first_idx,
                        first_intersection,
                        second_idx,
                        second_intersection,
                    )
                } else {
                    (
                        second_idx,
                        second_intersection,
                        first_idx,
                        first_intersection,
                    )
                };

            // Add points from original contour to the first section
            for i in 0..earlier_idx {
                contour_a_points.push(contour.points[i].clone());
            }

            // Add first intersection point (if not already at an existing point)
            if closest_to_first.unwrap().1 > 0.1 {
                contour_a_points.push(create_point_at_position(
                    earlier_point.x as f64,
                    earlier_point.y as f64,
                    norad::PointType::Line,
                ));
            }

            // Add the cut line to the first contour
            contour_a_points.push(create_point_at_position(
                later_point.x as f64,
                later_point.y as f64,
                norad::PointType::Line,
            ));

            // Add second intersection point to second contour
            contour_b_points.push(create_point_at_position(
                later_point.x as f64,
                later_point.y as f64,
                norad::PointType::Line,
            ));

            // Add remaining points from original contour
            for i in later_idx..contour.points.len() {
                contour_b_points.push(contour.points[i].clone());
            }

            // Add the remaining points from the beginning of the contour
            for i in 0..=earlier_idx {
                if i < contour.points.len() {
                    contour_b_points.push(contour.points[i].clone());
                }
            }

            // Close the contours with the cut line
            contour_b_points.push(create_point_at_position(
                earlier_point.x as f64,
                earlier_point.y as f64,
                norad::PointType::Line,
            ));
        }

        // Create the two final contours
        if contour_a_points.len() >= 3 {
            result_contours.push(norad::Contour::new(
                contour_a_points,
                None,
                None,
            ));
        }

        if contour_b_points.len() >= 3 {
            result_contours.push(norad::Contour::new(
                contour_b_points,
                None,
                None,
            ));
        }
    } else {
        // For more complex cases with more than 2 intersections
        // Return the original contour for now
        // In a real implementation, this would handle multiple intersections
        result_contours.push(contour.clone());
    }

    // If something went wrong and we have no results, return the original
    if result_contours.is_empty() {
        vec![contour.clone()]
    } else {
        result_contours
    }
}

/// Helper function to create a ContourPoint at the given position
fn create_point_at_position(
    x: f64,
    y: f64,
    point_type: norad::PointType,
) -> norad::ContourPoint {
    norad::ContourPoint::new(
        x as f32, y as f32, point_type, false, // not smooth
        None,  // no name
        None,  // no identifier
        None,  // no comments
    )
}

/// Helper function to check if two points are close to each other
fn is_point_near(p1: Vec2, p2: Vec2, threshold: f32) -> bool {
    p1.distance(p2) < threshold
}
