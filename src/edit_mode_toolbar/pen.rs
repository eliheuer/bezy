use super::EditModeSystem;
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint};
use crate::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};

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

/// Resource to track the state of the pen tool
#[derive(Resource)]
pub struct PenToolState {
    /// Whether the pen tool is active
    pub active: bool,
    /// The state of the pen tool
    pub state: PenState,
    /// The current path being drawn
    pub current_path: Option<BezPath>,
    /// Points already placed in the current path
    pub points: Vec<Vec2>,
    /// The current cursor position
    pub cursor_position: Option<Vec2>,
    /// How close to the start point to close the path
    pub close_path_threshold: f32,
}

impl Default for PenToolState {
    fn default() -> Self {
        Self {
            active: true,
            state: PenState::default(),
            current_path: None,
            points: Vec::new(),
            cursor_position: None,
            close_path_threshold: 15.0,
        }
    }
}

/// The state of the pen tool
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum PenState {
    /// Ready to place a point
    #[default]
    Ready,
    /// Drawing a path with points
    Drawing,
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
    mut pen_state: ResMut<PenToolState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
) {
    if current_mode.0 != crate::edit_mode_toolbar::EditMode::Pen {
        // Commit any open path before deactivating
        if pen_state.state == PenState::Drawing
            && !pen_state.points.is_empty()
            && pen_state.points.len() >= 2
        {
            if let Some(_contour) =
                create_contour_from_points(&pen_state.points)
            {
                // Signal that we've made a change to the glyph
                app_state_changed.send(crate::draw::AppStateChanged);
                info!("Committing path on mode change");
            }
        }

        // Clear state and mark inactive
        *pen_state = PenToolState::default();
        pen_state.active = false;
        commands.insert_resource(PenModeActive(false));
    }
}

/// System to handle mouse events for the pen tool
#[allow(clippy::too_many_arguments)]
pub fn handle_pen_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    camera_q: Query<
        (&Camera, &GlobalTransform),
        With<crate::cameras::DesignCamera>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    cli_args: Res<crate::cli::CliArgs>,
    mut app_state: ResMut<crate::data::AppState>,
    mut app_state_changed: EventWriter<crate::draw::AppStateChanged>,
    ui_hover_state: Res<crate::ui_interaction::UiHoverState>,
) {
    // Only handle events when in pen mode
    if let Some(pen_mode) = pen_mode {
        if !pen_mode.0 {
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
        warn!("No active camera found for pen tool");
        return;
    };

    // Handle cursor movement
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to world coordinates
            match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                Ok(world_position) => {
                    pen_state.cursor_position = Some(world_position);
                }
                Err(_) => {}
            }
        }
    }

    // Handle mouse down
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(world_pos) = pen_state.cursor_position {
            // Get shift state for alignment
            let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight);

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
            let adjusted_pos = if shift_pressed && !pen_state.points.is_empty()
            {
                let last_point = pen_state.points.last().unwrap();
                axis_lock_position(world_pos, *last_point)
            } else {
                world_pos
            };

            match pen_state.state {
                PenState::Ready => {
                    // Start a new path
                    let mut path = BezPath::new();
                    path.move_to((
                        adjusted_pos.x as f64,
                        adjusted_pos.y as f64,
                    ));
                    pen_state.current_path = Some(path);
                    pen_state.points.push(adjusted_pos);
                    pen_state.state = PenState::Drawing;

                    info!("Started new path at: {:?}", adjusted_pos);
                }
                PenState::Drawing => {
                    // Check if clicking on start point to close the path
                    if !pen_state.points.is_empty() {
                        let start_point = pen_state.points[0];
                        let distance = start_point.distance(adjusted_pos);

                        if distance < pen_state.close_path_threshold
                            && pen_state.points.len() > 1
                        {
                            info!("Closing path - clicked near start point");

                            // Close the path in the BezPath
                            if let Some(ref mut path) = pen_state.current_path {
                                path.close_path();
                            }

                            // Convert to contour and add to glyph
                            if let Some(contour) =
                                create_contour_from_points(&pen_state.points)
                            {
                                // Add contour to the current glyph
                                if let Some(glyph_name) = cli_args
                                    .find_glyph(&app_state.workspace.font.ufo)
                                {
                                    let glyph_name = glyph_name.clone();

                                    // Get mutable access to the font
                                    let font_obj =
                                        app_state.workspace.font_mut();

                                    // Get the current glyph
                                    if let Some(default_layer) =
                                        font_obj.ufo.get_default_layer_mut()
                                    {
                                        if let Some(glyph) = default_layer
                                            .get_glyph_mut(&glyph_name)
                                        {
                                            // Get or create the outline
                                            let outline = glyph
                                                .outline
                                                .get_or_insert_with(|| {
                                                    norad::glyph::Outline {
                                                        contours: Vec::new(),
                                                        components: Vec::new(),
                                                    }
                                                });

                                            // Add the new contour
                                            outline.contours.push(contour);
                                            info!(
                                                "Added new contour to glyph {}",
                                                glyph_name
                                            );

                                            // Notify that the app state has changed
                                            app_state_changed.send(
                                                crate::draw::AppStateChanged,
                                            );
                                        }
                                    }
                                }
                            }

                            // Reset for next path
                            pen_state.current_path = None;
                            pen_state.points.clear();
                            pen_state.state = PenState::Ready;
                        } else {
                            // Add line to existing path
                            if let Some(ref mut path) = pen_state.current_path {
                                path.line_to((
                                    adjusted_pos.x as f64,
                                    adjusted_pos.y as f64,
                                ));
                                pen_state.points.push(adjusted_pos);
                                info!(
                                    "Added point to path: {:?}",
                                    adjusted_pos
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle right click to finish path without closing
    if mouse_button_input.just_pressed(MouseButton::Right) {
        if pen_state.state == PenState::Drawing && pen_state.points.len() >= 2 {
            info!("Finishing open path with right click");

            // Convert to contour and add to glyph
            if let Some(contour) = create_contour_from_points(&pen_state.points)
            {
                // Add contour to the current glyph
                if let Some(glyph_name) =
                    cli_args.find_glyph(&app_state.workspace.font.ufo)
                {
                    let glyph_name = glyph_name.clone();

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

                            // Add the new contour
                            outline.contours.push(contour);
                            info!(
                                "Added new open contour to glyph {}",
                                glyph_name
                            );

                            // Notify that the app state has changed
                            app_state_changed
                                .send(crate::draw::AppStateChanged);
                        }
                    }
                }
            }

            // Reset for next path
            pen_state.current_path = None;
            pen_state.points.clear();
            pen_state.state = PenState::Ready;
        }
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
    let line_color = Color::srgba(1.0, 1.0, 1.0, 0.9); // White
    let preview_color = Color::srgba(0.8, 0.8, 0.8, 0.5); // Translucent white
    let close_highlight_color = Color::srgb(0.2, 1.0, 0.3); // Green for close indicator

    // Visualization parameters
    let point_size = 5.0;

    // Draw points and lines between them
    for (i, point) in pen_state.points.iter().enumerate() {
        // Draw point
        gizmos.circle_2d(
            *point,
            point_size,
            if i == 0 {
                // Highlight start point
                Color::srgb(0.0, 1.0, 0.5)
            } else {
                point_color
            },
        );

        // Draw line to next point
        if i < pen_state.points.len() - 1 {
            gizmos.line_2d(*point, pen_state.points[i + 1], line_color);
        }
    }

    // Draw preview line from last point to cursor
    if let (Some(cursor_pos), true) =
        (pen_state.cursor_position, !pen_state.points.is_empty())
    {
        let last_point = *pen_state.points.last().unwrap();
        
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
        if pen_state.points.len() > 1 {
            let start_point = pen_state.points[0];
            let distance = start_point.distance(preview_cursor_pos);

            if distance < pen_state.close_path_threshold {
                // Draw highlight to indicate path can be closed
                gizmos.circle_2d(
                    start_point,
                    pen_state.close_path_threshold,
                    Color::srgba(0.2, 1.0, 0.3, 0.3),
                );
                gizmos.line_2d(last_point, start_point, close_highlight_color);
            }
        }
    }

    // Draw cursor position
    if let Some(cursor_pos) = pen_state.cursor_position {
        // Apply snap to grid for cursor display if enabled
        let display_cursor_pos = if SNAP_TO_GRID_ENABLED {
            Vec2::new(
                (cursor_pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                (cursor_pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
            )
        } else {
            cursor_pos
        };
        
        gizmos.circle_2d(display_cursor_pos, 3.0, Color::srgba(1.0, 1.0, 1.0, 0.7));
    }
}

/// System to handle keyboard events for the pen tool
pub fn handle_pen_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
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
        pen_state.state = PenState::Ready;
        info!("Cancelled current path with Escape key");
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
fn create_contour_from_points(points: &[Vec2]) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    // Convert points to ContourPoints
    let contour_points: Vec<ContourPoint> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let point_type = if i == 0 {
                norad::PointType::Move
            } else {
                norad::PointType::Line
            };

            ContourPoint::new(
                p.x, p.y, point_type, false, // not smooth
                None,  // no name
                None,  // no identifier
                None,  // no comments
            )
        })
        .collect();

    // Create the contour
    Some(Contour::new(contour_points, None, None))
}
