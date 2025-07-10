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
    // respawn_active_sort_points, // REMOVED: Replaced with ECS-based system
    handle_sort_placement_input, // NEW: Uses centralized input system
    manage_sort_activation, // NEW: ECS-based sort activation management
    spawn_missing_sort_entities, // NEW: Spawn ECS entities for buffer sorts
    spawn_active_sort_points_ecs, // NEW: ECS-based point spawning for active sort
    despawn_inactive_sort_points_ecs, // NEW: ECS-based point despawning
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<crate::core::state::text_editor::TextEditorState>()
            .init_resource::<crate::core::state::text_editor::ActiveSortEntity>()
            .add_systems(
                Update,
                (
                    // Initialize the text editor when font is loaded
                    initialize_text_editor_sorts,
                    // Spawn ECS entities for buffer sorts FIRST
                    spawn_missing_sort_entities,
                    // Handle sort placement (has priority over click detection)
                    handle_sort_placement_input,
                    // NEW: Handle Unicode character input using Bevy events
                    handle_unicode_text_input,
                    // NEW: Manage sort activation in ECS
                    manage_sort_activation,
                    // NEW: Spawn points for active sort (ECS-based)
                    spawn_active_sort_points_ecs,
                    // NEW: Despawn points for inactive sorts (ECS-based)
                    despawn_inactive_sort_points_ecs,
                    // Debug system
                    debug_text_editor_state,
                    // respawn_active_sort_points, // REMOVED: Replaced with ECS-based system
                ),
            )
            .add_systems(
                Update,
                // Render the sorts AFTER all state changes
                render_text_editor_sorts,
            );
    }
} 