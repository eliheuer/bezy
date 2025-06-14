//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::editing::sort::{ActiveSortState, SortEvent};
use crate::systems::sort_manager::{
    auto_activate_first_sort, enforce_single_active_sort, handle_sort_events,
    spawn_initial_sort, sync_sort_transforms,
};
use bevy::prelude::*;

/// System sets for Sort management
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SortSystemSet {
    Management,
    PointSpawning,
    DataUpdate,
    Rendering,
}

/// Plugin that adds all sort functionality to the application
pub struct SortPlugin;

impl Plugin for SortPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add sort events
            .add_event::<SortEvent>()
            // Add sort resources
            .init_resource::<ActiveSortState>()
            .configure_sets(
                Update,
                (
                    SortSystemSet::Management,
                    SortSystemSet::PointSpawning,
                    SortSystemSet::DataUpdate,
                    SortSystemSet::Rendering,
                )
                    .chain(),
            )
            // Sort management systems
            .add_systems(
                Update,
                (
                    spawn_initial_sort,
                    handle_sort_events,
                    sync_sort_transforms,
                    enforce_single_active_sort,
                    auto_activate_first_sort,
                )
                    .in_set(SortSystemSet::Management),
            );
        // // Point and crosshair entity management
        // .add_systems(
        //     Update,
        //     (
        //         // spawn_sort_point_entities,
        //         // manage_sort_crosshairs,
        //         // manage_newly_spawned_crosshairs,
        //         // respawn_sort_points_on_glyph_change,
        //         // debug_sort_point_entities,
        //     )
        //         .in_set(SortSystemSet::PointSpawning)
        //         .after(SortSystemSet::Management),
        // )
        // // Sort data updates - this includes moving the sort based on crosshair
        // .add_systems(
        //     Update,
        //     (
        //         // update_sort_glyph_data,
        //         // update_sort_from_crosshair_move,
        //         // sync_points_to_sort_move,
        //         // sync_crosshair_to_sort_move,
        //     )
        //         .in_set(SortSystemSet::DataUpdate)
        //         .after(SortSystemSet::PointSpawning),
        // )
        // // Sort rendering and unicode text management
        // .add_systems(
        //     Update,
        //     (
        //         // render_sorts_system,
        //         // manage_sort_unicode_text,
        //         // update_sort_unicode_text_positions,
        //         // update_sort_unicode_text_colors,
        //         // render_sort_crosshairs,
        //         // sync_crosshair_to_sort_move,
        //         // debug_sort_point_entities,
        //     )
        //         .in_set(SortSystemSet::Rendering)
        //         .after(SortSystemSet::DataUpdate),
        // )
        // // Interactions need to be handled alongside or after main input processing
        // .add_systems(
        //     Update,
        //     (
        //         // handle_sort_clicks.in_set(SelectionSystemSet::Input),
        //         // // Move crosshair sync to run after selection input to avoid interference
        //         // sync_crosshair_to_sort_move.after(SelectionSystemSet::Input),
        //     ),
        // );
    }
} 