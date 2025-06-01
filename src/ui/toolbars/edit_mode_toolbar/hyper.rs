use super::EditModeSystem;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint, PointType};

/// Resource to track if hyper pen mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct HyperModeActive(pub bool);

/// Plugin to register hyper pen mode systems
pub struct HyperModePlugin;

impl Plugin for HyperModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HyperToolState>()
            .init_resource::<HyperModeActive>()
            .add_systems(
                Update,
                (
                    handle_hyper_mouse_events,
                    render_hyper_preview,
                    handle_hyper_keyboard_events,
                    reset_hyper_mode_when_inactive,
                ),
            );
    }
}

/// The state of the hyper pen tool
#[derive(Resource)]
pub struct HyperToolState {
    /// Whether the hyper pen tool is active
    pub active: bool,
    /// The state of the hyper pen tool
    pub state: HyperState,
    /// The current path being drawn
    pub current_path: Option<BezPath>,
    /// Points already placed in the current path
    pub points: Vec<Vec2>,
    /// Control points for the hyperbezier curve
    pub control_points: Vec<Vec2>,
    /// Whether each point is smooth or a corner
    pub is_smooth: Vec<bool>,
    /// The current cursor position
    pub cursor_position: Option<Vec2>,
    /// How close to the start point to close the path
    pub close_path_threshold: f32,
}

impl Default for HyperToolState {
    fn default() -> Self {
        Self {
            active: true,
            state: HyperState::default(),
            current_path: None,
            points: Vec::new(),
            control_points: Vec::new(),
            is_smooth: Vec::new(),
            cursor_position: None,
            close_path_threshold: 15.0,
        }
    }
}

/// The state of the hyper pen tool
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub(crate) enum HyperState {
    /// Ready to place a point
    #[default]
    Ready,
    /// Drawing a path with points
    Drawing,
    /// Dragging a control point
    _DraggingControl,
}

/// Hyper Pen mode for drawing paths with smooth curves
pub struct HyperMode;

impl EditModeSystem for HyperMode {
    fn update(&self, commands: &mut Commands) {
        // Mark hyper pen mode as active
        commands.insert_resource(HyperModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entering Hyper Pen Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Hyper Pen Mode");
    }
}

/// System to handle deactivation of hyper pen mode when another mode is selected
pub fn reset_hyper_mode_when_inactive(
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
    mut hyper_state: ResMut<HyperToolState>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
) {
    if current_mode.0 != crate::ui::toolbars::edit_mode_toolbar::EditMode::Hyper
    {
        // Commit any open path before deactivating
        if hyper_state.state == HyperState::Drawing
            && !hyper_state.points.is_empty()
            && hyper_state.points.len() >= 2
        {
            if let Some(_contour) = create_contour_from_points_when_closing(
                &hyper_state.points,
                &hyper_state.is_smooth,
                &hyper_state.control_points,
            ) {
                // Signal that we've made a change to the glyph
                app_state_changed.send(crate::rendering::draw::AppStateChanged);
                info!("Committing hyperbezier path on mode change");
            }
        }

        // Clear state and mark inactive
        *hyper_state = HyperToolState::default();
        hyper_state.active = false;
        commands.insert_resource(HyperModeActive(false));
    }
}

/// System to handle mouse events for the hyper pen tool
#[allow(clippy::too_many_arguments)]
pub fn handle_hyper_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    camera_q: Query<
        (&Camera, &GlobalTransform),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hyper_state: ResMut<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    mut app_state: ResMut<crate::core::state::AppState>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // Only handle events when in hyper pen mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
            return;
        }
    }

    // Don't process drawing events when hovering over UI
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
        warn!("No active camera found for hyper pen tool");
        return;
    };

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                Ok(world_position) => {
                    hyper_state.cursor_position = Some(world_position);
                }
                Err(_) => {}
            }
        }
    }

    // Handle left click for adding points
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = hyper_state.cursor_position {
            // Get modifier states
            let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight);
            let alt_pressed = keyboard.pressed(KeyCode::AltLeft)
                || keyboard.pressed(KeyCode::AltRight);

            // Apply snap to grid if enabled
            let world_pos = if SNAP_TO_GRID_ENABLED {
                Vec2::new(
                    (world_pos.x / SNAP_TO_GRID_VALUE).round()
                        * SNAP_TO_GRID_VALUE,
                    (world_pos.y / SNAP_TO_GRID_VALUE).round()
                        * SNAP_TO_GRID_VALUE,
                )
            } else {
                world_pos
            };

            // Adjust position based on shift (for axis alignment)
            let adjusted_pos =
                if shift_pressed && !hyper_state.points.is_empty() {
                    let last_point = hyper_state.points.last().unwrap();
                    axis_lock_position(world_pos, *last_point)
                } else {
                    world_pos
                };

            match hyper_state.state {
                HyperState::Ready => {
                    // Start a new path
                    let mut path = BezPath::new();
                    path.move_to((
                        adjusted_pos.x as f64,
                        adjusted_pos.y as f64,
                    ));
                    hyper_state.current_path = Some(path);
                    hyper_state.points.push(adjusted_pos);
                    hyper_state.is_smooth.push(!alt_pressed); // Default to smooth points
                    hyper_state.state = HyperState::Drawing;

                    info!(
                        "Started new hyperbezier path at: {:?}",
                        adjusted_pos
                    );
                }
                HyperState::Drawing => {
                    // Check if clicking near start point to close the path
                    if !hyper_state.points.is_empty() {
                        let start_point = hyper_state.points[0];
                        let distance = start_point.distance(adjusted_pos);

                        if distance < hyper_state.close_path_threshold
                            && hyper_state.points.len() > 1
                        {
                            info!("Closing hyperbezier path - clicked near start point");

                            // We need to connect the last point to the first point with a proper curve
                            // First get the first and last points
                            let first_point = hyper_state.points[0];
                            let last_point =
                                *hyper_state.points.last().unwrap();

                            // Calculate control points for the closing segment
                            let direction = first_point - last_point;
                            let distance = direction.length();
                            let control_scale = distance * 0.33;
                            let normalized_dir = direction.normalize_or_zero();

                            // Create control points for the closing segment
                            let control1 =
                                last_point + normalized_dir * control_scale;
                            let control2 =
                                first_point - normalized_dir * control_scale;

                            // Add these control points to the list
                            hyper_state.control_points.push(control1);
                            hyper_state.control_points.push(control2);

                            // Close the path in the BezPath
                            if let Some(ref mut path) = hyper_state.current_path
                            {
                                // Add the final curve segment first
                                path.curve_to(
                                    (control1.x as f64, control1.y as f64),
                                    (control2.x as f64, control2.y as f64),
                                    (
                                        first_point.x as f64,
                                        first_point.y as f64,
                                    ),
                                );
                                // Then close it
                                path.close_path();
                            }

                            // Convert to contour and add to glyph
                            if let Some(contour) =
                                create_contour_from_points_when_closing(
                                    &hyper_state.points,
                                    &hyper_state.is_smooth,
                                    &hyper_state.control_points,
                                )
                            {
                                // Add contour to the current glyph
                                commit_contour_to_glyph(
                                    contour,
                                    &glyph_navigation,
                                    &mut app_state,
                                    &mut app_state_changed,
                                );
                            }

                            // Reset for next path
                            hyper_state.current_path = None;
                            hyper_state.points.clear();
                            hyper_state.is_smooth.clear();
                            hyper_state.control_points.clear();
                            hyper_state.state = HyperState::Ready;
                        } else {
                            // Add point to existing path - extract data before borrowing
                            if hyper_state.points.is_empty() {
                                return;
                            }

                            // Get the last point
                            let prev_point =
                                *hyper_state.points.last().unwrap();
                            let is_smooth = !alt_pressed;

                            // Calculate hyperbezier control points
                            // For a smooth, G2 continuous curve, we want the control points
                            // to be positioned to maintain curvature between segments
                            let direction = adjusted_pos - prev_point;
                            let distance = direction.length();

                            // In hyperbezier, the control points are positioned more
                            // intelligently than in regular beziers
                            // For our MVP, use distance-based scaling for more natural curves
                            let control_scale = distance * 0.33; // Use 1/3 distance for control points
                            let normalized_dir = direction.normalize_or_zero();

                            // First control point extends from previous point
                            let control1 =
                                prev_point + normalized_dir * control_scale;

                            // Second control point comes back from current point
                            let control2 =
                                adjusted_pos - normalized_dir * control_scale;

                            // Now borrow the path to add curve
                            if let Some(ref mut path) = hyper_state.current_path
                            {
                                path.curve_to(
                                    (control1.x as f64, control1.y as f64),
                                    (control2.x as f64, control2.y as f64),
                                    (
                                        adjusted_pos.x as f64,
                                        adjusted_pos.y as f64,
                                    ),
                                );
                            }

                            // Store the point and other data after the path borrow is dropped
                            hyper_state.points.push(adjusted_pos);
                            hyper_state.is_smooth.push(is_smooth);
                            hyper_state.control_points.push(control1);
                            hyper_state.control_points.push(control2);

                            info!(
                                "Added point to hyperbezier path: {:?}",
                                adjusted_pos
                            );
                        }
                    }
                }
                HyperState::_DraggingControl => {
                    // Finish dragging control point
                    hyper_state.state = HyperState::Drawing;
                }
            }
        }
    }

    // Double click to toggle point type (smooth/corner)
    if mouse_button_input.pressed(MouseButton::Left)
        && mouse_button_input.just_pressed(MouseButton::Left)
    {
        if let Some(world_pos) = hyper_state.cursor_position {
            if hyper_state.state == HyperState::Drawing
                && !hyper_state.points.is_empty()
            {
                // Check if we clicked near an existing point
                for (i, point) in hyper_state.points.iter().enumerate() {
                    if point.distance(world_pos) < 10.0 {
                        // Toggle the point type
                        hyper_state.is_smooth[i] = !hyper_state.is_smooth[i];
                        info!("Toggled point type at index {}", i);

                        // Recalculate the path
                        update_hyperbezier_path(&mut hyper_state);
                        break;
                    }
                }
            }
        }
    }

    // Handle right click to finish path without closing
    if mouse_button_input.just_pressed(MouseButton::Right) {
        if hyper_state.state == HyperState::Drawing
            && hyper_state.points.len() >= 2
        {
            info!("Finishing open hyperbezier path with right click");

            // Convert to contour and add to glyph
            if let Some(contour) = create_contour_from_points_when_closing(
                &hyper_state.points,
                &hyper_state.is_smooth,
                &hyper_state.control_points,
            ) {
                // Add contour to the current glyph
                commit_contour_to_glyph(
                    contour,
                    &glyph_navigation,
                    &mut app_state,
                    &mut app_state_changed,
                );
            }

            // Reset for next path
            hyper_state.current_path = None;
            hyper_state.points.clear();
            hyper_state.is_smooth.clear();
            hyper_state.control_points.clear();
            hyper_state.state = HyperState::Ready;
        }
    }
}

/// Update the hyperbezier path when points are modified
fn update_hyperbezier_path(hyper_state: &mut HyperToolState) {
    if hyper_state.points.len() < 2 {
        return;
    }

    // Create a temporary structure to hold our calculated control points
    let mut temp_control_points =
        Vec::with_capacity(hyper_state.points.len() * 2);

    // First calculate all control points without modifying the path
    let points = &hyper_state.points;
    let is_smooth = &hyper_state.is_smooth;

    for i in 1..points.len() {
        let prev = points[i - 1];
        let current = points[i];

        // Get the direction and distance between these points
        let direction = current - prev;
        let distance = direction.length();
        let control_scale = distance * 0.33;

        // Calculate control points based on smoothness
        if i == 1 || !is_smooth[i - 1] {
            // First segment or previous point is a corner - use standard control points
            let normalized_dir = direction.normalize_or_zero();
            let control1 = prev + normalized_dir * control_scale;
            let control2 = current - normalized_dir * control_scale;

            temp_control_points.push(control1);
            temp_control_points.push(control2);
        } else {
            // Previous point is smooth - need to maintain continuity

            // For the first control point, we need to reflect the previous segment's
            // last control point through the shared on-curve point
            let control1 = if i >= 2 && !temp_control_points.is_empty() {
                // Get previous segment's last control point
                let prev_control2 = temp_control_points[(i - 2) * 2 + 1];

                // Reflect it through prev point to maintain tangent continuity
                let reflection_vector = prev - prev_control2;
                prev + reflection_vector.normalize_or_zero() * control_scale
            } else {
                // No previous segment, use standard control
                prev + direction.normalize_or_zero() * control_scale
            };

            // For the second control point, standard approach
            let control2 = if !is_smooth[i] {
                // Corner point ahead - use standard control
                current - direction.normalize_or_zero() * control_scale
            } else {
                // Smooth point ahead
                current - direction.normalize_or_zero() * control_scale
            };

            temp_control_points.push(control1);
            temp_control_points.push(control2);
        }
    }

    // Now recreate the path
    let mut path = BezPath::new();
    path.move_to((points[0].x as f64, points[0].y as f64));

    // Add the remaining points as curves using our pre-calculated control points
    for i in 1..points.len() {
        let j = (i - 1) * 2;
        if j + 1 < temp_control_points.len() {
            let control1 = temp_control_points[j];
            let control2 = temp_control_points[j + 1];
            let current = points[i];

            path.curve_to(
                (control1.x as f64, control1.y as f64),
                (control2.x as f64, control2.y as f64),
                (current.x as f64, current.y as f64),
            );
        }
    }

    // Finally, update the state
    hyper_state.current_path = Some(path);
    hyper_state.control_points = temp_control_points;
}

/// System to render a preview of the hyper pen tool's current path
pub fn render_hyper_preview(
    mut gizmos: Gizmos,
    hyper_state: Res<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
) {
    // Only render when in hyper pen mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
            return;
        }
    }

    // Define colors
    let point_color = Color::srgb(1.0, 1.0, 0.0); // Yellow for regular points
    let smooth_color = Color::srgb(0.0, 1.0, 1.0); // Cyan for smooth points
    let corner_color = Color::srgb(1.0, 0.5, 0.0); // Orange for corner points
    let preview_color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Translucent white
    let control_color = Color::srgba(0.7, 0.7, 1.0, 0.5); // Light blue for control points
    let close_highlight_color = Color::srgb(0.2, 1.0, 0.3); // Green for close indicator

    // Visualization parameters
    let point_size = 5.0;
    let control_size = 3.0;

    // Draw points and lines between them
    for (i, point) in hyper_state.points.iter().enumerate() {
        // Draw point with color based on type
        let color = if i == 0 {
            Color::srgb(0.0, 1.0, 0.5) // Highlight start point
        } else if i < hyper_state.is_smooth.len() && hyper_state.is_smooth[i] {
            smooth_color
        } else if i < hyper_state.is_smooth.len() {
            corner_color
        } else {
            point_color
        };

        gizmos.circle_2d(*point, point_size, color);

        // Draw control points and handles if we have them
        if i > 0 && hyper_state.control_points.len() >= (i - 1) * 2 + 2 {
            let ctrl1 = hyper_state.control_points[(i - 1) * 2];
            let ctrl2 = hyper_state.control_points[(i - 1) * 2 + 1];

            // Draw control points
            gizmos.circle_2d(ctrl1, control_size, control_color);
            gizmos.circle_2d(ctrl2, control_size, control_color);

            // Draw handles to control points
            if i > 0 {
                gizmos.line_2d(hyper_state.points[i - 1], ctrl1, control_color);
                gizmos.line_2d(*point, ctrl2, control_color);
            }
        }
    }

    // Draw preview line from last point to cursor
    if let (Some(cursor_pos), true) =
        (hyper_state.cursor_position, !hyper_state.points.is_empty())
    {
        let last_point = *hyper_state.points.last().unwrap();

        // Apply snap to grid for preview if enabled
        let preview_cursor_pos = if SNAP_TO_GRID_ENABLED {
            Vec2::new(
                (cursor_pos.x / SNAP_TO_GRID_VALUE).round()
                    * SNAP_TO_GRID_VALUE,
                (cursor_pos.y / SNAP_TO_GRID_VALUE).round()
                    * SNAP_TO_GRID_VALUE,
            )
        } else {
            cursor_pos
        };

        gizmos.line_2d(last_point, preview_cursor_pos, preview_color);

        // Check if cursor is near start point (for closing path)
        if hyper_state.points.len() > 1 {
            let start_point = hyper_state.points[0];
            let distance = start_point.distance(preview_cursor_pos);

            if distance < hyper_state.close_path_threshold {
                // Draw highlight to indicate path can be closed
                gizmos.circle_2d(
                    start_point,
                    hyper_state.close_path_threshold,
                    Color::srgba(0.2, 1.0, 0.3, 0.3),
                );
                gizmos.line_2d(last_point, start_point, close_highlight_color);
            }
        }
    }

    // Draw cursor position
    if let Some(cursor_pos) = hyper_state.cursor_position {
        // Apply snap to grid for cursor display if enabled
        let display_cursor_pos = if SNAP_TO_GRID_ENABLED {
            Vec2::new(
                (cursor_pos.x / SNAP_TO_GRID_VALUE).round()
                    * SNAP_TO_GRID_VALUE,
                (cursor_pos.y / SNAP_TO_GRID_VALUE).round()
                    * SNAP_TO_GRID_VALUE,
            )
        } else {
            cursor_pos
        };

        gizmos.circle_2d(
            display_cursor_pos,
            3.0,
            Color::srgba(1.0, 1.0, 1.0, 0.7),
        );
    }
}

/// System to handle keyboard events for the hyper pen tool
pub fn handle_hyper_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hyper_state: ResMut<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
) {
    // Only handle events when in hyper pen mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
            return;
        }
    }

    // Handle Escape key to cancel current path
    if keyboard.just_pressed(KeyCode::Escape) {
        hyper_state.current_path = None;
        hyper_state.points.clear();
        hyper_state.is_smooth.clear();
        hyper_state.control_points.clear();
        hyper_state.state = HyperState::Ready;
        info!("Cancelled current hyperbezier path with Escape key");
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

/// Create a contour from a list of points
#[allow(dead_code)]
fn create_contour_from_points(
    points: &[Vec2],
    is_smooth: &[bool],
) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    // Create a temporary structure to hold our calculated control points
    let mut temp_control_points = Vec::with_capacity((points.len() - 1) * 2);

    // Calculate all control points for all segments
    for i in 1..points.len() {
        let prev = points[i - 1];
        let current = points[i];

        // Get the direction and distance between these points
        let direction = current - prev;
        let distance = direction.length();
        let control_scale = distance * 0.33;
        let normalized_dir = direction.normalize_or_zero();

        if i == 1 || !is_smooth[i - 1] {
            // First segment or previous point is a corner - use standard control points
            let control1 = prev + normalized_dir * control_scale;
            let control2 = current - normalized_dir * control_scale;

            temp_control_points.push(control1);
            temp_control_points.push(control2);
        } else {
            // Previous point is smooth - need to maintain continuity

            // For the first control point, reflect previous control point through shared point
            let control1 = if i >= 2 && !temp_control_points.is_empty() {
                let prev_control2 = temp_control_points[(i - 2) * 2 + 1];

                // Reflect through prev point to maintain tangent continuity
                let reflection_vector = prev - prev_control2;
                let _reflection_length = reflection_vector.length();

                // Use normalized reflection vector with proper scale
                prev + reflection_vector.normalize_or_zero() * control_scale
            } else {
                // No previous segment, use standard control
                prev + normalized_dir * control_scale
            };

            // For the second control point, standard approach
            let control2 = current - normalized_dir * control_scale;

            temp_control_points.push(control1);
            temp_control_points.push(control2);
        }
    }

    // Now build final contour points with all necessary control points
    let mut contour_points = Vec::new();

    // Add the first point as a move point (always on-curve)
    contour_points.push(ContourPoint::new(
        points[0].x,
        points[0].y,
        PointType::Move,
        is_smooth[0], // First point's smoothness
        None,
        None,
        None,
    ));

    // Now add all segments with their control points
    for i in 1..points.len() {
        // Current on-curve point
        let curr_point = points[i];
        let is_smooth_point = if i < is_smooth.len() {
            is_smooth[i]
        } else {
            true
        };

        // Get control points for this segment
        let control_idx = (i - 1) * 2;
        if control_idx + 1 < temp_control_points.len() {
            // We have control points for this segment - use them to make a curve segment
            let control1 = temp_control_points[control_idx];
            let control2 = temp_control_points[control_idx + 1];

            // Add first control point
            contour_points.push(ContourPoint::new(
                control1.x,
                control1.y,
                PointType::OffCurve,
                false, // Control points are never smooth
                None,
                None,
                None,
            ));

            // Add second control point
            contour_points.push(ContourPoint::new(
                control2.x,
                control2.y,
                PointType::OffCurve,
                false, // Control points are never smooth
                None,
                None,
                None,
            ));

            // Add the on-curve point as a Curve type
            contour_points.push(ContourPoint::new(
                curr_point.x,
                curr_point.y,
                PointType::Curve, // This is a curve on-curve point
                is_smooth_point,
                None,
                None,
                None,
            ));
        } else {
            // Fallback to line segment if no control points (shouldn't happen normally)
            contour_points.push(ContourPoint::new(
                curr_point.x,
                curr_point.y,
                PointType::Line,
                is_smooth_point,
                None,
                None,
                None,
            ));
        }
    }

    // Create the contour
    Some(Contour::new(contour_points, None, None))
}

/// Helper function to commit a contour to the current glyph
fn commit_contour_to_glyph(
    contour: Contour,
    glyph_navigation: &crate::core::state::GlyphNavigation,
    app_state: &mut crate::core::state::AppState,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
) {
    // Add contour to the current glyph
    if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo)
    {
        let glyph_name = glyph_name.clone();

        // Get mutable access to the font
        let font_obj = app_state.workspace.font_mut();

        // Get the current glyph
        if let Some(default_layer) = font_obj.ufo.get_default_layer_mut() {
            if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
                // Get or create the outline
                let outline = glyph.outline.get_or_insert_with(|| {
                    norad::glyph::Outline {
                        contours: Vec::new(),
                        components: Vec::new(),
                    }
                });

                // Add the new contour
                outline.contours.push(contour);
                info!("Added new hyperbezier contour to glyph {}", glyph_name);

                // Notify that the app state has changed
                app_state_changed.send(crate::rendering::draw::AppStateChanged);
            }
        }
    }
}

/// Calculate auto control points for a smooth curve
#[allow(dead_code)]
fn calculate_auto_control_point(points: &[Vec2], control_index: usize) -> Vec2 {
    if points.len() < 2 {
        return Vec2::ZERO;
    }

    let last_index = points.len() - 1;
    let prev_point = points[last_index];
    let next_point = *points.last().unwrap();

    // Simple calculation for auto control points
    // In a real implementation, this would use the hyperbezier algorithm
    let direction = next_point - prev_point;
    let third = direction / 3.0;

    if control_index == 0 {
        prev_point + third
    } else {
        next_point - third
    }
}

/// Special function to create a contour from a list of points when closing a path
fn create_contour_from_points_when_closing(
    points: &[Vec2],
    is_smooth: &[bool],
    control_points: &[Vec2],
) -> Option<Contour> {
    if points.len() < 2 || control_points.len() < 2 {
        return None;
    }

    // We'll build the contour directly from the on-curve and control points
    let mut contour_points = Vec::new();

    // Add the first point as a move point (always on-curve)
    contour_points.push(ContourPoint::new(
        points[0].x,
        points[0].y,
        PointType::Move,
        is_smooth[0], // First point's smoothness
        None,
        None,
        None,
    ));

    // For each segment, add both control points and the destination on-curve point
    for i in 1..points.len() {
        let control_idx = (i - 1) * 2;
        if control_idx + 1 < control_points.len() {
            // Get the control points for this segment
            let control1 = control_points[control_idx];
            let control2 = control_points[control_idx + 1];

            // Add first control point
            contour_points.push(ContourPoint::new(
                control1.x,
                control1.y,
                PointType::OffCurve,
                false, // Control points are never smooth
                None,
                None,
                None,
            ));

            // Add second control point
            contour_points.push(ContourPoint::new(
                control2.x,
                control2.y,
                PointType::OffCurve,
                false, // Control points are never smooth
                None,
                None,
                None,
            ));

            // Add the on-curve point as a Curve type
            contour_points.push(ContourPoint::new(
                points[i].x,
                points[i].y,
                PointType::Curve, // This is a curve on-curve point
                if i < is_smooth.len() {
                    is_smooth[i]
                } else {
                    true
                },
                None,
                None,
                None,
            ));
        }
    }

    // If we have a closing segment (from last to first point), add it
    if control_points.len() >= (points.len() - 1) * 2 + 2 {
        // Get the control points for the closing segment
        let control1 = control_points[(points.len() - 1) * 2];
        let control2 = control_points[(points.len() - 1) * 2 + 1];

        // Add first control point
        contour_points.push(ContourPoint::new(
            control1.x,
            control1.y,
            PointType::OffCurve,
            false, // Control points are never smooth
            None,
            None,
            None,
        ));

        // Add second control point
        contour_points.push(ContourPoint::new(
            control2.x,
            control2.y,
            PointType::OffCurve,
            false, // Control points are never smooth
            None,
            None,
            None,
        ));

        // We don't need to add the first point again since it's already there,
        // and the contour is marked as closed by default
    }

    // Create the contour as closed
    Some(Contour::new(contour_points, None, None))
}
