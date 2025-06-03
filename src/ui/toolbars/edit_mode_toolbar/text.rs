//! Text Mode - Sort Placement Tool
//!
//! The text mode allows users to place sorts by clicking in the design space.
//! Sorts snap to grid when placed and show a metrics box preview while placing.

use super::EditModeSystem;
use crate::editing::sort::{SortEvent};
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::ui::panes::design_space::ViewPort;
use crate::rendering::cameras::DesignCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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

pub struct TextMode;

impl EditModeSystem for TextMode {
    fn update(&self, _commands: &mut Commands) {
        // Text mode update is handled by dedicated systems
    }

    fn on_enter(&self) {
        info!("Entered text mode - click to place sorts");
    }

    fn on_exit(&self) {
        info!("Exited text mode");
    }
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

/// System to track when text mode is active
pub fn update_text_mode_active(
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
    mut text_mode_active: ResMut<TextModeActive>,
) {
    text_mode_active.0 = current_mode.0 == crate::ui::toolbars::edit_mode_toolbar::EditMode::Text;
}

/// System to update cursor position in text mode
pub fn handle_text_mode_cursor(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if !text_mode_active.0 {
        return;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    // Update cursor position
    for _cursor_moved in cursor_moved_events.read() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Apply grid snapping
                let snapped_position = if SNAP_TO_GRID_ENABLED {
                    Vec2::new(
                        (world_position.x / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                        (world_position.y / SNAP_TO_GRID_VALUE).round() * SNAP_TO_GRID_VALUE,
                    )
                } else {
                    world_position
                };

                text_mode_state.cursor_position = Some(snapped_position);
                text_mode_state.showing_preview = true;
            }
        }
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
) {
    if !text_mode_active.0 {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = text_mode_state.cursor_position {
            // Get the current glyph to place as a sort
            if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
                if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
                    if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                        // Create a sort event
                        sort_events.send(SortEvent::CreateSort {
                            glyph: (**glyph).clone(),
                            position: cursor_pos,
                        });

                        info!("Placed sort '{}' at position ({:.1}, {:.1})", 
                              glyph_name, cursor_pos.x, cursor_pos.y);
                    } else {
                        warn!("Could not find glyph '{}' in default layer", glyph_name);
                    }
                } else {
                    warn!("No default layer found in font");
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

    // Get viewport for coordinate transformations
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => return,
    };

    // Get the current glyph to preview
    if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
        if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
            if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                // Use the shared metrics rendering function with semi-transparent color
                render_preview_metrics(&mut gizmos, &viewport, glyph, &app_state.workspace.info.metrics, cursor_pos);
            }
        }
    }
}

/// Render a preview metrics box with semi-transparent appearance
fn render_preview_metrics(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &norad::Glyph,
    metrics: &crate::core::state::FontMetrics,
    position: Vec2,
) {
    use crate::ui::panes::design_space::DPoint;
    use crate::ui::theme::METRICS_GUIDE_COLOR;

    // Create a semi-transparent version of the metrics color for preview
    let preview_color = Color::srgba(
        METRICS_GUIDE_COLOR.to_srgba().red,
        METRICS_GUIDE_COLOR.to_srgba().green,
        METRICS_GUIDE_COLOR.to_srgba().blue,
        0.5, // Semi-transparent for preview
    );

    let upm = metrics.units_per_em;
    let x_height = metrics.x_height.unwrap_or_else(|| (upm * 0.5).round());
    let cap_height = metrics.cap_height.unwrap_or_else(|| (upm * 0.7).round());
    let ascender = metrics.ascender.unwrap_or_else(|| (upm * 0.8).round());
    let descender = metrics.descender.unwrap_or_else(|| -(upm * 0.2).round());
    let width = glyph
        .advance
        .as_ref()
        .map(|a| a.width as f64)
        .unwrap_or_else(|| (upm * 0.5).round());

    // All coordinates are offset by the position
    let offset_x = position.x;
    let offset_y = position.y;

    // Draw the standard metrics bounding box (descender to ascender)
    draw_preview_rect(
        gizmos,
        viewport,
        (offset_x, offset_y + descender as f32),
        (offset_x + width as f32, offset_y + ascender as f32),
        preview_color,
    );

    // Draw the full UPM bounding box (from 0 to UPM height)
    draw_preview_rect(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y + upm as f32),
        preview_color,
    );

    // Draw baseline
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y),
        preview_color,
    );

    // Draw x-height line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + x_height as f32),
        (offset_x + width as f32, offset_y + x_height as f32),
        preview_color,
    );

    // Draw cap-height line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + cap_height as f32),
        (offset_x + width as f32, offset_y + cap_height as f32),
        preview_color,
    );

    // Draw ascender line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + ascender as f32),
        (offset_x + width as f32, offset_y + ascender as f32),
        preview_color,
    );

    // Draw descender line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + descender as f32),
        (offset_x + width as f32, offset_y + descender as f32),
        preview_color,
    );

    // Draw UPM top line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + upm as f32),
        (offset_x + width as f32, offset_y + upm as f32),
        preview_color,
    );
}

/// Draw a line in design space for preview
fn draw_preview_line(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    start: (f32, f32),
    end: (f32, f32),
    color: Color,
) {
    use crate::ui::panes::design_space::DPoint;
    let start_screen = viewport.to_screen(DPoint::from(start));
    let end_screen = viewport.to_screen(DPoint::from(end));
    gizmos.line_2d(start_screen, end_screen, color);
}

/// Draw a rectangle outline in design space for preview
fn draw_preview_rect(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    top_left: (f32, f32),
    bottom_right: (f32, f32),
    color: Color,
) {
    use crate::ui::panes::design_space::DPoint;
    let tl_screen = viewport.to_screen(DPoint::from(top_left));
    let br_screen = viewport.to_screen(DPoint::from(bottom_right));

    // Draw the rectangle outline (four lines)
    gizmos.line_2d(
        Vec2::new(tl_screen.x, tl_screen.y),
        Vec2::new(br_screen.x, tl_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(br_screen.x, tl_screen.y),
        Vec2::new(br_screen.x, br_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(br_screen.x, br_screen.y),
        Vec2::new(tl_screen.x, br_screen.y),
        color,
    );
    gizmos.line_2d(
        Vec2::new(tl_screen.x, br_screen.y),
        Vec2::new(tl_screen.x, tl_screen.y),
        color,
    );
}

/// System to reset text mode state when not active
pub fn reset_text_mode_when_inactive(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
) {
    if !text_mode_active.0 {
        text_mode_state.cursor_position = None;
        text_mode_state.showing_preview = false;
    }
}
