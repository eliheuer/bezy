//! Text Mode - Sort Placement Tool
//!
//! The text mode allows users to place sorts by clicking in the design space.
//! Sorts snap to grid when placed and show a metrics box preview while placing.

use crate::editing::sort::SortEvent;
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::BezySettings;
use crate::ui::panes::design_space::ViewPort;
use crate::rendering::cameras::DesignCamera;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct TextTool;

impl EditTool for TextTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "text"
    }
    
    fn name(&self) -> &'static str {
        "Text"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E017}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }
    
    fn default_order(&self) -> i32 {
        40 // After drawing tools, around position 5
    }
    
    fn description(&self) -> &'static str {
        "Place text and create sorts"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Text tool behavior is handled by dedicated systems
    }
    
    fn on_enter(&self) {
        info!("Entered Text tool - click to place sorts");
    }
    
    fn on_exit(&self) {
        info!("Exited Text tool");
    }
}

/// Resource to track if text mode is active
#[derive(Resource, Default)]
pub struct TextModeActive(pub bool);

/// Resource to track text mode state for sort placement
#[derive(Resource, Default)]
pub struct TextModeState {
    /// Current cursor position in world coordinates
    pub cursor_position: Option<Vec2>,
    /// Whether we're showing a preview
    pub showing_preview: bool,
}

/// Plugin to add text mode functionality
pub struct TextModePlugin;

impl Plugin for TextModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextModeActive>()
            .init_resource::<TextModeState>()
            .add_systems(
                Update,
                (
                    update_text_mode_active,
                    handle_text_mode_cursor,
                    handle_text_mode_clicks,
                    render_sort_preview,
                    reset_text_mode_when_inactive,
                ),
            );
    }
}

/// Plugin for the Text tool
pub struct TextToolPlugin;

impl Plugin for TextToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_text_tool)
           .add_plugins(TextModePlugin);
    }
}

fn register_text_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(TextTool));
}

/// System to track when text mode is active
pub fn update_text_mode_active(
    mut text_mode_active: ResMut<TextModeActive>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_text_mode = current_tool.get_current() == Some("text");
    
    if text_mode_active.0 != is_text_mode {
        text_mode_active.0 = is_text_mode;
        debug!("Text mode active state changed: {}", is_text_mode);
    }
}

/// System to handle cursor movement in text mode
pub fn handle_text_mode_cursor(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if !text_mode_active.0 {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let cursor_moved = !cursor_moved_events.is_empty();
    cursor_moved_events.clear(); // Consume the events

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            // Apply sort-specific grid snapping
            let settings = BezySettings::default();
            let snapped_position = settings.apply_sort_grid_snap(world_position);

            // Update state
            let position_changed = text_mode_state.cursor_position != Some(snapped_position);
            text_mode_state.cursor_position = Some(snapped_position);
            text_mode_state.showing_preview = true;
            
            // Debug logging (only when position changes or cursor moved)
            if cursor_moved || position_changed {
                debug!("Text mode cursor updated: pos=({:.1}, {:.1}), showing_preview={}", 
                       snapped_position.x, snapped_position.y, text_mode_state.showing_preview);
            }
        } else {
            debug!("Failed to convert cursor position to world coordinates");
        }
    } else {
        debug!("No cursor position available");
    }
}

/// System to handle mouse clicks in text mode for sort placement
pub fn handle_text_mode_clicks(
    text_mode_active: Res<TextModeActive>,
    text_mode_state: Res<TextModeState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut sort_events: EventWriter<SortEvent>,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    if !text_mode_active.0 {
        return;
    }

    // Don't place sorts when hovering over or clicking on UI elements
    if ui_hover_state.is_hovering_ui {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = text_mode_state.cursor_position {
            // Get the current glyph to place as a sort
            if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state) {
                // Check if glyph exists in our data
                if app_state.workspace.font.glyphs.contains_key(&glyph_name) {
                    info!("Placing sort for glyph '{}' at position ({:.1}, {:.1})", 
                          glyph_name, cursor_pos.x, cursor_pos.y);
                    
                    // Send sort creation event
                    sort_events.write(SortEvent::CreateSort {
                        glyph_name: glyph_name.clone(),
                        position: cursor_pos,
                    });
                } else {
                    warn!("Glyph '{}' not found in font data", glyph_name);
                }
            } else {
                warn!("No current glyph selected for sort placement");
            }
        }
    }
}

/// System to render sort preview while in text mode
pub fn render_sort_preview(
    text_mode_active: Res<TextModeActive>,
    text_mode_state: Res<TextModeState>,
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    viewports: Query<&ViewPort>,
) {
    if !text_mode_active.0 || !text_mode_state.showing_preview {
        return;
    }

    let Some(cursor_pos) = text_mode_state.cursor_position else {
        return;
    };

    let Ok(viewport) = viewports.get_single() else {
        return;
    };

    // Get the current glyph for preview
    if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state) {
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
            render_sort_preview_complete(
                &mut gizmos,
                viewport,
                glyph_data,
                cursor_pos,
            );
        }
    }
}

fn render_sort_preview_complete(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph_data: &crate::core::state::GlyphData,
    position: Vec2,
) {
    let preview_color = Color::srgba(0.5, 0.8, 1.0, 0.7); // Light blue preview
    
    // Render preview metrics
    render_preview_metrics_with_color(gizmos, viewport, glyph_data, position, preview_color);
    
    // Render preview glyph outline if it has contours
    if let Some(outline) = &glyph_data.outline {
        if !outline.contours.is_empty() {
            render_preview_glyph_outline(gizmos, viewport, &outline.contours, position, preview_color);
        }
    }
}

fn render_preview_metrics_with_color(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph_data: &crate::core::state::GlyphData,
    position: Vec2,
    color: Color,
) {
    let advance_width = glyph_data.advance_width as f32;
    
    // Draw metrics box
    let left = position.x;
    let right = position.x + advance_width;
    let baseline = position.y;
    
    // Use default font metrics since we don't have access to the full metrics here
    let ascender = baseline + 800.0; // Default ascender
    let descender = baseline - 200.0; // Default descender
    
    // Draw metrics lines
    draw_preview_line(gizmos, viewport, (left, baseline), (right, baseline), color); // Baseline
    draw_preview_line(gizmos, viewport, (left, ascender), (right, ascender), color); // Ascender
    draw_preview_line(gizmos, viewport, (left, descender), (right, descender), color); // Descender
    
    // Draw side bearings
    draw_preview_line(gizmos, viewport, (left, descender), (left, ascender), color); // Left bearing
    draw_preview_line(gizmos, viewport, (right, descender), (right, ascender), color); // Right bearing
}

fn render_preview_glyph_outline(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contours: &[crate::core::state::ContourData],
    offset: Vec2,
    color: Color,
) {
    for contour in contours {
        render_preview_contour_path(gizmos, viewport, contour, offset, color);
    }
}

fn render_preview_contour_path(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &crate::core::state::ContourData,
    offset: Vec2,
    color: Color,
) {
    if contour.points.is_empty() {
        return;
    }

    // Simple line rendering for preview - just connect all points
    for i in 0..contour.points.len() {
        let current = &contour.points[i];
        let next = &contour.points[(i + 1) % contour.points.len()];
        
        let start = (
            current.x as f32 + offset.x,
            current.y as f32 + offset.y,
        );
        let end = (
            next.x as f32 + offset.x,
            next.y as f32 + offset.y,
        );
        
        draw_preview_line(gizmos, viewport, start, end, color);
    }
}

fn draw_preview_line(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    let start_screen = viewport.to_screen(Vec2::new(start.0, start.1));
    let end_screen = viewport.to_screen(Vec2::new(end.0, end.1));
    gizmos.line_2d(start_screen, end_screen, color);
}

pub fn reset_text_mode_when_inactive(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
) {
    if !text_mode_active.0 {
        text_mode_state.cursor_position = None;
        text_mode_state.showing_preview = false;
    }
} 