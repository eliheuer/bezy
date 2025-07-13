//! Knife Tool - Path cutting and slicing tool
//!
//! This tool allows users to cut paths by drawing a line across them.
//! The tool shows a preview of the cutting line and intersection points.

#![allow(unused_variables)]

use crate::core::state::AppState;
#[allow(unused_imports)]
use crate::core::state::GlyphNavigation;
use crate::editing::selection::systems::AppStateChanged;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "knife"
    }

    fn name(&self) -> &'static str {
        "Knife"
    }

    fn icon(&self) -> &'static str {
        "\u{E013}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }

    fn default_order(&self) -> i32 {
        110 // Advanced tool, later in toolbar
    }

    fn description(&self) -> &'static str {
        "Cut and slice paths"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(KnifeModeActive(true));
    }

    fn on_enter(&self) {
        info!("Entered Knife tool");
    }

    fn on_exit(&self) {
        info!("Exited Knife tool");
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
#[derive(Resource, Default)]
pub struct KnifeToolState {
    /// The current gesture state
    pub gesture: KnifeGestureState,
    /// Whether shift key is pressed (for axis-aligned cuts)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
}

impl KnifeToolState {
    pub fn new() -> Self {
        Self {
            gesture: KnifeGestureState::Ready,
            shift_locked: false,
            intersections: Vec::new(),
        }
    }

    /// Get the cutting line with axis locking if shift is pressed
    pub fn get_cutting_line(&self) -> Option<(Vec2, Vec2)> {
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
                Some((start, actual_end))
            }
            KnifeGestureState::Ready => None,
        }
    }
}

/// Plugin for the knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeModeActive>()
            .init_resource::<KnifeToolState>()
            .add_systems(Startup, register_knife_tool)
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

fn register_knife_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(KnifeTool));
}

/// Handle mouse events for the knife tool
pub fn handle_knife_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert cursor position to world coordinates
    if let Ok(world_position) =
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    {
        // Update shift lock state
        knife_state.shift_locked = keyboard.pressed(KeyCode::ShiftLeft)
            || keyboard.pressed(KeyCode::ShiftRight);

        // Handle mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            knife_state.gesture = KnifeGestureState::Cutting {
                start: world_position,
                current: world_position,
            };
            knife_state.intersections.clear();
        }

        // Handle mouse movement during cutting
        if let KnifeGestureState::Cutting { start, .. } = knife_state.gesture {
            knife_state.gesture = KnifeGestureState::Cutting {
                start,
                current: world_position,
            };

            // Update intersections for preview
            update_intersections(&mut knife_state, &app_state);
        }

        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) {
            if let Some((start, end)) = knife_state.get_cutting_line() {
                // Perform the cut
                perform_cut(start, end, &mut app_state, &mut app_state_changed);
            }

            // Reset state
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.intersections.clear();
        }
    }
}

/// Handle keyboard events for the knife tool
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
    } else {
        return;
    }

    // Handle Escape key to cancel current cut
    if keyboard.just_pressed(KeyCode::Escape) {
        knife_state.gesture = KnifeGestureState::Ready;
        knife_state.intersections.clear();
        info!("Cancelled knife cut");
    }
}

/// Reset knife mode when tool is inactive
pub fn reset_knife_mode_when_inactive(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut knife_state: ResMut<KnifeToolState>,
) {
    if current_tool.get_current() != Some("knife") {
        // Reset state and mark inactive
        *knife_state = KnifeToolState::new();
        commands.insert_resource(KnifeModeActive(false));
    }
}

/// Render the knife tool preview
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
    } else {
        return;
    }

    // Draw the cutting line
    if let Some((start, end)) = knife_state.get_cutting_line() {
        let line_color = Color::srgba(1.0, 0.3, 0.3, 0.9);
        draw_dashed_line(&mut gizmos, start, end, 8.0, 4.0, line_color);

        // Draw start point
        let start_color = Color::srgba(0.3, 1.0, 0.5, 1.0);
        gizmos.circle_2d(start, 4.0, start_color);
    }

    // Draw intersection points
    let intersection_color = Color::srgba(1.0, 1.0, 0.0, 1.0);
    for &intersection in &knife_state.intersections {
        gizmos.circle_2d(intersection, 3.0, intersection_color);
    }
}

/// Update intersection points for preview
fn update_intersections(
    knife_state: &mut KnifeToolState,
    app_state: &AppState,
) {
    knife_state.intersections.clear();

    // For now, just return empty intersections
    // TODO: Implement proper path intersection calculation
    // This would require complex geometric calculations to find where
    // the cutting line intersects with the glyph contours
}

/// Perform the actual cut operation
fn perform_cut(
    _start: Vec2,
    _end: Vec2,
    _app_state: &mut AppState,
    _app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    // For now, just log the cut operation
    info!("Knife cut performed (not yet implemented)");

    // TODO: Implement actual path cutting
    // This would involve:
    // 1. Finding all intersection points with glyph contours
    // 2. Splitting contours at intersection points
    // 3. Creating new contour segments
    // 4. Updating the glyph data
}

/// Draw a dashed line
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

    let mut current_pos = 0.0;

    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;

        gizmos.line_2d(dash_start, dash_end, color);

        current_pos += segment_length;
    }
}
