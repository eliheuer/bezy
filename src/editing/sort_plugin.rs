//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::editing::sort::{SortEvent, ActiveSortState};
use crate::rendering::sort_renderer::render_sorts_system;
use crate::systems::sort_interaction::handle_sort_clicks;
use crate::systems::sort_manager::{
    handle_sort_events, sync_sort_transforms, enforce_single_active_sort, spawn_sort_point_entities, update_sort_glyph_data
};
use crate::ui::toolbars::edit_mode_toolbar::text::{TextModePlugin};
use bevy::prelude::*;

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
            
            // Add sort management systems
            .add_systems(
                Update,
                (
                    // Sort event handling
                    handle_sort_events,
                    
                    // Sort interaction
                    handle_sort_clicks,
                    
                    // Sort state management
                    sync_sort_transforms,
                    enforce_single_active_sort,
                    
                    // Sort point entity management
                    spawn_sort_point_entities,
                    
                    // Sort glyph data updates
                    update_sort_glyph_data,
                    
                    // Sort rendering
                    render_sorts_system,
                )
                .chain(), // Run in order to ensure proper state management
            );
    }
} 