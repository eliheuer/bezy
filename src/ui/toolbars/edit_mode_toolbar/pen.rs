//! # Pen Tool
//!
//! The pen tool allows users to draw vector paths by clicking points in sequence.
//! Click to place points, click near the start point to close the path, or right-click
//! to finish an open path. Hold Shift for axis-aligned drawing, press Escape to cancel.
//!
//! The tool converts placed points into UFO contours that are saved to the font file.

use super::EditModeSystem;
use crate::core::input::{
    helpers, InputEvent, InputMode, InputState, ModifierState,
};
use crate::core::pointer::PointerInfo;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::core::state::AppState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected,
};
use crate::editing::selection::systems::AppStateChanged;
use crate::editing::selection::{
    DragPointState, DragSelectionState, SelectionState,
};
use crate::editing::sort::ActiveSortState;
use crate::geometry::design_space::DPoint;
use crate::systems::sort_manager::SortPointEntity;
use crate::systems::ui_interaction::UiHoverState;
use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint};

pub struct PenTool;

impl EditTool for PenTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "pen"
    }

    fn name(&self) -> &'static str {
        "Pen"
    }

    fn icon(&self) -> &'static str {
        "\u{E011}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('p')
    }

    fn default_order(&self) -> i32 {
        20 // After select, primary drawing tool
    }

    fn description(&self) -> &'static str {
        "Draw paths and contours"
    }

    fn update(&self, commands: &mut Commands) {
        // Ensure pen mode is active
        commands.insert_resource(PenModeActive(true));
        commands.insert_resource(SelectModeActive(false));
    }

    fn on_enter(&self) {
        info!("Entered Pen tool");
    }

    fn on_exit(&self) {
        info!("Exited Pen tool");
    }
}

// ================================================================
// CONSTANTS
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
            .init_resource::<PenModeActive>()
            .add_systems(Startup, register_pen_tool)
            .add_systems(
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

fn register_pen_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PenTool));
}

// ================================================================
// RESOURCES AND STATE
// ================================================================

/// Resource to track if pen mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct PenModeActive(pub bool);

/// Input consumer for pen tool
#[derive(Resource)]
pub struct PenInputConsumer;

impl crate::systems::input_consumer::InputConsumer for PenInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Only handle input if pen mode is active
        if !helpers::is_input_mode(input_state, InputMode::Pen) {
            return false;
        }

        // Handle mouse events
        matches!(
            event,
            InputEvent::MouseClick { .. }
                | InputEvent::MouseDrag { .. }
                | InputEvent::MouseRelease { .. }
        )
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Pen: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen click handling
                }
            }
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                delta: _,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Pen: Processing mouse drag from {:?} to {:?} with modifiers {:?}",
                          start_position, current_position, modifiers);
                    // TODO: Implement pen drag handling
                }
            }
            InputEvent::MouseRelease {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    info!("Pen: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen release handling
                }
            }
            _ => {}
        }
    }
}

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
}

impl Default for PenToolState {
    fn default() -> Self {
        Self {
            active: true,
            state: PenState::Ready,
            current_path: None,
            points: Vec::new(),
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
        commands.insert_resource(SelectModeActive(false));
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
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut pen_state: ResMut<PenToolState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    // Early return if still in pen mode
    if current_tool.get_current() == Some("pen") {
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
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    if !is_path_drawable(pen_state) {
        return;
    }

    if let Some(_contour) =
        create_contour_from_points(&pen_state.points, Vec2::ZERO)
    {
        app_state_changed.write(AppStateChanged);
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
pub fn handle_pen_mouse_events(
    mut commands: Commands,
    pointer_info: Res<PointerInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    mut app_state: ResMut<crate::core::state::AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
    ui_hover_state: Res<UiHoverState>,
) {
    if !is_pen_mode_active(&pen_mode) || ui_hover_state.is_hovering_ui {
        return;
    }

    // Get the active sort from text_editor_state
    let active_sort_info = text_editor_state
        .as_ref()
        .and_then(|state| state.get_active_sort());

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let cursor_pos = pointer_info.design.to_raw();
        handle_left_click(
            &mut commands,
            &keyboard,
            &mut pen_state,
            &glyph_navigation,
            &mut app_state,
            &mut app_state_changed,
            active_sort_info,
            text_editor_state.as_deref(),
            cursor_pos,
        );
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        handle_right_click(
            &mut commands,
            &mut pen_state,
            &glyph_navigation,
            &mut app_state,
            &mut app_state_changed,
            active_sort_info,
            text_editor_state.as_deref(),
        );
    }
}

/// Check if pen mode is currently active
fn is_pen_mode_active(pen_mode: &Option<Res<PenModeActive>>) -> bool {
    pen_mode.as_ref().map_or(false, |mode| mode.0)
}

/// Handle left mouse button clicks
fn handle_left_click(
    _commands: &mut Commands,
    keyboard: &Res<ButtonInput<KeyCode>>,
    pen_state: &mut ResMut<PenToolState>,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
    cursor_pos: Vec2,
) {
    let final_position =
        calculate_final_position(cursor_pos, keyboard, pen_state);

    if pen_state.state == PenState::Ready {
        start_new_path(pen_state, final_position);
    } else if should_close_path(pen_state, final_position) {
        close_current_path(
            pen_state,
            glyph_navigation,
            app_state,
            app_state_changed,
            active_sort_info,
            text_editor_state,
        );
    } else {
        add_point_to_path(pen_state, final_position);
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
        apply_snap_to_grid(cursor_pos)
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
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
) {
    if !pen_state.points.is_empty() {
        let active_sort_offset = if let (Some((sort_index, _)), Some(state)) =
            (active_sort_info, text_editor_state)
        {
            state
                .get_sort_visual_position(sort_index)
                .unwrap_or(Vec2::ZERO)
        } else {
            Vec2::ZERO
        };

        if let Some(contour) =
            create_contour_from_points(&pen_state.points, active_sort_offset)
        {
            add_contour_to_glyph(
                contour,
                glyph_navigation,
                app_state,
                app_state_changed,
                true,
                active_sort_info,
            );
        }
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
    _commands: &mut Commands,
    pen_state: &mut ResMut<PenToolState>,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
) {
    if pen_state.state == PenState::Drawing && pen_state.points.len() >= 2 {
        info!("Finishing open path with right click");

        let active_sort_offset = if let (Some((sort_index, _)), Some(state)) =
            (active_sort_info, text_editor_state)
        {
            state
                .get_sort_visual_position(sort_index)
                .unwrap_or(Vec2::ZERO)
        } else {
            Vec2::ZERO
        };

        if let Some(contour) =
            create_contour_from_points(&pen_state.points, active_sort_offset)
        {
            add_contour_to_glyph(
                contour,
                glyph_navigation,
                app_state,
                app_state_changed,
                false,
                active_sort_info,
            );
        }

        reset_pen_state(pen_state);
    }
}

/// Add a contour to the current glyph using the current thread-safe architecture
fn add_contour_to_glyph(
    contour: Contour,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    is_closed: bool,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
) {
    let glyph_name = if let Some((_sort_index, sort_entry)) = active_sort_info {
        info!(
            "PEN TOOL: Using active sort glyph: {}",
            sort_entry.kind.glyph_name()
        );
        sort_entry.kind.glyph_name().to_string()
    } else {
        let Some(glyph_name) = glyph_navigation.current_glyph.clone() else {
            warn!("PEN TOOL: No glyph found in navigation and no active sort");
            return;
        };
        info!("PEN TOOL: Using glyph navigation glyph: {}", glyph_name);
        glyph_name
    };

    info!(
        "PEN TOOL: Adding contour with {} points to glyph {}",
        contour.points.len(),
        glyph_name
    );

    // For now, we'll create a simplified implementation that works with the current architecture
    // TODO: This needs to be properly implemented when the full glyph editing system is ready
    // The current architecture uses thread-safe data structures and doesn't have direct norad access

    // Convert the norad contour to our thread-safe ContourData
    let contour_data =
        crate::core::state::font_data::ContourData::from_norad_contour(
            &contour,
        );

    // Check if the glyph exists in our thread-safe data
    if let Some(glyph_data) =
        app_state.workspace.font.glyphs.get_mut(&glyph_name)
    {
        // Get or create the outline data
        let outline_data = glyph_data.outline.get_or_insert_with(|| {
            crate::core::state::font_data::OutlineData {
                contours: Vec::new(),
            }
        });

        // Add the new contour
        outline_data.contours.push(contour_data);

        let path_type = if is_closed { "closed" } else { "open" };
        let source = if active_sort_info.is_some() {
            "active sort"
        } else {
            "glyph navigation"
        };
        info!("PEN TOOL: Successfully added {} contour to glyph {} (from {}). Total contours now: {}", 
               path_type, glyph_name, source, outline_data.contours.len());

        // Notify that the app state has changed
        app_state_changed.write(AppStateChanged);
    } else {
        warn!(
            "PEN TOOL: Could not find glyph '{}' in app state to add contour",
            glyph_name
        );
    }
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
    pointer_info: Res<PointerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    pen_state: Res<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
) {
    if !is_pen_mode_active(&pen_mode) {
        return;
    }

    draw_placed_points_and_lines(&mut gizmos, &pen_state);
    let cursor_pos = pointer_info.design.to_raw();
    draw_preview_elements(&mut gizmos, &pen_state, cursor_pos, &keyboard);
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

/// Draw preview elements like the next line segment and axis-locked guides
fn draw_preview_elements(
    gizmos: &mut Gizmos,
    pen_state: &PenToolState,
    cursor_pos: Vec2,
    keyboard: &Res<ButtonInput<KeyCode>>,
) {
    // Draw cursor indicator
    gizmos.circle_2d(
        cursor_pos,
        CURSOR_INDICATOR_SIZE,
        Color::srgb(0.0, 1.0, 0.0),
    );

    if let Some(&last_point) = pen_state.points.last() {
        // Calculate the final position for the preview, same logic as for placing points
        let final_pos =
            calculate_final_position(cursor_pos, keyboard, pen_state);

        // Draw line from last point to cursor's final position
        gizmos.line_2d(last_point, final_pos, Color::srgb(0.0, 1.0, 0.0));

        // Draw a circle at the final position
        gizmos.circle_2d(
            final_pos,
            POINT_PREVIEW_SIZE,
            Color::srgb(0.0, 1.0, 0.0),
        );

        // If close to the start point, draw a circle to indicate path closing
        draw_close_indicator_if_needed(
            gizmos, pen_state, cursor_pos, last_point,
        );
    }
}

/// If the cursor is close to the start point, draw a special indicator
fn draw_close_indicator_if_needed(
    gizmos: &mut Gizmos,
    pen_state: &PenToolState,
    cursor_pos: Vec2,
    _last_point: Vec2,
) {
    if let Some(&first_point) = pen_state.points.first() {
        if cursor_pos.distance(first_point) < CLOSE_PATH_THRESHOLD {
            gizmos.circle_2d(
                first_point,
                CLOSE_PATH_THRESHOLD,
                Color::srgba(1.0, 0.0, 0.0, 0.5),
            );
        }
    }
}

// ================================================================
// UTILITY FUNCTIONS
// ================================================================

/// Lock a position to horizontal or vertical axis relative to another point
/// (used when shift is held to constrain movement)
fn axis_lock_position(pos: Vec2, relative_to: Vec2) -> Vec2 {
    let dxy = pos - relative_to;
    if dxy.x.abs() > dxy.y.abs() {
        Vec2::new(pos.x, relative_to.y)
    } else {
        Vec2::new(relative_to.x, pos.y)
    }
}

/// Create a UFO contour from a list of points
fn create_contour_from_points(
    points: &[Vec2],
    active_sort_offset: Vec2,
) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    info!(
        "PEN TOOL: Creating contour with active_sort_offset: ({:.1}, {:.1})",
        active_sort_offset.x, active_sort_offset.y
    );

    let mut contour_points = Vec::new();

    for point in points {
        // Convert from world coordinates to glyph-local coordinates
        let glyph_local_point = *point - active_sort_offset;

        info!(
            "  - Converting point: world({:.1}, {:.1}) -> local({:.1}, {:.1})",
            point.x, point.y, glyph_local_point.x, glyph_local_point.y
        );

        contour_points.push(ContourPoint::new(
            glyph_local_point.x as f64,
            glyph_local_point.y as f64,
            crate::core::state::font_data::PointTypeData::Line
                .to_norad_point_type(), // Convert our internal type to norad for I/O
            false, // not smooth
            None,  // no name
            None,  // no identifier
        ));
    }

    Some(Contour::new(contour_points, None))
}

/// Snaps a position to the grid, based on the current zoom level and settings.
fn apply_snap_to_grid(pos: Vec2) -> Vec2 {
    // For now, a simple 10-unit grid. This should be driven by settings.
    let grid_size = 10.0;
    (pos / grid_size).round() * grid_size
}
