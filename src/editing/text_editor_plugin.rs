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
    spawn_active_sort_points_optimized, // NEW: Optimized instant point spawning
    despawn_inactive_sort_points_optimized, // NEW: Optimized instant point despawning
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<crate::core::state::text_editor::TextEditorState>()
            .init_resource::<crate::core::state::text_editor::ActiveSortEntity>()
            
            // Initialize text editor state
            .add_systems(Startup, initialize_text_editor_sorts)
            
            // Sort activation management (runs first)
            .add_systems(Update, (
                manage_sort_activation,
                spawn_missing_sort_entities,
            ).chain())
            
            // Instant point spawning/despawning (runs immediately after activation)
            .add_systems(Update, (
                spawn_active_sort_points_optimized,
                despawn_inactive_sort_points_optimized,
            ).chain().after(manage_sort_activation))
            
            // Rendering and input handling
            .add_systems(Update, (
                render_text_editor_sorts,
                // handle_text_editor_keyboard_input, // DISABLED: Causes double input
                // handle_arabic_text_input, // DISABLED: Causes double input
                handle_unicode_text_input, // KEEP: Most comprehensive text input system
                handle_sort_placement_input,
            ))
            
            // Debug systems (optional)
            .add_systems(Update, debug_text_editor_state);
    }
} 