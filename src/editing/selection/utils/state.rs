//! State management utilities for selection

use crate::editing::selection::components::SelectionState;
use crate::editing::selection::events::AppStateChanged;
use bevy::prelude::*;

/// System to clear selection when app state changes
pub fn clear_selection_on_app_change(
    _commands: Commands,
    mut app_state_events: EventReader<AppStateChanged>,
    mut selection_state: ResMut<SelectionState>,
) {
    for _event in app_state_events.read() {
        // Clear selection when app state changes
        selection_state.selected.clear();
        info!("Cleared selection due to app state change");
    }
}

/// System to update hover state (currently disabled)
#[allow(dead_code)]
pub fn update_hover_state(/* hover state parameters would go here when enabled */
) {
    // Hover functionality is disabled per user request
}
