//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::text_editor_sorts::{
    debug_text_editor_state,
    despawn_inactive_sort_points_optimized, // NEW: Optimized instant point despawning
    despawn_missing_buffer_sort_entities,   // NEW: Despawn deleted buffer sorts
    detect_sort_glyph_changes, // NEW: Detect glyph changes and force point regeneration
    // handle_text_input_with_cosmic, // DISABLED: Legacy system causing double input
    handle_arabic_text_input, // NEW: Arabic and Unicode text input
    regenerate_points_on_fontir_change, // NEW: Regenerate points when FontIR data changes
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
            .init_resource::<crate::systems::text_editor_sorts::sort_rendering::CursorRenderingState>()
            .init_resource::<crate::core::state::text_editor::ActiveSortEntity>()
            // Initialize text editor state
            .add_systems(Startup, initialize_text_editor_sorts)
            // Input handling
            .add_systems(Update, (
                handle_unicode_text_input,
                handle_sort_placement_input,
            ).in_set(super::FontEditorSets::Input))
            // Text buffer updates
            .add_systems(Update, (
                spawn_missing_sort_entities,
                crate::systems::text_editor_sorts::sort_entities::update_buffer_sort_positions,
                crate::systems::text_editor_sorts::sort_entities::auto_activate_selected_sorts,
                manage_sort_activation,
            ).chain().in_set(super::FontEditorSets::EntitySync))
            // Entity spawning/despawning 
            .add_systems(Update, (
                detect_sort_glyph_changes, // Detect glyph changes and trigger point regeneration
                spawn_active_sort_points_optimized,
                despawn_inactive_sort_points_optimized,
                regenerate_points_on_fontir_change, // Regenerate when FontIR data changes
            ).chain().in_set(super::FontEditorSets::EntitySync))
            // Rendering systems
            .add_systems(Update, (
                render_text_editor_sorts,
                crate::systems::text_editor_sorts::sort_rendering::render_text_editor_cursor,
            ).in_set(super::FontEditorSets::Rendering))
            // Cleanup systems (the old cleanup system is now replaced by component-relationship cleanup)
            .add_systems(Update, 
                despawn_missing_buffer_sort_entities
                    .in_set(super::FontEditorSets::Cleanup)
            )
            // Debug systems (no set - can run anytime)
            .add_systems(Update, debug_text_editor_state);
    }
}
