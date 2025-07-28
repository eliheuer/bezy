//! Coordination between outline rendering systems
//! 
//! This module prevents double rendering by coordinating between
//! mesh_glyph_outline and unified_glyph_editing systems.

use bevy::prelude::*;
use std::collections::HashSet;

/// Resource that tracks which sorts should use unified rendering
/// When a sort is in this set, mesh_glyph_outline should skip it
#[derive(Resource, Default)]
pub struct UnifiedRenderingSorts {
    /// Set of sort entities that should use unified rendering
    pub sorts: HashSet<Entity>,
}

impl UnifiedRenderingSorts {
    /// Mark a sort as using unified rendering
    pub fn insert(&mut self, sort_entity: Entity) {
        self.sorts.insert(sort_entity);
    }
    
    /// Remove a sort from unified rendering
    pub fn remove(&mut self, sort_entity: Entity) {
        self.sorts.remove(&sort_entity);
    }
    
    /// Check if a sort should use unified rendering
    pub fn contains(&self, sort_entity: Entity) -> bool {
        self.sorts.contains(&sort_entity)
    }
    
    /// Clear all sorts
    pub fn clear(&mut self) {
        self.sorts.clear();
    }
}

pub struct OutlineCoordinationPlugin;

impl Plugin for OutlineCoordinationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnifiedRenderingSorts>();
    }
}