//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::editing::sort::{SortEvent, ActiveSortState};
use crate::editing::selection::SelectionSystemSet;
use crate::rendering::sort_renderer::render_sorts_system;
use crate::systems::sort_interaction::handle_sort_clicks;
use crate::systems::sort_manager::{
    handle_sort_events, sync_sort_transforms, enforce_single_active_sort, 
    spawn_sort_point_entities, update_sort_glyph_data, spawn_initial_sort,
    auto_activate_first_sort, handle_glyph_navigation_changes, respawn_sort_points_on_glyph_change,
    debug_sort_point_entities
};
use crate::ui::toolbars::edit_mode_toolbar::text::{TextModePlugin};
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
            
            // Add the text mode plugin for sort placement
            .add_plugins(TextModePlugin)
            
            // Configure system sets to run in proper order
            .configure_sets(
                Update,
                (
                    SortSystemSet::Management,
                    SortSystemSet::PointSpawning,
                    SortSystemSet::DataUpdate,
                    SortSystemSet::Rendering,
                )
                    .chain()
                    .before(SelectionSystemSet::Input), // Ensure sorts run before selection
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
                    handle_glyph_navigation_changes,
                )
                    .in_set(SortSystemSet::Management),
            )
            
            // Point entity management systems
            .add_systems(
                Update,
                (
                    spawn_sort_point_entities,
                    // respawn_sort_points_on_glyph_change, // Temporarily disabled to test dragging
                    debug_sort_point_entities,
                )
                    .in_set(SortSystemSet::PointSpawning)
                    .after(SortSystemSet::Management),
            )
            
            // Sort data updates
            .add_systems(
                Update,
                (
                    update_sort_glyph_data,
                )
                    .in_set(SortSystemSet::DataUpdate)
                    .after(SortSystemSet::PointSpawning),
            )
            
            // Sort rendering
            .add_systems(
                Update,
                (
                    render_sorts_system,
                )
                    .in_set(SortSystemSet::Rendering)
                    .after(SortSystemSet::DataUpdate),
            )
            
            // Sort interaction (needs to run with selection input)
            .add_systems(
                Update,
                handle_sort_clicks.in_set(SelectionSystemSet::Input),
            );
    }
} 