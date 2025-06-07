//! Text Mode - Sort Placement Tool
//!
//! The text mode allows users to place sorts by clicking in the design space.
//! Sorts snap to grid when placed and show a metrics box preview while placing.

use super::EditModeSystem;
use crate::editing::sort::{SortEvent};
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::apply_sort_grid_snap;
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
    mut text_mode_active: ResMut<TextModeActive>,
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
) {
    let is_text_mode = current_mode.0 == crate::ui::toolbars::edit_mode_toolbar::EditMode::Text;
    
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
            let snapped_position = apply_sort_grid_snap(world_position);

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
            if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
                if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
                    if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                        sort_events.send(SortEvent::CreateSort {
                            glyph_name: glyph.name.clone(),
                            position: Vec2::new(
                                cursor_pos.x,
                                cursor_pos.y,
                            ),
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
    // Debug logging
    if text_mode_active.0 {
        debug!("Text mode active: showing_preview={}, cursor_pos={:?}", 
               text_mode_state.showing_preview, text_mode_state.cursor_position);
    }
    
    if !text_mode_active.0 || !text_mode_state.showing_preview {
        return;
    }

    let Some(cursor_pos) = text_mode_state.cursor_position else {
        debug!("No cursor position available for preview");
        return;
    };

    // Get viewport for coordinate transformations - use default if none found (like main rendering systems)
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => {
            debug!("No viewport found, using default viewport for preview");
            ViewPort::default()
        }
    };

    // Get the current glyph to preview
    if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state.workspace.font.ufo) {
        debug!("Found glyph for preview: {}", glyph_name);
        if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
            if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                debug!("Rendering sort preview for glyph: {}", glyph_name);
                // Render preview with selection marquee color (yellow)
                render_sort_preview_complete(&mut gizmos, &viewport, glyph, &app_state.workspace.info.metrics, cursor_pos);
            } else {
                debug!("Glyph '{}' not found in default layer", glyph_name);
            }
        } else {
            debug!("No default layer found");
        }
    } else {
        debug!("No current glyph found for preview");
    }
}

/// Render a complete sort preview with metrics and glyph outline in selection color
fn render_sort_preview_complete(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &norad::Glyph,
    metrics: &crate::core::state::FontMetrics,
    position: Vec2,
) {
    use crate::ui::theme::SELECTED_POINT_COLOR;

    // Use the selection marquee color (yellow) with some transparency for preview
    let preview_color = Color::srgba(
        SELECTED_POINT_COLOR.to_srgba().red,
        SELECTED_POINT_COLOR.to_srgba().green,
        SELECTED_POINT_COLOR.to_srgba().blue,
        0.7, // Semi-transparent for preview
    );

    // First render the metrics box
    render_preview_metrics_with_color(gizmos, viewport, glyph, metrics, position, preview_color);
    
    // Then render the glyph outline if it exists
    if let Some(outline) = &glyph.outline {
        render_preview_glyph_outline(gizmos, viewport, outline, position, preview_color);
    }
}

/// Render preview metrics box with specified color
fn render_preview_metrics_with_color(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    glyph: &norad::Glyph,
    metrics: &crate::core::state::FontMetrics,
    position: Vec2,
    color: Color,
) {
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
        color,
    );

    // Draw the full UPM bounding box (from 0 to UPM height)
    draw_preview_rect(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y + upm as f32),
        color,
    );

    // Draw baseline
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y),
        (offset_x + width as f32, offset_y),
        color,
    );

    // Draw x-height line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + x_height as f32),
        (offset_x + width as f32, offset_y + x_height as f32),
        color,
    );

    // Draw cap-height line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + cap_height as f32),
        (offset_x + width as f32, offset_y + cap_height as f32),
        color,
    );

    // Draw ascender line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + ascender as f32),
        (offset_x + width as f32, offset_y + ascender as f32),
        color,
    );

    // Draw descender line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + descender as f32),
        (offset_x + width as f32, offset_y + descender as f32),
        color,
    );

    // Draw UPM top line
    draw_preview_line(
        gizmos,
        viewport,
        (offset_x, offset_y + upm as f32),
        (offset_x + width as f32, offset_y + upm as f32),
        color,
    );
}

/// Render preview glyph outline with specified color
fn render_preview_glyph_outline(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    outline: &norad::glyph::Outline,
    offset: Vec2,
    color: Color,
) {
    // Render each contour in the outline (outline only, no handles for preview)
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }

        // Draw only the path for preview
        render_preview_contour_path(gizmos, viewport, contour, offset, color);
    }
}

/// Render a contour path for preview with specified color
fn render_preview_contour_path(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    contour: &norad::Contour,
    offset: Vec2,
    color: Color,
) {
    use crate::ui::panes::design_space::DPoint;
    
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Simple line-based rendering for preview (faster than full curve rendering)
    for i in 0..points.len() {
        let current_point = &points[i];
        let next_index = (i + 1) % points.len();
        let next_point = &points[next_index];

        let start_pos = viewport.to_screen(DPoint::from((
            current_point.x as f32 + offset.x,
            current_point.y as f32 + offset.y,
        )));
        let end_pos = viewport.to_screen(DPoint::from((
            next_point.x as f32 + offset.x,
            next_point.y as f32 + offset.y,
        )));

        gizmos.line_2d(start_pos, end_pos, color);
    }
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
