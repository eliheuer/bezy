use super::EditModeSystem;
use crate::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
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
pub(crate) enum HyperState {
    /// Ready to place a point
    #[default]
    Ready,
    /// Drawing a path with points
    Drawing,
    /// Dragging a control point
    DraggingControl,
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
    current_mode: Res<crate::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
    mut hyper_state: ResMut<HyperToolState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
) {
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Hyper {
        // Commit any open path before deactivating
        if hyper_state.state == HyperState::Drawing
            && !hyper_state.points.is_empty()
            && hyper_state.points.len() >= 2
        {
            if let Some(_contour) =
                create_contour_from_points(&hyper_state.points, &hyper_state.is_smooth)
            {
                // Signal that we've made a change to the glyph
                app_state_changed.send(crate::draw::AppStateChanged);
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
        With<crate::cameras::DesignCamera>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hyper_state: ResMut<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
    cli_args: Res<crate::cli::CliArgs>,
    mut app_state: ResMut<crate::data::AppState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
    ui_hover_state: Res<crate::ui_interaction::UiHoverState>,
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
                    (world_pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                    (world_pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                )
            } else {
                world_pos
            };

            // Adjust position based on shift (for axis alignment)
            let adjusted_pos = if shift_pressed && !hyper_state.points.is_empty() {
                let last_point = hyper_state.points.last().unwrap();
                axis_lock_position(world_pos, *last_point)
            } else {
                world_pos
            };

            match hyper_state.state {
                HyperState::Ready => {
                    // Start a new path
                    let mut path = BezPath::new();
                    path.move_to((adjusted_pos.x as f64, adjusted_pos.y as f64));
                    hyper_state.current_path = Some(path);
                    hyper_state.points.push(adjusted_pos);
                    hyper_state.is_smooth.push(!alt_pressed); // Default to smooth points
                    hyper_state.state = HyperState::Drawing;

                    info!("Started new hyperbezier path at: {:?}", adjusted_pos);
                }
                HyperState::Drawing => {
                    // Check if clicking near start point to close the path
                    if !hyper_state.points.is_empty() {
                        let start_point = hyper_state.points[0];
                        let distance = start_point.distance(adjusted_pos);

                        if distance < hyper_state.close_path_threshold && hyper_state.points.len() > 1 {
                            info!("Closing hyperbezier path - clicked near start point");

                            // Close the path in the BezPath
                            if let Some(ref mut path) = hyper_state.current_path {
                                path.close_path();
                            }

                            // Convert to contour and add to glyph
                            if let Some(contour) = create_contour_from_points(
                                &hyper_state.points,
                                &hyper_state.is_smooth,
                            ) {
                                // Add contour to the current glyph
                                commit_contour_to_glyph(contour, &cli_args, &mut app_state, &mut app_state_changed);
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
                            let prev_point = *hyper_state.points.last().unwrap();
                            let is_smooth = !alt_pressed;
                            
                            // Calculate control points
                            let direction = adjusted_pos - prev_point;
                            let third = direction / 3.0;
                            let control1 = prev_point + third;
                            let control2 = adjusted_pos - third;
                            
                            // Now borrow the path to add curve
                            if let Some(ref mut path) = hyper_state.current_path {
                                path.curve_to(
                                    (control1.x as f64, control1.y as f64),
                                    (control2.x as f64, control2.y as f64),
                                    (adjusted_pos.x as f64, adjusted_pos.y as f64),
                                );
                            }
                            
                            // Store the point and other data after the path borrow is dropped
                            hyper_state.points.push(adjusted_pos);
                            hyper_state.is_smooth.push(is_smooth);
                            hyper_state.control_points.push(control1);
                            hyper_state.control_points.push(control2);
                            
                            info!("Added point to hyperbezier path: {:?}", adjusted_pos);
                        }
                    }
                }
                HyperState::DraggingControl => {
                    // Finish dragging control point
                    hyper_state.state = HyperState::Drawing;
                }
            }
        }
    }

    // Double click to toggle point type (smooth/corner)
    if mouse_button_input.pressed(MouseButton::Left) && mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = hyper_state.cursor_position {
            if hyper_state.state == HyperState::Drawing && !hyper_state.points.is_empty() {
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
        if hyper_state.state == HyperState::Drawing && hyper_state.points.len() >= 2 {
            info!("Finishing open hyperbezier path with right click");

            // Convert to contour and add to glyph
            if let Some(contour) = create_contour_from_points(
                &hyper_state.points,
                &hyper_state.is_smooth,
            ) {
                // Add contour to the current glyph
                commit_contour_to_glyph(contour, &cli_args, &mut app_state, &mut app_state_changed);
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
    let mut temp_control_points = Vec::with_capacity(hyper_state.points.len() * 2);
    
    // First calculate all control points without modifying the path
    let points = &hyper_state.points;
    let is_smooth = &hyper_state.is_smooth;
    let control_points = &hyper_state.control_points;
    
    for i in 1..points.len() {
        let prev = points[i-1];
        let current = points[i];
        
        // Calculate control points based on smoothness
        let control1 = if i == 1 || !is_smooth[i-1] {
            let third = (current - prev) / 3.0;
            prev + third
        } else {
            // Use reflection for smooth points
            let prev_control = if i >= 2 && control_points.len() >= (i-2)*2 + 2 {
                2.0 * prev - control_points[(i-2)*2 + 1]
            } else {
                prev + (current - prev) / 3.0
            };
            prev_control
        };
        
        let control2 = if i >= is_smooth.len() || !is_smooth[i] {
            let third = (current - prev) / 3.0;
            current - third
        } else {
            // Simple control point for now
            prev + 2.0 * (current - prev) / 3.0
        };
        
        // Store in temporary vector
        temp_control_points.push(control1);
        temp_control_points.push(control2);
    }
    
    // Now recreate the path
    let mut path = BezPath::new();
    path.move_to((points[0].x as f64, points[0].y as f64));
    
    // Add the remaining points as curves using our pre-calculated control points
    for i in 1..points.len() {
        let j = (i-1) * 2;
        if j+1 < temp_control_points.len() {
            let control1 = temp_control_points[j];
            let control2 = temp_control_points[j+1];
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
        if i > 0 && hyper_state.control_points.len() >= (i-1)*2+2 {
            let ctrl1 = hyper_state.control_points[(i-1)*2];
            let ctrl2 = hyper_state.control_points[(i-1)*2+1];
            
            // Draw control points
            gizmos.circle_2d(ctrl1, control_size, control_color);
            gizmos.circle_2d(ctrl2, control_size, control_color);
            
            // Draw handles to control points
            if i > 0 {
                gizmos.line_2d(hyper_state.points[i-1], ctrl1, control_color);
                gizmos.line_2d(*point, ctrl2, control_color);
            }
        }
    }

    // Draw preview line from last point to cursor
    if let (Some(cursor_pos), true) = (hyper_state.cursor_position, !hyper_state.points.is_empty()) {
        let last_point = *hyper_state.points.last().unwrap();

        // Apply snap to grid for preview if enabled
        let preview_cursor_pos = if SNAP_TO_GRID_ENABLED {
            Vec2::new(
                (cursor_pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                (cursor_pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
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
                (cursor_pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                (cursor_pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
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
fn create_contour_from_points(points: &[Vec2], is_smooth: &[bool]) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    // Convert points to ContourPoints
    let mut contour_points = Vec::new();
    
    // Add the first point as a move point
    contour_points.push(ContourPoint::new(
        points[0].x, 
        points[0].y, 
        PointType::Move, 
        false, // not smooth
        None,  // no name
        None,  // no identifier
        None,  // no comments
    ));
    
    // Add remaining points with control points
    for i in 1..points.len() {
        let is_smooth_point = if i < is_smooth.len() { is_smooth[i] } else { true };
        
        // For a simple MVP, we'll use line points if we don't have proper control points
        if i == 1 || !is_smooth_point {
            contour_points.push(ContourPoint::new(
                points[i].x, 
                points[i].y, 
                PointType::Line, 
                is_smooth_point,
                None, None, None
            ));
        } else {
            // In a full implementation, we'd calculate proper control points based on the hyperbezier algorithm
            // For now, we'll use simple cubic Bezier control points
            let prev = points[i-1];
            let curr = points[i];
            let control1 = prev + (curr - prev) / 3.0;
            let control2 = prev + 2.0 * (curr - prev) / 3.0;
            
            // Add control points and the on-curve point
            contour_points.push(ContourPoint::new(
                control1.x, control1.y, PointType::OffCurve, false, None, None, None
            ));
            
            contour_points.push(ContourPoint::new(
                control2.x, control2.y, PointType::OffCurve, false, None, None, None
            ));
            
            contour_points.push(ContourPoint::new(
                curr.x, curr.y, PointType::Curve, is_smooth_point, None, None, None
            ));
        }
    }

    // Create the contour
    Some(Contour::new(contour_points, None, None))
}

/// Helper function to commit a contour to the current glyph
fn commit_contour_to_glyph(
    contour: Contour,
    cli_args: &crate::cli::CliArgs,
    app_state: &mut crate::data::AppState,
    app_state_changed: &mut EventWriter<crate::draw::AppStateChanged>,
) {
    // Add contour to the current glyph
    if let Some(glyph_name) = cli_args.find_glyph(&app_state.workspace.font.ufo) {
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
                app_state_changed.send(crate::draw::AppStateChanged);
            }
        }
    }
}

/// Calculate auto control points for a smooth curve
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
