//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::editing::sort::{
    ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent,
};
use crate::rendering::cameras::DesignCamera;
use crate::rendering::sort_renderer::{
    manage_sort_labels, render_sorts_system, update_sort_label_colors,
    update_sort_label_positions,
};
use crate::systems::sort_manager::{
    handle_sort_events, respawn_sort_points_on_glyph_change, spawn_initial_sort,
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
                    // REMOVED: enforce_single_active_sort, auto_activate_first_sort, handle_glyph_navigation_changes,
                    // These are now handled by TextEditorState + sync system
                )
                    .in_set(SortSystemSet::Management),
            )
            // Point spawning systems
            .add_systems(
                Update,
                (
                    // spawn_sort_point_entities, // DISABLED: Causes duplicate point entities
                    respawn_sort_points_on_glyph_change,
                )
                    .in_set(SortSystemSet::PointSpawning),
            )
            // Rendering systems (must run after all data is updated)
            // Re-enabled: Old sort rendering system for freeform sorts (glyph grid)
            .add_systems(
                Update,
                (
                    render_sorts_system // Re-enabled for freeform sorts (glyph grid)
                        .after(crate::editing::selection::nudge::handle_nudge_input),
                                         // manage_sort_unicode_text, // DISABLED: Old system
                                         // update_sort_unicode_text_positions, // DISABLED: Old system
                                         // update_sort_unicode_text_colors, // DISABLED: Old system
                )
                    .in_set(SortSystemSet::Rendering),
            );
    }
}
