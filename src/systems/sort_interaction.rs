//! Sort interaction system
//!
//! Handles mouse interactions with sorts, such as clicking to activate them.

use crate::editing::sort::{Sort, SortEvent, InactiveSort};
use crate::core::state::AppState;
use crate::rendering::cameras::DesignCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to handle mouse clicks on sorts
pub fn handle_sort_clicks(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    sorts_query: Query<(Entity, &Sort), With<InactiveSort>>,
    app_state: Res<AppState>,
    mut sort_events: EventWriter<SortEvent>,
    current_mode: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode>,
) {
    // Only handle clicks when in select mode
    if current_mode.0 != crate::ui::toolbars::edit_mode_toolbar::EditMode::Select {
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
    for (entity, sort) in sorts_query.iter() {
        if sort.contains_point(world_position, &app_state.workspace.info.metrics) {
            // Activate this sort
            sort_events.send(SortEvent::ActivateSort {
                sort_entity: entity,
            });
            
            info!("Activated sort '{}' by clicking", sort.glyph.name);
            return; // Only activate one sort per click
        }
    }

    // If we didn't click on any sort, deactivate the current active sort
    sort_events.send(SortEvent::DeactivateSort);
} 