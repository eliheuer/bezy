//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::text_editor_sorts::{
    initialize_text_editor_sorts,
    // handle_text_editor_sort_clicks, // REMOVED: legacy system
    render_text_editor_sorts,
    handle_text_editor_keyboard_input,
    // handle_text_input_with_cosmic, // DISABLED: Legacy system causing double input
    handle_arabic_text_input, // NEW: Arabic and Unicode text input
    handle_unicode_text_input, // NEW: Unicode character input using Bevy events
    debug_text_editor_state,
    respawn_active_sort_points, // NEW: Spawns all points for the active sort
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
                    // Handle sort placement FIRST (has priority over click detection)
                    handle_sort_placement_input,
                    // NEW: Handle Unicode character input using Bevy events
                    handle_unicode_text_input,
                    // Debug system
                    debug_text_editor_state,
                    // Sync active sort and spawn points for active sort
                    respawn_active_sort_points,
                ),
            )
            .add_systems(
                Update,
                // Render the sorts AFTER all state changes
                render_text_editor_sorts,
            );
    }
} 