//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::text_editor_sorts::{
    initialize_text_editor_sorts,
    handle_text_editor_sort_clicks,
    render_text_editor_sorts,
    handle_text_editor_keyboard_input,
    debug_text_editor_state,
    sync_text_editor_active_sort,
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    // Initialize the text editor when font is loaded
                    initialize_text_editor_sorts,
                    // Handle mouse clicks on sorts
                    handle_text_editor_sort_clicks,
                    // Handle keyboard navigation
                    handle_text_editor_keyboard_input,
                    // Debug system
                    debug_text_editor_state,
                    // Sync active sort with selection system
                    sync_text_editor_active_sort,
                    // Render the sorts
                    render_text_editor_sorts,
                ).chain(), // Run in order to ensure proper initialization
            );
    }
} 