//! Text Mode - Sort Placement Tool
//!
//! The text mode allows users to place sorts by clicking in the design space.
//! Sorts snap to grid when placed and show a metrics box preview while placing.

use super::EditModeSystem;
use crate::core::sort::{SortEvent};
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::ui::panes::design_space::ViewPort;
use crate::ui::theme::METRICS_GUIDE_COLOR;
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
                // Create a temporary sort to get its bounds
                let temp_sort = crate::core::sort::Sort::new((**glyph).clone(), cursor_pos);
                let bounds = temp_sort.get_metrics_bounds(&app_state.workspace.info.metrics);

                // Draw preview metrics box
                render_preview_metrics_box(&mut gizmos, &viewport, &bounds);
            }
        }
    }
}

/// Render a preview metrics box
fn render_preview_metrics_box(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    bounds: &crate::core::sort::SortBounds,
) {
    use crate::ui::panes::design_space::DPoint;

    let preview_color = Color::srgba(
        METRICS_GUIDE_COLOR.to_srgba().red,
        METRICS_GUIDE_COLOR.to_srgba().green,
        METRICS_GUIDE_COLOR.to_srgba().blue,
        0.5, // Semi-transparent for preview
    );

    // Convert bounds to screen coordinates
    let tl_screen = viewport.to_screen(DPoint::from((bounds.min.x, bounds.max.y)));
    let tr_screen = viewport.to_screen(DPoint::from((bounds.max.x, bounds.max.y)));
    let bl_screen = viewport.to_screen(DPoint::from((bounds.min.x, bounds.min.y)));
    let br_screen = viewport.to_screen(DPoint::from((bounds.max.x, bounds.min.y)));

    // Draw the preview rectangle outline
    gizmos.line_2d(tl_screen, tr_screen, preview_color); // top
    gizmos.line_2d(tr_screen, br_screen, preview_color); // right
    gizmos.line_2d(br_screen, bl_screen, preview_color); // bottom
    gizmos.line_2d(bl_screen, tl_screen, preview_color); // left
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
