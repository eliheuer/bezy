//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::editing::selection::SelectionPlugin;
use crate::editing::sort::ActiveSortState;
use crate::editing::sort::SortEvent;
use crate::rendering::sort_renderer::{
    render_sorts_system, manage_sort_unicode_text, update_sort_unicode_text_positions,
    update_sort_unicode_text_colors,
};
use crate::systems::sort_manager::{
    enforce_single_active_sort, handle_sort_events, create_startup_sorts,
    make_first_sort_active_system, sync_sort_transforms, handle_glyph_navigation_changes,
    respawn_sort_points_on_glyph_change, spawn_sort_point_entities,
};
use bevy::prelude::*;

pub struct SortPlugin;

impl Plugin for SortPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SelectionPlugin)
            .init_resource::<ActiveSortState>()
            .add_event::<SortEvent>()
            .add_systems(
                Update,
                (
                    create_startup_sorts,
                    handle_sort_events,
                    sync_sort_transforms,
                    enforce_single_active_sort,
                    make_first_sort_active_system,
                    handle_glyph_navigation_changes,
                    spawn_sort_point_entities,
                    respawn_sort_points_on_glyph_change,
                    render_sorts_system,
                    manage_sort_unicode_text,
                    update_sort_unicode_text_positions,
                    update_sort_unicode_text_colors,
                ),
            );
    }
} 