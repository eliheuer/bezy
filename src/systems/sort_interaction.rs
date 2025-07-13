//! Sort interaction system
//!
//! Handles mouse interactions with sorts, such as clicking to activate them.

#![allow(deprecated)]

use crate::core::state::AppState;
use crate::editing::sort::{ActiveSort, Sort, SortEvent};
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;
use crate::ui::toolbars::edit_mode_toolbar::SelectModeActive;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// System to handle mouse clicks on sorts
pub fn handle_sort_clicks(
    _mouse_button_input: Res<ButtonInput<MouseButton>>,
    _window_query: Query<&Window, With<PrimaryWindow>>,
    _camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    _sorts_query: Query<(Entity, &Sort, Has<ActiveSort>)>,
    _app_state: Res<AppState>,
    _sort_events: EventWriter<SortEvent>,
    _select_mode: Option<Res<SelectModeActive>>,
    _click_pos: Option<
        Res<crate::editing::selection::systems::ClickWorldPosition>,
    >,
    _ui_hover_state: Res<UiHoverState>,
) {
    // DISABLED: This system is for the old ECS-based sort system
    // The current system uses text editor sorts instead
    return;
}
