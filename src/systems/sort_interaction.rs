//! Sort interaction system
//!
//! Handles mouse interactions with sorts, such as clicking to activate them.

use crate::editing::sort::{Sort, SortEvent, ActiveSort};
use crate::core::state::AppState;
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;
use crate::ui::toolbars::edit_mode_toolbar::CurrentTool;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to handle mouse clicks on sorts
pub fn handle_sort_clicks(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    sorts_query: Query<(Entity, &Sort, Has<ActiveSort>)>,
    app_state: Res<AppState>,
    mut sort_events: EventWriter<SortEvent>,
    current_tool: Res<CurrentTool>,
    click_pos: Option<Res<crate::editing::selection::systems::ClickWorldPosition>>,
    ui_hover_state: Res<UiHoverState>,
) {
    // If we're in pen mode, do nothing. Pen tool handles clicks.
    if current_tool.get_current() == Some("pen") {
        return;
    }

    if ui_hover_state.is_hovering_ui {
        return;
    }

    // If the click was already handled by another system (e.g., on a point or crosshair), do nothing.
    if click_pos.is_some() {
        return;
    }

    // Only handle clicks when in select mode
    if current_tool.get_current() != Some("select") {
        return;
    }

    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Check if the click is within any sort's bounds
    for (entity, sort, is_active) in sorts_query.iter() {
        if sort.contains_point(world_position, &app_state.workspace.info.metrics) {
            if is_active {
                // Sort is already active - keep it active (allow editing)
                info!("Clicked on already active sort '{}' - keeping active", sort.glyph_name);
                return; // Don't deactivate, allow editing to continue
            } else {
                // Sort is inactive - activate it
                sort_events.send(SortEvent::ActivateSort {
                    sort_entity: entity,
                });
                info!("Activated sort '{}' by clicking", sort.glyph_name);
            }
            return; // Only handle one sort per click
        }
    }

    // If we didn't click on any sort AND no other system claimed the click, deactivate the current active sort
    // This prevents deactivation when clicking on crosshairs or other UI elements
    if click_pos.is_none() {
        sort_events.send(SortEvent::DeactivateSort);
    }
} 