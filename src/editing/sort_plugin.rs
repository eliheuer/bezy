//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::editing::sort::ActiveSortState;
use crate::editing::sort::SortEvent;
use crate::rendering::sort_renderer::render_sorts_system;
use crate::systems::sort_manager::{
    auto_activate_first_sort, enforce_single_active_sort,
    handle_sort_events, spawn_initial_sort,
};
use bevy::prelude::*;

pub struct SortPlugin;

impl Plugin for SortPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ActiveSortState>()
            .add_event::<SortEvent>()
            .add_systems(
                Update,
                (
                    spawn_initial_sort,
                    handle_sort_events,
                    auto_activate_first_sort,
                    enforce_single_active_sort,
                    render_sorts_system,
                ).chain()
            );
    }
} 