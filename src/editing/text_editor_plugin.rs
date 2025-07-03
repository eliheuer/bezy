//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::text_editor_sorts::{
    initialize_text_editor_sorts,
    handle_text_editor_sort_clicks, // ENABLED: Fixed coordinate system issues
    render_text_editor_sorts,
    handle_text_editor_keyboard_input,
    debug_text_editor_state,
    sync_text_editor_active_sort,
    handle_sort_placement_input, // NEW: Uses centralized input system
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<crate::core::state::text_editor::TextEditorState>()
            .add_systems(
                Update,
                (
                    // Initialize the text editor when font is loaded
                    initialize_text_editor_sorts,
                    // Handle mouse clicks on sorts - ENABLED: Fixed coordinate system issues
                    handle_text_editor_sort_clicks,
                    // NEW: Handle sort placement using centralized input system
                    handle_sort_placement_input,
                    // Handle keyboard navigation
                    handle_text_editor_keyboard_input,
                    // Debug system
                    debug_text_editor_state,
                    // Sync active sort with selection system
                    sync_text_editor_active_sort,
                    // Render the sorts
                    render_text_editor_sorts,
                ), // Run systems in parallel
            );
    }
} 