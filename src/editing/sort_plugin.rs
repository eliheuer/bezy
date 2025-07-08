//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

// use crate::editing::selection::SelectionPlugin;
use crate::editing::sort::ActiveSortState;
use crate::editing::sort::SortEvent;
use crate::rendering::sort_renderer::{
    render_sorts_system, manage_sort_labels, update_sort_label_positions,
    update_sort_label_colors,
};
use crate::systems::sort_manager::{
    handle_sort_events, spawn_initial_sort,
    sync_sort_transforms, spawn_sort_point_entities,
    respawn_sort_points_on_glyph_change,
};
// use crate::systems::sort_interaction::handle_sort_clicks; // DISABLED: Old input system conflicts with selection
use bevy::prelude::*;

/// System sets for Sort management to ensure proper ordering
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SortSystemSet {
    Input,
    Management,
    PointSpawning,
    Rendering,
}

pub struct SortPlugin;

impl Plugin for SortPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(SelectionPlugin)
            .init_resource::<ActiveSortState>()
            .add_event::<SortEvent>()
            // Configure system sets to run in proper order
            .configure_sets(
                Update,
                (
                    SortSystemSet::Management,
                    SortSystemSet::PointSpawning,
                    SortSystemSet::Rendering,
                )
                    .chain(), // Ensure they run in order
            )
            // Management systems (events, basic sort operations)
            .add_systems(
                Update,
                (
                    spawn_initial_sort,
                    handle_sort_events,
                    sync_sort_transforms,
                    // REMOVED: enforce_single_active_sort, auto_activate_first_sort, handle_glyph_navigation_changes,
                    // These are now handled by TextEditorState + sync system
                )
                    .in_set(SortSystemSet::Management),
            )
            // Point spawning systems
            .add_systems(
                Update,
                (
                    spawn_sort_point_entities,
                    respawn_sort_points_on_glyph_change,
                )
                    .in_set(SortSystemSet::PointSpawning),
            );
            // Rendering systems (must run after all data is updated)
            // DISABLED: Old sort rendering system conflicts with new text editor system
            // The new text editor system (render_text_editor_sorts) handles all sort rendering
            // with proper TextBuffer vs Freeform style distinction
            // .add_systems(
            //     Update,
            //     (
            //         // render_sorts_system, // DISABLED: Conflicts with text editor rendering
            //         // manage_sort_unicode_text, // DISABLED: Old system
            //         // update_sort_unicode_text_positions, // DISABLED: Old system
            //         // update_sort_unicode_text_colors, // DISABLED: Old system
            //     )
            //         .in_set(SortSystemSet::Rendering),
            // );
    }
} 