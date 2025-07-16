//! Font editing tools
//!
//! This module contains all the tools for editing fonts in Bezy.
//! Each tool implements the EditTool trait and provides specific functionality.

pub mod adapters;
pub mod hyper;
pub mod knife;
pub mod measure;
pub mod metaballs;
pub mod pan;
pub mod pen;
pub mod select;
pub mod shapes;
pub mod text;

// Re-export all tools
pub use adapters::*;
pub use hyper::*;
pub use knife::*;
pub use measure::*;
pub use metaballs::*;
pub use pan::*;
pub use pen::*;
pub use select::*;
pub use shapes::*;
pub use text::*;

use bevy::prelude::*;

/// Information about a tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub shortcut: Option<KeyCode>,
}

/// Trait that all editing tools must implement
pub trait EditTool: Send + Sync {
    /// Get information about this tool
    fn info(&self) -> ToolInfo;

    /// Called when the tool is activated
    fn on_activate(&mut self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }

    /// Called when the tool is deactivated
    fn on_deactivate(&mut self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }

    /// Called every frame while the tool is active
    fn update(&self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }
}

// Note: Tools are now registered via adapters.rs to bridge with legacy system
