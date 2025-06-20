//! # Edit Mode Toolbar Module
//!
//! This is the main module for the edit mode toolbar system, which provides a comprehensive
//! tool switching interface for graphics and design applications. It orchestrates all the
//! different editing modes and their associated UI components.
//!
//! ## Architecture Overview
//!
//! The module uses a dynamic tool registration system that allows tools to be easily added
//! by simply implementing the `EditTool` trait and registering with the `ToolRegistry`.
//!
//! ### Core Components
//! - **Tool Registry**: Dynamic registration system for edit tools
//! - **EditTool Trait**: Common interface that all tools must implement
//! - **UI Module**: Handles the visual toolbar interface and mode switching
//! - **Plugin System**: Bevy plugin that coordinates everything
//!
//! ### Adding New Tools
//!
//! The system is designed for maximum ease of use. Adding a new tool requires:
//!
//! 1. **Create your tool file** (`my_tool.rs`):
//!    ```rust
//!    use bevy::prelude::*;
//!    use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
//!
//!    pub struct MyTool;
//!
//!    impl EditTool for MyTool {
//!        fn id(&self) -> ToolId { "my_tool" }
//!        fn name(&self) -> &'static str { "My Tool" }
//!        fn icon(&self) -> &'static str { "\u{E019}" }
//!        fn default_order(&self) -> i32 { 50 }
//!        
//!        fn update(&self, commands: &mut Commands) {
//!            // Your tool's behavior while active
//!        }
//!    }
//!
//!    pub struct MyToolPlugin;
//!    impl Plugin for MyToolPlugin {
//!        fn build(&self, app: &mut App) {
//!            app.add_systems(Startup, register_my_tool);
//!        }
//!    }
//!
//!    fn register_my_tool(mut registry: ResMut<ToolRegistry>) {
//!        registry.register_tool(Box::new(MyTool));
//!    }
//!    ```
//!
//! 2. **Add module declaration**: `mod my_tool;` and `pub use my_tool::MyToolPlugin;`
//!
//! 3. **Register plugin**: `app.add_plugins(MyToolPlugin);`
//!
//! The tool automatically appears in the toolbar with proper ordering and functionality.

use bevy::prelude::*;
use std::collections::HashMap;

mod select;
mod ui;
mod pan;
mod measure;
mod text;
mod pen;
mod shapes;
mod knife;
mod hyper;

// Re-export the UI system
pub use ui::{handle_toolbar_mode_selection, spawn_edit_mode_toolbar, update_current_edit_mode};

// Re-export tool plugins
pub use select::SelectToolPlugin;
pub use pan::PanToolPlugin;
pub use measure::MeasureToolPlugin;
pub use text::TextToolPlugin;
pub use pen::PenToolPlugin;
pub use shapes::ShapesToolPlugin;
pub use knife::KnifeToolPlugin;
pub use hyper::HyperToolPlugin;

/// Unique identifier for an edit tool
pub type ToolId = &'static str;

/// Trait that defines a complete edit tool with all its metadata and behavior.
///
/// This trait provides everything needed to create a fully functional toolbar tool.
/// Tools are self-contained and define their own appearance, behavior, and lifecycle.
///
/// # Required Methods
///
/// - `id()`: Unique string identifier (e.g., "select", "pen", "eraser")
/// - `name()`: Human-readable name displayed in UI
/// - `icon()`: Unicode character for the toolbar button
/// - `update()`: Core tool behavior, called every frame while active
///
/// # Optional Methods
///
/// All other methods have sensible defaults but can be overridden:
/// - `shortcut_key()`: Keyboard shortcut (e.g., Some('v') for Select)
/// - `default_order()`: Toolbar position (lower numbers = earlier position)
/// - `description()`: Tooltip text and help documentation
/// - `on_enter()`: Setup when tool becomes active
/// - `on_exit()`: Cleanup when switching away from tool
/// - `supports_temporary_mode()`: Whether tool can be temporarily activated
pub trait EditTool: Send + Sync + 'static {
    /// Unique identifier for this tool (used internally)
    fn id(&self) -> ToolId;
    
    /// Display name shown in UI
    fn name(&self) -> &'static str;
    
    /// Unicode icon character for the toolbar button
    fn icon(&self) -> &'static str;
    
    /// Optional keyboard shortcut (e.g., Some('v') for Select tool)
    fn shortcut_key(&self) -> Option<char> { None }
    
    /// Default ordering priority (lower numbers appear first)
    /// Common values: 10=Select, 20=Pen, 30=Eraser, 50=Shapes, 100=Utilities
    fn default_order(&self) -> i32 { 100 }
    
    /// Description for tooltips/help
    fn description(&self) -> &'static str { "" }
    
    /// Called every frame while this tool is active
    fn update(&self, commands: &mut Commands);
    
    /// Called when switching to this tool
    fn on_enter(&self) {}
    
    /// Called when switching away from this tool  
    fn on_exit(&self) {}
    
    /// Whether this tool supports temporary activation (e.g., spacebar for pan)
    fn supports_temporary_mode(&self) -> bool { false }
}

/// Registry for all available edit tools.
///
/// This resource manages all registered tools and their display order in the toolbar.
/// Tools register themselves during app startup, and the registry handles ordering
/// based on `default_order()` values and custom ordering preferences.
#[derive(Resource, Default)]
pub struct ToolRegistry {
    tools: HashMap<ToolId, Box<dyn EditTool>>,
    ordered_tool_ids: Vec<ToolId>,
    ordering_dirty: bool,
}

impl ToolRegistry {
    /// Register a new tool with the registry
    pub fn register_tool(&mut self, tool: Box<dyn EditTool>) {
        let id = tool.id();
        info!("Registering tool: {} ({})", tool.name(), id);
        self.tools.insert(id, tool);
        self.ordering_dirty = true;
    }

    /// Get a tool by ID
    pub fn get_tool(&self, id: ToolId) -> Option<&dyn EditTool> {
        self.tools.get(id).map(|tool| tool.as_ref())
    }

    /// Get all tools in their display order
    pub fn get_ordered_tools(&mut self) -> &[ToolId] {
        if self.ordering_dirty {
            self.rebuild_ordering();
        }
        &self.ordered_tool_ids
    }

    /// Get all tool IDs (unordered)
    pub fn get_all_tool_ids(&self) -> Vec<ToolId> {
        self.tools.keys().copied().collect()
    }

    /// Rebuild the tool ordering based on default_order() values
    fn rebuild_ordering(&mut self) {
        let mut tools_with_order: Vec<(ToolId, i32)> = self.tools
            .iter()
            .map(|(id, tool)| (*id, tool.default_order()))
            .collect();
        
        // Sort by order value, then by ID for consistent ordering
        tools_with_order.sort_by(|a, b| {
            a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
        });
        
        self.ordered_tool_ids = tools_with_order.into_iter().map(|(id, _)| id).collect();
        self.ordering_dirty = false;
        
        info!("Tool ordering rebuilt: {:?}", self.ordered_tool_ids);
    }

    /// Apply custom ordering (for future extensibility)
    pub fn apply_custom_ordering(&mut self, custom_order: &[ToolId]) {
        // For now, just use the custom order if provided
        if !custom_order.is_empty() {
            self.ordered_tool_ids = custom_order.to_vec();
            self.ordering_dirty = false;
        }
    }
}

/// Resource to track the current and previous tools
#[derive(Resource, Default)]
pub struct CurrentTool {
    pub current: Option<ToolId>,
    pub previous: Option<ToolId>,
}

impl CurrentTool {
    /// Switch to a new tool
    pub fn switch_to(&mut self, new_tool: ToolId) {
        self.previous = self.current;
        self.current = Some(new_tool);
        info!("Switched from {:?} to {}", self.previous, new_tool);
    }

    /// Get the current tool ID
    pub fn get_current(&self) -> Option<ToolId> {
        self.current
    }

    /// Get the previous tool ID
    pub fn get_previous(&self) -> Option<ToolId> {
        self.previous
    }
}

/// Initialize the default tool (Select)
fn initialize_default_tool(
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    // Set the default tool to Select if it's available
    if tool_registry.get_tool("select").is_some() {
        current_tool.switch_to("select");
    }
}

/// Main plugin for the edit mode toolbar system
pub struct EditModeToolbarPlugin;

impl Plugin for EditModeToolbarPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<ToolRegistry>()
            .init_resource::<CurrentTool>()
            // Add tool plugins
            .add_plugins((
                SelectToolPlugin,
                PanToolPlugin,
                MeasureToolPlugin,
                TextToolPlugin,
                PenToolPlugin,
                ShapesToolPlugin,
                KnifeToolPlugin,
                HyperToolPlugin,
            ))
            // Add UI systems
            .add_systems(Startup, (
                spawn_edit_mode_toolbar,
                initialize_default_tool.after(spawn_edit_mode_toolbar),
            ))
            .add_systems(Update, (
                handle_toolbar_mode_selection,
                update_current_edit_mode,
            ));
    }
} 