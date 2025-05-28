//! # Pen Tool
//!
//! The pen tool allows users to draw vector paths by clicking points in sequence.
//! Click to place points, click near the start point to close the path, or right-click
//! to finish an open path. Hold Shift for axis-aligned drawing, press Escape to cancel.
//!
//! The tool converts placed points into UFO contours that are saved to the font file.

use super::EditModeSystem;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint};

// ================================================================
// CONSTANTS TODO: MOVE TO SETTINGS
// ================================================================

/// Distance threshold for closing a path by clicking near the start point
const CLOSE_PATH_THRESHOLD: f32 = 16.0;
/// Size of drawn points in the preview
const POINT_PREVIEW_SIZE: f32 = 4.0;
const CURSOR_INDICATOR_SIZE: f32 = 4.0;

// ================================================================
// PLUGIN SETUP
// ================================================================

/// Bevy plugin that sets up the pen tool
///
/// This plugin initializes the pen tool's state resources and registers
/// all the systems needed for pen functionality:
/// - Mouse input handling for placing points
/// - Keyboard shortcuts (Escape to cancel)
/// - Visual preview rendering of the current path
/// - Cleanup when switching away from pen mode
pub struct PenModePlugin;

impl Plugin for PenModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PenToolState>()
            .init_resource::<PenModeActive>();
        app.add_systems(
            Update,
            (
                handle_pen_mouse_events,
                handle_pen_keyboard_events,
                render_pen_preview,
                reset_pen_mode_when_inactive,
            ),
        );
    }
}

// ================================================================
// RESOURCES AND STATE
// ================================================================

/// Resource to track if pen mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct PenModeActive(pub bool);

/// The main state manager for the pen tool
///
/// The pen tool works like this:
/// 1. Start in Ready state - waiting for first click
/// 2. First click starts a new path and moves to Drawing state
/// 3. Subsequent clicks add points to the current path
/// 4. Click near start point to close the path
/// 5. Right-click to finish an open path
/// 6. Escape cancels the current path
#[derive(Resource)]
pub struct PenToolState {
    /// Whether the pen tool is currently active
    pub active: bool,
    /// The current drawing state (Ready or Drawing)
    pub state: PenState,
    /// The path being constructed (using kurbo for geometry)
    pub current_path: Option<BezPath>,
    /// Points that have been placed in the current path
    pub points: Vec<Vec2>,
    /// Current mouse cursor position in world coordinates
    pub cursor_position: Option<Vec2>,
}

impl Default for PenToolState {
    fn default() -> Self {
        Self {
            active: true,
            state: PenState::Ready,
            current_path: None,
            points: Vec::new(),
            cursor_position: None,
        }
    }
}

/// The two states the pen tool can be in
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum PenState {
    /// Ready to start a new path (no points placed yet)
    #[default]
    Ready,
    /// Currently drawing a path (at least one point placed)
    Drawing,
}

// ================================================================
// EDIT MODE IMPLEMENTATION
// ================================================================

/// Pen mode for drawing vector paths in glyphs
pub struct PenMode;

impl EditModeSystem for PenMode {
    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entering Pen Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Pen Mode");
    }
}

// ================================================================
// MODE MANAGEMENT
// ================================================================

/// Handles cleanup when switching away from pen mode
///
/// If the user was in the middle of drawing a path, this system
/// will automatically commit it before switching modes.
pub fn reset_pen_mode_when_inactive(
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
    mut commands: Commands,
    mut pen_state: ResMut<PenToolState>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
) {
    // Early return if still in pen mode
    if current_mode.0 == crate::ui::toolbars::edit_mode_toolbar::EditMode::Pen {
        return;
    }

    // Save any work in progress before switching modes
    try_commit_current_path(&pen_state, &mut app_state_changed);

    // Clean up and deactivate pen mode
    deactivate_pen_mode(&mut pen_state, &mut commands);
}

/// Attempts to commit the current path if it has enough points to be drawable
fn try_commit_current_path(
    pen_state: &PenToolState,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
) {
    if !is_path_drawable(pen_state) {
        return;
    }

    if let Some(_contour) = create_contour_from_points(&pen_state.points) {
        app_state_changed.send(crate::rendering::draw::AppStateChanged);
        info!("Auto-committing path when switching modes");
    }
}

/// Checks if the current path has enough points to create a drawable contour
fn is_path_drawable(pen_state: &PenToolState) -> bool {
    pen_state.state == PenState::Drawing && pen_state.points.len() >= 2
}

/// Resets pen state and marks pen mode as inactive
fn deactivate_pen_mode(
    pen_state: &mut ResMut<PenToolState>,
    commands: &mut Commands,
) {
    **pen_state = PenToolState::default();
    pen_state.active = false;
    commands.insert_resource(PenModeActive(false));
}

// ================================================================
// MOUSE INPUT HANDLING
// ================================================================

/// Main system for handling mouse interactions with the pen tool
#[allow(clippy::too_many_arguments)]
pub fn handle_pen_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    camera_q: Query<
        (&Camera, &GlobalTransform),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    cli_args: Res<crate::core::cli::CliArgs>,
    mut app_state: ResMut<crate::core::data::AppState>,
    mut app_state_changed: EventWriter<crate::rendering::draw::AppStateChanged>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // Early returns for invalid states
    if !is_pen_mode_active(&pen_mode) || ui_hover_state.is_hovering_ui {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some((camera, camera_transform)) = find_active_camera(&camera_q) else {
        warn!("No active camera found for pen tool");
        return;
    };

    // Update cursor position from mouse movement
    update_cursor_position(
        &mut cursor_moved_events,
        &window,
        camera,
        camera_transform,
        &mut pen_state,
    );

    // Handle mouse clicks
    if mouse_button_input.just_pressed(MouseButton::Left) {
        handle_left_click(
            &keyboard,
            &mut pen_state,
            &cli_args,
            &mut app_state,
            &mut app_state_changed,
        );
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        handle_right_click(
            &mut pen_state,
            &cli_args,
            &mut app_state,
            &mut app_state_changed,
        );
    }
}

/// Check if pen mode is currently active
fn is_pen_mode_active(pen_mode: &Option<Res<PenModeActive>>) -> bool {
    pen_mode.as_ref().map_or(false, |mode| mode.0)
}

/// Find the active camera for coordinate conversion
fn find_active_camera<'a>(
    camera_q: &'a Query<
        (&Camera, &GlobalTransform),
        With<crate::rendering::cameras::DesignCamera>,
    >,
) -> Option<(&'a Camera, &'a GlobalTransform)> {
    camera_q.iter().find(|(camera, _)| camera.is_active)
}

/// Update the cursor position from mouse movement events
fn update_cursor_position(
    cursor_moved_events: &mut EventReader<CursorMoved>,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    pen_state: &mut ResMut<PenToolState>,
) {
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_pos)
            {
                pen_state.cursor_position = Some(world_position);
            }
        }
    }
}

/// Handle left mouse button clicks
fn handle_left_click(
    keyboard: &Res<ButtonInput<KeyCode>>,
    pen_state: &mut ResMut<PenToolState>,
    cli_args: &Res<crate::core::cli::CliArgs>,
    app_state: &mut ResMut<crate::core::data::AppState>,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
) {
    let Some(cursor_pos) = pen_state.cursor_position else {
        return;
    };

    // Apply snapping and axis locking
    let final_pos = calculate_final_position(cursor_pos, keyboard, pen_state);

    match pen_state.state {
        PenState::Ready => {
            start_new_path(pen_state, final_pos);
        }
        PenState::Drawing => {
            if should_close_path(pen_state, final_pos) {
                close_current_path(
                    pen_state,
                    cli_args,
                    app_state,
                    app_state_changed,
                );
            } else {
                add_point_to_path(pen_state, final_pos);
            }
        }
    }
}

/// Calculate the final position after applying snap-to-grid and axis locking
fn calculate_final_position(
    cursor_pos: Vec2,
    keyboard: &Res<ButtonInput<KeyCode>>,
    pen_state: &PenToolState,
) -> Vec2 {
    // Apply snap to grid first
    let snapped_pos = if SNAP_TO_GRID_ENABLED {
        Vec2::new(
            (cursor_pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
            (cursor_pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
        )
    } else {
        cursor_pos
    };

    // Apply axis locking if shift is held and we have points
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if shift_pressed && !pen_state.points.is_empty() {
        let last_point = pen_state.points.last().unwrap();
        axis_lock_position(snapped_pos, *last_point)
    } else {
        snapped_pos
    }
}

/// Start a new path with the first point
fn start_new_path(pen_state: &mut ResMut<PenToolState>, position: Vec2) {
    let mut path = BezPath::new();
    path.move_to((position.x as f64, position.y as f64));

    pen_state.current_path = Some(path);
    pen_state.points.push(position);
    pen_state.state = PenState::Drawing;

    info!("Started new path at: {:?}", position);
}

/// Check if we should close the path (clicked near start point)
fn should_close_path(pen_state: &PenToolState, position: Vec2) -> bool {
    if pen_state.points.len() <= 1 {
        return false;
    }

    let start_point = pen_state.points[0];
    let distance = start_point.distance(position);
    distance < CLOSE_PATH_THRESHOLD
}

/// Close the current path and add it to the glyph
fn close_current_path(
    pen_state: &mut ResMut<PenToolState>,
    cli_args: &Res<crate::core::cli::CliArgs>,
    app_state: &mut ResMut<crate::core::data::AppState>,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
) {
    info!("Closing path - clicked near start point");

    // Close the path in kurbo
    if let Some(ref mut path) = pen_state.current_path {
        path.close_path();
    }

    // Add the closed path to the current glyph
    if let Some(contour) = create_contour_from_points(&pen_state.points) {
        add_contour_to_glyph(
            contour,
            cli_args,
            app_state,
            app_state_changed,
            true,
        );
    }

    // Reset for next path
    reset_pen_state(pen_state);
}

/// Add a point to the current path
fn add_point_to_path(pen_state: &mut ResMut<PenToolState>, position: Vec2) {
    if let Some(ref mut path) = pen_state.current_path {
        path.line_to((position.x as f64, position.y as f64));
        pen_state.points.push(position);
        info!("Added point to path: {:?}", position);
    }
}

/// Handle right mouse button clicks (finish open path)
fn handle_right_click(
    pen_state: &mut ResMut<PenToolState>,
    cli_args: &Res<crate::core::cli::CliArgs>,
    app_state: &mut ResMut<crate::core::data::AppState>,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
) {
    if pen_state.state == PenState::Drawing && pen_state.points.len() >= 2 {
        info!("Finishing open path with right click");

        if let Some(contour) = create_contour_from_points(&pen_state.points) {
            add_contour_to_glyph(
                contour,
                cli_args,
                app_state,
                app_state_changed,
                false,
            );
        }

        reset_pen_state(pen_state);
    }
}

/// Add a contour to the current glyph
fn add_contour_to_glyph(
    contour: Contour,
    cli_args: &Res<crate::core::cli::CliArgs>,
    app_state: &mut ResMut<crate::core::data::AppState>,
    app_state_changed: &mut EventWriter<
        crate::rendering::draw::AppStateChanged,
    >,
    is_closed: bool,
) {
    let Some(glyph_name) = cli_args.find_glyph(&app_state.workspace.font.ufo)
    else {
        return;
    };
    let glyph_name = glyph_name.clone();

    // Get mutable access to the font and glyph
    let font_obj = app_state.workspace.font_mut();
    let Some(default_layer) = font_obj.ufo.get_default_layer_mut() else {
        return;
    };
    let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) else {
        return;
    };

    // Get or create the outline
    let outline = glyph.outline.get_or_insert_with(|| norad::glyph::Outline {
        contours: Vec::new(),
        components: Vec::new(),
    });

    // Add the new contour
    outline.contours.push(contour);

    let path_type = if is_closed { "closed" } else { "open" };
    info!("Added new {} contour to glyph {}", path_type, glyph_name);

    // Notify that the app state has changed
    app_state_changed.send(crate::rendering::draw::AppStateChanged);
}

/// Reset pen state to ready for next path
fn reset_pen_state(pen_state: &mut ResMut<PenToolState>) {
    pen_state.current_path = None;
    pen_state.points.clear();
    pen_state.state = PenState::Ready;
}

// ================================================================
// KEYBOARD INPUT HANDLING
// ================================================================

/// Handle keyboard shortcuts for the pen tool
pub fn handle_pen_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
) {
    if !is_pen_mode_active(&pen_mode) {
        return;
    }

    // Escape key cancels the current path
    if keyboard.just_pressed(KeyCode::Escape) {
        reset_pen_state(&mut pen_state);
        info!("Cancelled current path with Escape key");
    }
}

// ================================================================
// VISUAL PREVIEW RENDERING
// ================================================================

/// Render visual preview of the pen tool's current state
///
/// This shows:
/// - Placed points (yellow circles, green for start point)
/// - Lines connecting placed points (white)
/// - Preview line from last point to cursor (translucent white)
/// - Close indicator when near start point (green highlight)
/// - Current cursor position (small white circle)
pub fn render_pen_preview(
    mut gizmos: Gizmos,
    pen_state: Res<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
) {
    if !is_pen_mode_active(&pen_mode) {
        return;
    }

    // Draw the placed points and connecting lines
    draw_placed_points_and_lines(&mut gizmos, &pen_state);

    // Draw preview elements (cursor, preview line, close indicator)
    draw_preview_elements(&mut gizmos, &pen_state);
}

/// Draw all the points that have been placed and lines between them
fn draw_placed_points_and_lines(gizmos: &mut Gizmos, pen_state: &PenToolState) {
    let point_color = Color::srgb(1.0, 1.0, 0.0); // Yellow
    let start_point_color = Color::srgb(0.0, 1.0, 0.5); // Green
    let line_color = Color::srgba(1.0, 1.0, 1.0, 0.9); // White

    for (i, point) in pen_state.points.iter().enumerate() {
        // Draw point (start point gets special color)
        let color = if i == 0 {
            start_point_color
        } else {
            point_color
        };
        gizmos.circle_2d(*point, POINT_PREVIEW_SIZE, color);

        // Draw line to next point
        if i < pen_state.points.len() - 1 {
            gizmos.line_2d(*point, pen_state.points[i + 1], line_color);
        }
    }
}

/// Draw preview elements: cursor, preview line, and close indicator
fn draw_preview_elements(gizmos: &mut Gizmos, pen_state: &PenToolState) {
    let Some(cursor_pos) = pen_state.cursor_position else {
        return;
    };

    // Apply snap to grid for preview
    let snapped_cursor = apply_snap_to_grid(cursor_pos);

    // Draw cursor indicator
    gizmos.circle_2d(
        snapped_cursor,
        CURSOR_INDICATOR_SIZE,
        Color::srgba(1.0, 1.0, 1.0, 0.7),
    );

    if pen_state.points.is_empty() {
        return;
    }

    let last_point = *pen_state.points.last().unwrap();

    // Draw preview line from last point to cursor
    gizmos.line_2d(
        last_point,
        snapped_cursor,
        Color::srgba(1.0, 1.0, 1.0, 0.5),
    );

    // Draw close indicator if near start point
    if pen_state.points.len() > 1 {
        draw_close_indicator_if_needed(
            gizmos,
            pen_state,
            snapped_cursor,
            last_point,
        );
    }
}

/// Draw the close path indicator when cursor is near start point
fn draw_close_indicator_if_needed(
    gizmos: &mut Gizmos,
    pen_state: &PenToolState,
    cursor_pos: Vec2,
    last_point: Vec2,
) {
    let start_point = pen_state.points[0];
    let distance = start_point.distance(cursor_pos);

    if distance < CLOSE_PATH_THRESHOLD {
        // Draw highlight circle around start point
        gizmos.circle_2d(
            start_point,
            CLOSE_PATH_THRESHOLD,
            Color::srgba(0.2, 1.0, 0.3, 0.3),
        );

        // Draw line from last point to start point in green
        gizmos.line_2d(last_point, start_point, Color::srgb(0.2, 1.0, 0.3));
    }
}

// ================================================================
// UTILITY FUNCTIONS
// ================================================================

/// Apply snap-to-grid if enabled
fn apply_snap_to_grid(pos: Vec2) -> Vec2 {
    if SNAP_TO_GRID_ENABLED {
        Vec2::new(
            (pos.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
            (pos.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
        )
    } else {
        pos
    }
}

/// Lock a position to horizontal or vertical axis relative to another point
/// (used when shift is held to constrain movement)
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

/// Convert a list of points into a UFO Contour for font storage
///
/// This creates the actual geometry that gets saved in the font file.
/// The first point becomes a "move" operation, subsequent points are "line" operations.
fn create_contour_from_points(points: &[Vec2]) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    let contour_points: Vec<ContourPoint> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let point_type = if i == 0 {
                norad::PointType::Move // First point starts the path
            } else {
                norad::PointType::Line // Subsequent points are line segments
            };

            ContourPoint::new(
                p.x, p.y, point_type, false, // not smooth
                None,  // no name
                None,  // no identifier
                None,  // no comments
            )
        })
        .collect();

    Some(Contour::new(contour_points, None, None))
}
