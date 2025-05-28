use super::EditModeSystem;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use kurbo::{BezPath, ParamCurve};
use log::info;

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

impl KnifeToolState {
    /// Create a line tuple for intersection testing
    fn create_line(&self) -> Option<((f64, f64), (f64, f64))> {
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

                Some((
                    (start.x as f64, start.y as f64),
                    (actual_end.x as f64, actual_end.y as f64),
                ))
            }
            KnifeGestureState::Ready => None, // Not in cutting state, no line to check
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
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
    mut knife_state: ResMut<KnifeToolState>,
) {
    if current_mode.0 != crate::ui::toolbars::edit_mode_toolbar::EditMode::Knife {
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
    camera_q: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
    mut app_state: ResMut<crate::core::data::AppState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    }

    // Don't process knife interactions when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
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
                );

                // Notify that we've made a change to the glyph
                app_state_changed.send(crate::rendering::draw::AppStateChanged);
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
        draw_dashed_line(
            &mut gizmos,
            start,
            actual_end,
            crate::ui::theme::KNIFE_DASH_LENGTH,
            crate::ui::theme::KNIFE_GAP_LENGTH,
            crate::ui::theme::KNIFE_LINE_COLOR,
        );

        // Mark start point with a larger circle
        gizmos.circle_2d(start, 6.0, crate::ui::theme::KNIFE_START_POINT_COLOR);

        // Draw intersection points with a more visible indicator
        for point in &knife_state.intersections {
            // Draw filled circle at intersection
            gizmos.circle_2d(
                *point,
                6.0,
                crate::ui::theme::KNIFE_INTERSECTION_COLOR,
            );

            // Draw a small cross at each intersection for extra visibility
            let cross_size = crate::ui::theme::KNIFE_CROSS_SIZE;
            gizmos.line_2d(
                Vec2::new(point.x - cross_size, point.y),
                Vec2::new(point.x + cross_size, point.y),
                crate::ui::theme::KNIFE_INTERSECTION_COLOR,
            );
            gizmos.line_2d(
                Vec2::new(point.x, point.y - cross_size),
                Vec2::new(point.x, point.y + cross_size),
                crate::ui::theme::KNIFE_INTERSECTION_COLOR,
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
    app_state: Res<crate::core::data::AppState>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    }

    // Track shift key state for axis-aligned cuts
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    // Only update if the state has changed
    if knife_state.shift_locked != shift_pressed {
        knife_state.shift_locked = shift_pressed;

        // If we're in the middle of drawing a cut line, update intersections
        if let KnifeGestureState::Cutting { .. } = knife_state.gesture {
            // Update intersections based on the new constraint
            update_intersections(&mut *knife_state, &app_state);
        }
    }

    // Handle Escape key to cancel current knife operation
    if keyboard.just_pressed(KeyCode::Escape) {
        if let KnifeGestureState::Cutting { .. } = knife_state.gesture {
            // Reset to ready state
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.intersections.clear();
            info!("Knife operation cancelled with Escape key");
        }
    }
}

/// Update the intersection points for the knife tool
fn update_intersections(
    knife_state: &mut KnifeToolState,
    app_state: &crate::core::data::AppState,
) {
    info!("Updating intersections for knife tool");

    knife_state.intersections.clear();

    if let Some(line) = knife_state.create_line() {
        // Attempt to get a glyph
        let glyph_name = if let Some(selected) = &app_state.workspace.selected {
            info!("Using glyph from workspace selection: {}", selected);
            selected.clone()
        } else {
            // Try to get some common glyphs that are likely to exist
            let common_glyphs = ["a", "A", "space", "period"];

            let mut found_glyph = None;
            for &glyph_name in common_glyphs.iter() {
                let glyph_name = norad::GlyphName::from(glyph_name);
                if let Some(default_layer) =
                    app_state.workspace.font.ufo.get_default_layer()
                {
                    if default_layer.get_glyph(&glyph_name).is_some() {
                        info!("Using glyph: {}", glyph_name);
                        found_glyph = Some(glyph_name);
                        break;
                    }
                }
            }

            if let Some(glyph_name) = found_glyph {
                glyph_name
            } else {
                info!("No suitable glyph found");
                return;
            }
        };

        // Get the specified glyph
        if let Some(default_layer) =
            app_state.workspace.font.ufo.get_default_layer()
        {
            if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                info!("Working with glyph: {}", glyph_name);
                info!("Checking for intersections with line: ({:.2}, {:.2}) to ({:.2}, {:.2})",
                       line.0.0, line.0.1, line.1.0, line.1.1);

                if let Some(outline) = &glyph.outline {
                    // Check each contour for intersections
                    for (i, contour) in outline.contours.iter().enumerate() {
                        let mut contour_intersections = Vec::new();

                        // Find intersections between the line and this contour
                        find_intersections(
                            contour,
                            line,
                            &mut contour_intersections,
                        );

                        if !contour_intersections.is_empty() {
                            info!(
                                "Found {} intersections with contour {}",
                                contour_intersections.len(),
                                i
                            );

                            // Convert kurbo Points to Vec2 for storage in knife_state
                            for point in contour_intersections {
                                knife_state.intersections.push(Vec2::new(
                                    point.x as f32,
                                    point.y as f32,
                                ));
                            }
                        }
                    }
                }
            } else {
                info!("Glyph not found: {}", glyph_name);
            }
        }
    }

    info!("Found {} intersections", knife_state.intersections.len());
}

/// Helper function to convert a norad::Contour to a kurbo::BezPath for intersection testing
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
                // Handle cubic Bezier curves
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

/// Perform a cut operation with the knife tool
fn perform_cut(
    start: &Vec2,
    end: &Vec2,
    shift_locked: bool,
    app_state: &mut crate::core::data::AppState,
) {
    // Get the actual endpoint, adjusted for shift locking if needed
    let actual_end = if shift_locked {
        let delta = *end - *start;
        if delta.x.abs() > delta.y.abs() {
            // Horizontal line
            Vec2::new(end.x, start.y)
        } else {
            // Vertical line
            Vec2::new(start.x, end.y)
        }
    } else {
        *end
    };

    // Create the line for intersection testing
    let line = (
        (start.x as f64, start.y as f64),
        (actual_end.x as f64, actual_end.y as f64),
    );

    info!(
        "Performing cut with line: ({:.2}, {:.2}) to ({:.2}, {:.2})",
        start.x, start.y, actual_end.x, actual_end.y
    );

    // Try to get the glyph name from different sources
    let glyph_name = if let Some(selected) = &app_state.workspace.selected {
        info!("Using glyph from workspace selection: {}", selected);
        selected.clone()
    } else {
        // Try to get some common glyphs that are likely to exist
        let common_glyphs = ["a", "A", "space", "period"];

        let mut found_glyph = None;
        for &glyph_str in common_glyphs.iter() {
            let glyph_name = norad::GlyphName::from(glyph_str);
            if let Some(default_layer) =
                app_state.workspace.font.ufo.get_default_layer()
            {
                if default_layer.get_glyph(&glyph_name).is_some() {
                    info!("Using glyph: {}", glyph_name);
                    found_glyph = Some(glyph_name);
                    break;
                }
            }
        }

        if let Some(glyph_name) = found_glyph {
            glyph_name
        } else {
            info!("No suitable glyph found");
            return;
        }
    };

    // Process the selected glyph
    info!("Working with glyph: {}", glyph_name);

    // Get mutable access to the font
    let font_obj = app_state.workspace.font_mut();

    // Get the current glyph
    if let Some(default_layer) = font_obj.ufo.get_default_layer_mut() {
        if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
            // Get or create the outline
            if let Some(outline) = glyph.outline.as_mut() {
                // Check for intersections with the line
                info!("Checking for intersections with line: ({:.2}, {:.2}) to ({:.2}, {:.2})",
                      line.0.0, line.0.1, line.1.0, line.1.1);

                // Store results of cutting operations
                let mut contours_to_remove = Vec::new();
                let mut new_contours_to_add = Vec::new();
                let mut contours_cut = 0;

                // Process each contour
                for (idx, contour) in outline.contours.iter().enumerate() {
                    let mut intersections = Vec::new();

                    // Find intersections between the line and this contour
                    find_intersections(contour, line, &mut intersections);

                    info!(
                        "Contour {}: Found {} intersections",
                        idx,
                        intersections.len()
                    );

                    // If we have any intersections, attempt to cut the contour
                    if !intersections.is_empty() {
                        info!("Attempting to cut contour {} with {} intersections", 
                             idx, intersections.len());
                        let new_contours =
                            cut_contour(contour, &intersections, line);

                        if !new_contours.is_empty() {
                            info!("Successfully cut contour {} into {} new contours", 
                                 idx, new_contours.len());
                            contours_to_remove.push(idx);
                            new_contours_to_add.extend(new_contours);
                            contours_cut += 1;
                        } else {
                            info!("Cut failed for contour {}", idx);
                        }
                    } else {
                        // No intersections found for this contour
                        info!("No intersections found for contour {}", idx);
                    }
                }

                // Apply the cuts by removing cut contours and adding new ones
                if contours_cut > 0 {
                    // Remove the contours that were cut, in reverse order to avoid index shifting
                    for idx in contours_to_remove.iter().rev() {
                        outline.contours.remove(*idx);
                    }

                    // Add the new contours
                    outline.contours.extend(new_contours_to_add);

                    info!(
                        "Cut {} contours in glyph {}",
                        contours_cut, glyph_name
                    );
                } else {
                    info!("No contours were cut in glyph {}", glyph_name);
                }
            } else {
                info!("Glyph {} has no outline", glyph_name);
            }
        } else {
            info!("Glyph not found: {}", glyph_name);
        }
    } else {
        info!("Could not get default layer");
    }
}

/// Cut a contour based on intersections with a line
fn cut_contour(
    contour: &norad::Contour,
    intersections: &Vec<kurbo::Point>,
    _line: ((f64, f64), (f64, f64)), // Prefix with underscore to indicate intentionally unused
) -> Vec<norad::Contour> {
    let mut results = Vec::new();

    // For single intersection point, we'll handle differently
    if intersections.len() == 1 {
        info!("Single intersection case - attempting to cut open the contour");
        return cut_contour_single_intersection(contour, &intersections[0]);
    }

    // Need at least 2 intersections to make a meaningful cut
    if intersections.len() < 2 {
        info!("Not enough intersections to cut contour");
        return results;
    }

    info!(
        "Cutting contour with {} points using {} intersections",
        contour.points.len(),
        intersections.len()
    );

    // Convert intersections to Vec2 for simpler handling
    let intersections_as_vec2: Vec<Vec2> = intersections
        .iter()
        .map(|p| Vec2::new(p.x as f32, p.y as f32))
        .collect();

    let points = &contour.points;
    if points.is_empty() {
        info!("Contour has no points");
        return results;
    }

    // Track if each intersection is exactly on an existing point
    let mut intersection_at_point_indices = Vec::new();
    let mut intersection_on_segment = Vec::new();

    // Check each intersection
    for intersection in &intersections_as_vec2 {
        let mut found_exact = false;

        // Check if intersection is exactly on an existing point
        for (i, point) in points.iter().enumerate() {
            let point_vec = Vec2::new(point.x, point.y);
            if is_point_near_vec2(*intersection, point_vec, 1.0) {
                intersection_at_point_indices.push(i);
                found_exact = true;
                info!("Intersection at existing point index {}", i);
                break;
            }
        }

        if !found_exact {
            // Intersection is on a segment
            let mut found_segment = false;
            for i in 0..points.len() {
                let p1 = Vec2::new(points[i].x, points[i].y);
                let p2 = Vec2::new(
                    points[(i + 1) % points.len()].x,
                    points[(i + 1) % points.len()].y,
                );

                if is_point_on_segment_vec2(*intersection, p1, p2, 1.0) {
                    intersection_on_segment.push((i, *intersection));
                    found_segment = true;
                    info!(
                        "Intersection on segment between points {} and {}",
                        i,
                        (i + 1) % points.len()
                    );
                    break;
                }
            }

            if !found_segment {
                info!(
                    "Intersection not found on any segment: {:?}",
                    intersection
                );
            }
        }
    }

    // Now create new contours based on the intersection information
    if intersection_at_point_indices.len() == 2
        && intersection_on_segment.is_empty()
    {
        // Both intersections are at existing points - simplest case
        info!("Cut case: both intersections at existing points");

        let i1 = intersection_at_point_indices[0];
        let i2 = intersection_at_point_indices[1];

        // Create first contour: from i1 to i2
        let mut first_contour_points = Vec::new();
        let mut i = i1;

        loop {
            first_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();
            if i == i2 {
                first_contour_points.push(points[i].clone());
                break;
            }
        }

        // Create second contour: from i2 to i1
        let mut second_contour_points = Vec::new();
        i = i2;

        loop {
            second_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();
            if i == i1 {
                second_contour_points.push(points[i].clone());
                break;
            }
        }

        // Check if we have enough points to make valid contours
        let point_count1 = first_contour_points.len();
        let point_count2 = second_contour_points.len();

        // Only add contours with at least 3 points
        if point_count1 >= 3 {
            info!("Adding first contour with {} points", point_count1);
            results.push(norad::Contour::new(first_contour_points, None, None));
        } else {
            info!("First contour has only {} points, not adding", point_count1);
        }

        if point_count2 >= 3 {
            info!("Adding second contour with {} points", point_count2);
            results.push(norad::Contour::new(
                second_contour_points,
                None,
                None,
            ));
        } else {
            info!(
                "Second contour has only {} points, not adding",
                point_count2
            );
        }
    } else if intersection_at_point_indices.len() == 1
        && intersection_on_segment.len() == 1
    {
        // One intersection at point, one on segment
        info!("Cut case: one intersection at point, one on segment");

        let point_idx = intersection_at_point_indices[0];
        let (segment_idx, segment_point) = intersection_on_segment[0];

        // Create a new point for the segment intersection
        let new_point = norad::ContourPoint::new(
            segment_point.x,
            segment_point.y,
            norad::PointType::Line,
            false, // not smooth
            None,  // no name
            None,  // no identifier
            None,  // no metadata
        );

        // Create first contour: from point_idx to segment
        let mut first_contour_points = Vec::new();
        let mut i = point_idx;

        loop {
            first_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();
            if i == (segment_idx + 1) % points.len() {
                // Insert our new point before continuing
                first_contour_points.push(new_point.clone());
                break;
            }

            // Add the segment point if we're at the segment
            if i == segment_idx + 1 {
                first_contour_points.push(new_point.clone());
            }
        }

        // Create second contour: from segment to point_idx
        let mut second_contour_points = Vec::new();
        second_contour_points.push(new_point);

        i = (segment_idx + 1) % points.len();

        while i != point_idx {
            second_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();
        }

        second_contour_points.push(points[point_idx].clone());

        // Check if we have enough points to make valid contours
        let point_count1 = first_contour_points.len();
        let point_count2 = second_contour_points.len();

        // Only add contours with at least 3 points
        if point_count1 >= 3 {
            info!("Adding first contour with {} points", point_count1);
            results.push(norad::Contour::new(first_contour_points, None, None));
        } else {
            info!("First contour has only {} points, not adding", point_count1);
        }

        if point_count2 >= 3 {
            info!("Adding second contour with {} points", point_count2);
            results.push(norad::Contour::new(
                second_contour_points,
                None,
                None,
            ));
        } else {
            info!(
                "Second contour has only {} points, not adding",
                point_count2
            );
        }
    } else if intersection_at_point_indices.is_empty()
        && intersection_on_segment.len() == 2
    {
        // Both intersections on segments
        info!("Cut case: both intersections on segments");

        let (segment1_idx, segment1_point) = intersection_on_segment[0];
        let (segment2_idx, segment2_point) = intersection_on_segment[1];

        // Create new points for the segment intersections
        let new_point1 = norad::ContourPoint::new(
            segment1_point.x,
            segment1_point.y,
            norad::PointType::Line,
            false, // not smooth
            None,  // no name
            None,  // no identifier
            None,  // no metadata
        );

        let new_point2 = norad::ContourPoint::new(
            segment2_point.x,
            segment2_point.y,
            norad::PointType::Line,
            false, // not smooth
            None,  // no name
            None,  // no identifier
            None,  // no metadata
        );

        // Create first contour: from segment1 to segment2
        let mut first_contour_points = Vec::new();
        first_contour_points.push(new_point1.clone());

        let mut i = (segment1_idx + 1) % points.len();

        while i != (segment2_idx + 1) % points.len() {
            first_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();

            // Don't add the second point yet
            if i == segment2_idx + 1 {
                break;
            }
        }

        first_contour_points.push(new_point2.clone());

        // Create second contour: from segment2 to segment1
        let mut second_contour_points = Vec::new();
        second_contour_points.push(new_point2);

        i = (segment2_idx + 1) % points.len();

        while i != (segment1_idx + 1) % points.len() {
            second_contour_points.push(points[i].clone());
            i = (i + 1) % points.len();

            // Don't add the first point yet
            if i == segment1_idx + 1 {
                break;
            }
        }

        second_contour_points.push(new_point1);

        // Check if we have enough points to make valid contours
        let point_count1 = first_contour_points.len();
        let point_count2 = second_contour_points.len();

        // Only add contours with at least 3 points
        if point_count1 >= 3 {
            info!("Adding first contour with {} points", point_count1);
            results.push(norad::Contour::new(first_contour_points, None, None));
        } else {
            info!("First contour has only {} points, not adding", point_count1);
        }

        if point_count2 >= 3 {
            info!("Adding second contour with {} points", point_count2);
            results.push(norad::Contour::new(
                second_contour_points,
                None,
                None,
            ));
        } else {
            info!(
                "Second contour has only {} points, not adding",
                point_count2
            );
        }
    } else {
        // Complex case or more than 2 intersections - not handling
        info!("Complex case with {} point intersections and {} segment intersections - not supported",
             intersection_at_point_indices.len(), intersection_on_segment.len());
    }

    info!("Created {} new contours from cut operation", results.len());
    results
}

/// Cut a contour at a single intersection point, opening it up
fn cut_contour_single_intersection(
    contour: &norad::Contour,
    intersection: &kurbo::Point,
) -> Vec<norad::Contour> {
    let mut results = Vec::new();
    let points = &contour.points;

    if points.is_empty() {
        info!("Contour has no points to cut at single intersection");
        return results;
    }

    // Convert intersection to Vec2
    let intersection_vec2 =
        Vec2::new(intersection.x as f32, intersection.y as f32);

    // Check if intersection is exactly on an existing point
    for (i, point) in points.iter().enumerate() {
        let point_vec = Vec2::new(point.x, point.y);
        if is_point_near_vec2(intersection_vec2, point_vec, 1.0) {
            // Create a new open contour starting and ending at this point
            let mut new_points = Vec::new();

            // Start from the intersection point and go all the way around
            for idx in 0..points.len() {
                let adjusted_idx = (i + idx) % points.len();
                new_points.push(points[adjusted_idx].clone());
            }

            info!("Created open contour starting at intersection point {}", i);
            let point_count = new_points.len();

            if point_count >= 2 {
                // Create the contour and mark it as open
                let new_contour = norad::Contour::new(new_points, None, None);
                // There's no direct way to set a contour as open/closed in norad API
                // The contour is considered closed by default, so we'll add it anyway
                results.push(new_contour);
            } else {
                info!(
                    "Not enough points ({}) to create a valid contour",
                    point_count
                );
            }

            return results;
        }
    }

    // Intersection is on a segment, need to insert a new point
    for i in 0..points.len() {
        let p1 = Vec2::new(points[i].x, points[i].y);
        let p2 = Vec2::new(
            points[(i + 1) % points.len()].x,
            points[(i + 1) % points.len()].y,
        );

        if is_point_on_segment_vec2(intersection_vec2, p1, p2, 1.0) {
            // Create a new open contour with the intersection point duplicated
            let mut new_points = Vec::new();

            // Add all points up to the segment
            for j in 0..=i {
                new_points.push(points[j].clone());
            }

            // Add the intersection point
            let new_point = norad::ContourPoint::new(
                intersection_vec2.x,
                intersection_vec2.y,
                norad::PointType::Line,
                false, // not smooth
                None,  // no name
                None,  // no identifier
                None,  // no metadata
            );
            new_points.push(new_point.clone());

            // Add the rest of the points
            for j in (i + 1)..points.len() {
                new_points.push(points[j].clone());
            }

            // Add the intersection point again at the end to complete the circle
            new_points.push(new_point);

            info!("Created open contour by inserting intersection point between {} and {}", 
                  i, (i + 1) % points.len());
            let point_count = new_points.len();

            if point_count >= 2 {
                // Create the contour
                let new_contour = norad::Contour::new(new_points, None, None);
                results.push(new_contour);
            } else {
                info!(
                    "Not enough points ({}) to create a valid contour",
                    point_count
                );
            }

            return results;
        }
    }

    info!("Could not find intersection point on contour");
    results
}

/// Helper function to check if a point is near another point
fn is_point_near_vec2(p1: Vec2, p2: Vec2, threshold: f32) -> bool {
    (p1.x - p2.x).abs() < threshold && (p1.y - p2.y).abs() < threshold
}

/// Helper function to check if a point is on a line segment
fn is_point_on_segment_vec2(
    point: Vec2,
    line_start: Vec2,
    line_end: Vec2,
    threshold: f32,
) -> bool {
    // Calculate the distance from the point to the line
    let line_vector = line_end - line_start;
    let point_vector = point - line_start;

    // Project the point vector onto the line vector
    let line_length_squared = line_vector.length_squared();

    // Avoid division by zero
    if line_length_squared < 1e-6 {
        return false;
    }

    let t = point_vector.dot(line_vector) / line_length_squared;

    // If t is between 0 and 1, the projection is on the line segment
    if t < 0.0 || t > 1.0 {
        return false;
    }

    // Calculate the projection point
    let projection = line_start + line_vector * t;

    // Calculate the distance from the original point to the projection
    let distance = (point - projection).length();

    // If the distance is less than the threshold, the point is on the segment
    distance < threshold
}

/// Find intersections between a BezPath and a Line
fn find_intersections_bezpath(
    path: &BezPath,
    line: ((f64, f64), (f64, f64)),
    intersections: &mut Vec<kurbo::Point>,
) {
    let kurbo_line = kurbo::Line::new(
        kurbo::Point::new(line.0 .0, line.0 .1),
        kurbo::Point::new(line.1 .0, line.1 .1),
    );

    // Clear existing intersections
    intersections.clear();

    // Iterate through each segment of the path
    for segment in path.segments() {
        let segment_intersections = segment.intersect_line(kurbo_line);
        info!(
            "Segment check found {} intersections",
            segment_intersections.len()
        );

        for t in segment_intersections {
            // Get the point on the segment at parameter t
            let point = segment.eval(t.segment_t);

            // Check if this point is already in our list (with a small tolerance)
            let is_duplicate = intersections.iter().any(|p| {
                (p.x - point.x).abs() < 1.0 && (p.y - point.y).abs() < 1.0
            });

            if !is_duplicate {
                info!(
                    "Adding intersection point: ({:.2}, {:.2})",
                    point.x, point.y
                );
                intersections.push(point);
            }
        }
    }

    // Sort the intersections along the line for consistent processing
    if intersections.len() > 1 {
        sort_intersections_along_line(intersections, line);
    }
}

/// Find intersections between a Contour and a Line
fn find_intersections(
    contour: &norad::Contour,
    line: ((f64, f64), (f64, f64)),
    intersections: &mut Vec<kurbo::Point>,
) {
    // First convert the contour to a BezPath
    if let Some(path) = convert_contour_to_bezpath(contour) {
        find_intersections_bezpath(&path, line, intersections);
    } else {
        info!("Failed to convert contour to BezPath for intersection testing");
    }
}

/// Sort intersections along a line to ensure consistent ordering
fn sort_intersections_along_line(
    intersections: &mut Vec<kurbo::Point>,
    line: ((f64, f64), (f64, f64)),
) {
    let kurbo_line = kurbo::Line::new(
        kurbo::Point::new(line.0 .0, line.0 .1),
        kurbo::Point::new(line.1 .0, line.1 .1),
    );

    intersections.sort_by(|a, b| {
        let a_param = project_point_onto_line(*a, kurbo_line);
        let b_param = project_point_onto_line(*b, kurbo_line);
        a_param
            .partial_cmp(&b_param)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Project a point onto a line and return the parameter value along the line
fn project_point_onto_line(point: kurbo::Point, line: kurbo::Line) -> f64 {
    let line_vec = line.end() - line.start();
    let point_vec = point - line.start();

    // Calculate the projection parameter
    let line_length_squared = line_vec.dot(line_vec);
    if line_length_squared < 1e-10 {
        return 0.0; // Avoid division by near-zero
    }

    // Calculate the parameter value (t) along the line
    let t = point_vec.dot(line_vec) / line_length_squared;
    t.clamp(0.0, 1.0)
}
