//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::text_editor_sorts::{
    debug_text_editor_state,
    despawn_inactive_sort_points_optimized, // NEW: Optimized instant point despawning
    // handle_text_input_with_cosmic, // DISABLED: Legacy system causing double input
    handle_arabic_text_input, // NEW: Arabic and Unicode text input
    // respawn_active_sort_points, // REMOVED: Replaced with ECS-based system
    handle_sort_placement_input, // NEW: Uses centralized input system
    handle_text_editor_keyboard_input,
    handle_unicode_text_input, // NEW: Unicode character input using Bevy events
    initialize_text_editor_sorts,
    manage_sort_activation, // NEW: ECS-based sort activation management
    // handle_text_editor_sort_clicks, // REMOVED: legacy system
    render_text_editor_sorts,
    spawn_active_sort_points_optimized, // NEW: Optimized instant point spawning
    spawn_missing_sort_entities, // NEW: Spawn ECS entities for buffer sorts
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        info!("[TextEditorPlugin] Building plugin...");
        app
            // Initialize resources
            .init_resource::<crate::core::state::text_editor::TextEditorState>()
            .init_resource::<crate::core::state::text_editor::ActiveSortEntity>()
            // Initialize text editor state
            .add_systems(Startup, initialize_text_editor_sorts)
            // Sort activation management (runs after input)
            .add_systems(Update, (
                // manage_sort_activation, // DISABLED: Use selection system instead
                spawn_missing_sort_entities,
                crate::systems::text_editor_sorts::sort_entities::update_buffer_sort_positions, // NEW: Update existing sort positions
                // crate::systems::text_editor_sorts::sort_entities::auto_activate_selected_sorts, // TEMPORARILY DISABLED: May be interfering with text root activation
                crate::systems::text_editor_sorts::despawn_missing_buffer_sort_entities, // NEW: Despawn deleted buffer sorts
            ).chain().after(handle_unicode_text_input))
            // Instant point spawning/despawning (runs immediately after activation)
            .add_systems(Update, (
                spawn_active_sort_points_optimized,
                despawn_inactive_sort_points_optimized,
            ).chain().after(manage_sort_activation))
            // Input handling (must run before spawning)
            .add_systems(Update, handle_unicode_text_input)
            // Rendering and input handling (rendering must run after nudging)
            .add_systems(Update, (
                render_text_editor_sorts
                    .after(crate::editing::selection::nudge::handle_nudge_input),
                crate::systems::text_editor_sorts::sort_rendering::render_text_editor_cursor
                    .after(crate::editing::selection::nudge::handle_nudge_input),
                // handle_text_editor_keyboard_input, // DISABLED: Replaced by Unicode input system
                // handle_arabic_text_input, // DISABLED: Replaced by Unicode input system  
                handle_sort_placement_input,
            ).after(handle_unicode_text_input))
            // Debug systems (optional)
            .add_systems(Update, debug_text_editor_state);
    }
}
