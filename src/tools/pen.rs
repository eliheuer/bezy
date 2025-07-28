//! # Pen Tool
//!
//! The pen tool allows users to draw vector paths by clicking points in sequence.
//! Click to place points, click near the start point to close the path, or right-click
//! to finish an open path. Hold Shift for axis-aligned drawing, press Escape to cancel.
//!
//! The tool converts placed points into UFO contours that are saved to the font file.

#![allow(clippy::too_many_arguments)]

use super::{EditTool, ToolInfo};
use crate::core::io::input::{helpers, InputEvent, InputMode, InputState};
use crate::core::io::pointer::PointerInfo;
use crate::core::state::{AppState, ContourData, PointData, PointTypeData};
use crate::editing::selection::events::AppStateChanged;
use crate::geometry::design_space::DPoint;
use crate::systems::ui_interaction::UiHoverState;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use kurbo::{BezPath, Point};

pub struct PenTool;

impl EditTool for PenTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "pen",
            display_name: "Pen",
            icon: "\u{E011}",
            tooltip: "Draw paths and contours",
            shortcut: Some(KeyCode::KeyP),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(true));
        commands.insert_resource(InputMode::Pen);
        info!("Entered Pen tool");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(false));
        commands.insert_resource(InputMode::Normal);
        info!("Exited Pen tool");
    }

    fn update(&self, _commands: &mut Commands) {
        // Pen tool behavior handled by dedicated systems
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
                | InputEvent::MouseRelease { .. }
                | InputEvent::KeyPress { .. }
        )
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        debug!("Pen tool handling input: {:?}", event);
        // Input handling is done in dedicated systems for better ECS integration
    }
}

/// Current state of the pen tool's path drawing
#[derive(Resource, Default)]
pub struct PenToolState {
    /// Points that have been placed in the current path
    pub current_path: Vec<DPoint>,
    /// Whether the path should be closed (clicking near start point)
    pub should_close_path: bool,
    /// Whether we are currently placing a path
    pub is_drawing: bool,
}

// ================================================================
// PLUGIN SETUP
// ================================================================

/// Bevy plugin that sets up the pen tool
pub struct PenToolPlugin;

impl Plugin for PenToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PenToolState>()
            .init_resource::<PenModeActive>()
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

// ================================================================
// SYSTEMS
// ================================================================

/// System to handle mouse events for the pen tool
pub fn handle_pen_mouse_events(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    pointer_info: Res<PointerInfo>,
    ui_hover_state: Res<UiHoverState>,
) {
    if !pen_mode_active.0 || ui_hover_state.is_hovering_ui {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let click_position = pointer_info.design;

        // Check if we should close the path
        if pen_state.current_path.len() > 2 {
            if let Some(first_point) = pen_state.current_path.first() {
                let distance =
                    click_position.to_raw().distance(first_point.to_raw());
                if distance < CLOSE_PATH_THRESHOLD {
                    pen_state.should_close_path = true;
                    finalize_pen_path(
                        &mut pen_state,
                        &mut app_state,
                        &mut fontir_app_state,
                        &mut app_state_changed,
                    );
                    return;
                }
            }
        }

        // Add point to current path
        pen_state.current_path.push(click_position);
        pen_state.is_drawing = true;

        info!(
            "Pen tool: Added point at ({:.1}, {:.1}), total points: {}",
            click_position.x,
            click_position.y,
            pen_state.current_path.len()
        );
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        // Finish open path
        if pen_state.current_path.len() > 1 {
            finalize_pen_path(
                &mut pen_state,
                &mut app_state,
                &mut fontir_app_state,
                &mut app_state_changed,
            );
        }
    }
}

/// System to handle keyboard events for the pen tool
pub fn handle_pen_keyboard_events(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !pen_mode_active.0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Cancel current path
        pen_state.current_path.clear();
        pen_state.is_drawing = false;
        pen_state.should_close_path = false;
        info!("Pen tool: Cancelled current path");
    }
}

/// System to render the pen tool preview
pub fn render_pen_preview(
    mut gizmos: Gizmos,
    pen_state: Res<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    pointer_info: Res<PointerInfo>,
) {
    if !pen_mode_active.0 {
        return;
    }

    let preview_color = Color::srgb(0.0, 0.8, 1.0);
    let point_color = Color::srgb(1.0, 0.3, 0.0);

    // Draw current path points
    for (i, &point) in pen_state.current_path.iter().enumerate() {
        let pos = Vec2::new(point.x, point.y);
        gizmos.circle_2d(pos, POINT_PREVIEW_SIZE, point_color);

        // Draw line to next point
        if i > 0 {
            let prev_point = pen_state.current_path[i - 1];
            let prev_pos = Vec2::new(prev_point.x, prev_point.y);
            gizmos.line_2d(prev_pos, pos, preview_color);
        }
    }

    // Draw preview line to cursor if we have at least one point
    if let Some(&last_point) = pen_state.current_path.last() {
        let last_pos = Vec2::new(last_point.x, last_point.y);
        let cursor_pos =
            Vec2::new(pointer_info.design.x, pointer_info.design.y);
        gizmos.line_2d(last_pos, cursor_pos, preview_color.with_alpha(0.5));
    }

    // Draw cursor indicator
    let cursor_pos = Vec2::new(pointer_info.design.x, pointer_info.design.y);
    gizmos.circle_2d(
        cursor_pos,
        CURSOR_INDICATOR_SIZE,
        preview_color.with_alpha(0.7),
    );
}

/// System to reset pen mode when it becomes inactive
pub fn reset_pen_mode_when_inactive(
    mut pen_state: ResMut<PenToolState>,
    pen_mode_active: Res<PenModeActive>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    if pen_mode_active.is_changed() && !pen_mode_active.0 {
        pen_state.current_path.clear();
        pen_state.is_drawing = false;
        pen_state.should_close_path = false;
        app_state_changed.write(AppStateChanged);
        debug!("Reset pen state due to mode change");
    }
}

/// Helper function to finalize the current pen path
fn finalize_pen_path(
    pen_state: &mut ResMut<PenToolState>,
    _app_state: &mut Option<ResMut<AppState>>,
    fontir_app_state: &mut Option<ResMut<crate::core::state::FontIRAppState>>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    if pen_state.current_path.len() < 2 {
        return;
    }

    // Try FontIR first, then fallback to AppState
    if let Some(_fontir_state) = fontir_app_state.as_mut() {
        finalize_fontir_path(pen_state);
    } else if let Some(_app_state) = _app_state.as_mut() {
        finalize_appstate_path(pen_state);
    } else {
        warn!(
            "Pen tool: No AppState or FontIR available for path finalization"
        );
    }

    // Reset state
    pen_state.current_path.clear();
    pen_state.is_drawing = false;
    pen_state.should_close_path = false;

    app_state_changed.write(AppStateChanged);
}

/// Helper function to finalize path using FontIR BezPath operations
fn finalize_fontir_path(pen_state: &mut ResMut<PenToolState>) {
    // Create a BezPath from the current path
    let mut bez_path = BezPath::new();

    if let Some(&first_point) = pen_state.current_path.first() {
        bez_path
            .move_to(Point::new(first_point.x as f64, first_point.y as f64));

        for &point in pen_state.current_path.iter().skip(1) {
            bez_path.line_to(Point::new(point.x as f64, point.y as f64));
        }

        if pen_state.should_close_path {
            bez_path.close_path();
        }
    }

    // For now, just log the BezPath creation
    // TODO: Add the BezPath to the FontIR glyph data
    let path_info =
        format!("BezPath with {} elements", bez_path.elements().len());
    info!("Pen tool (FontIR): Created {} for current glyph", path_info);
    info!(
        "Pen tool (FontIR): Path elements: {:?}",
        bez_path.elements()
    );
}

/// Helper function to finalize path using traditional AppState operations
fn finalize_appstate_path(pen_state: &mut ResMut<PenToolState>) {
    // Convert path to ContourData
    let mut points = Vec::new();

    for (i, &point) in pen_state.current_path.iter().enumerate() {
        let point_type = if i == 0 {
            PointTypeData::Move
        } else {
            PointTypeData::Line
        };

        points.push(PointData {
            x: point.x as f64,
            y: point.y as f64,
            point_type,
        });
    }

    // Close path if requested
    if pen_state.should_close_path {
        points.push(PointData {
            x: pen_state.current_path[0].x as f64,
            y: pen_state.current_path[0].y as f64,
            point_type: PointTypeData::Line,
        });
    }

    let contour = ContourData { points };

    // Add contour to current glyph - this needs to be done through a proper system
    // For now just log that we would add it
    info!("Pen tool (AppState): Would add {} point contour to current glyph (contour has {} points)", 
          pen_state.current_path.len(), contour.points.len());
}
