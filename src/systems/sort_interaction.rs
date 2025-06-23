//! Sort interaction system
//!
//! Handles mouse interactions with sorts, such as clicking to activate them.

use crate::editing::sort::{Sort, SortEvent, ActiveSort};
use crate::core::state::AppState;
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;
use crate::ui::toolbars::edit_mode_toolbar::SelectModeActive;
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
    select_mode: Option<Res<SelectModeActive>>,
    click_pos: Option<Res<crate::editing::selection::systems::ClickWorldPosition>>,
    ui_hover_state: Res<UiHoverState>,
) {
    // Only handle clicks when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    } else {
        return;
    }

    if ui_hover_state.is_hovering_ui {
        return;
    }

    // If the click was already handled by another system (e.g., on a point or crosshair), do nothing.
    if click_pos.is_some() {
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
    // Collect all sorts that contain the click point
    let mut clicked_sorts: Vec<_> = sorts_query.iter()
        .filter(|(_, sort, _)| sort.contains_point(world_position, &app_state.workspace.info.metrics))
        .collect();
    
    // If multiple sorts overlap, pick the one with the smallest bounding box (most specific)
    if !clicked_sorts.is_empty() {
        // Sort by bounding box area (smallest first)
        clicked_sorts.sort_by(|a, b| {
            let area_a = a.1.advance_width * 1000.0; // Use advance width as proxy for area
            let area_b = b.1.advance_width * 1000.0;
            area_a.partial_cmp(&area_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let (entity, sort, is_active) = clicked_sorts[0];
        
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

    // If we didn't click on any sort AND no other system claimed the click, deactivate the current active sort
    // This prevents deactivation when clicking on crosshairs or other UI elements
    if click_pos.is_none() {
        sort_events.send(SortEvent::DeactivateSort);
    }
} 