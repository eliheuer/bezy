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
    enforce_single_active_sort, handle_sort_events, spawn_initial_sort,
    auto_activate_first_sort, sync_sort_transforms, handle_glyph_navigation_changes,
    respawn_sort_points_on_glyph_change, spawn_sort_point_entities,
};
use crate::systems::sort_interaction::handle_sort_clicks;
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
                    SortSystemSet::Input,
                    SortSystemSet::Management,
                    SortSystemSet::PointSpawning,
                    SortSystemSet::Rendering,
                )
                    .chain(), // Ensure they run in order
            )
            // Input systems (clicks and interactions)
            .add_systems(
                Update,
                handle_sort_clicks.in_set(SortSystemSet::Input),
            )
            // Management systems (events, activation, etc.)
            .add_systems(
                Update,
                (
                    spawn_initial_sort,
                    handle_sort_events,
                    sync_sort_transforms,
                    enforce_single_active_sort,
                    auto_activate_first_sort,
                    handle_glyph_navigation_changes,
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
            )
            // Rendering systems (must run after all data is updated)
            .add_systems(
                Update,
                (
                    render_sorts_system,
                    manage_sort_labels,
                    update_sort_label_positions,
                    update_sort_label_colors,
                )
                    .in_set(SortSystemSet::Rendering),
            );
    }
} 