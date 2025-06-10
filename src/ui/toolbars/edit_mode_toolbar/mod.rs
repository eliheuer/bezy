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
//!
//! ## Key Features
//!
//! - **Dynamic Registration**: Tools register themselves at runtime
//! - **Configurable Ordering**: Control the order tools appear in the toolbar
//! - **Temporary Modes**: Support for temporary mode activation (e.g., holding spacebar for pan)
//! - **State Management**: Proper enter/exit lifecycle for each tool
//! - **UI Integration**: Visual feedback and interactive toolbar
//! - **Extensibility**: Easy to add new tools with minimal code changes

use bevy::prelude::*;
use std::collections::HashMap;

mod hyper;
pub mod knife;
mod measure;
mod pan;
mod pen;
mod primitives;
pub mod select;
pub mod text;
mod ui;

// Add the temporary mode switching module
mod temporary_mode;

// Re-export the new tool system
pub use ui::{handle_toolbar_mode_selection, spawn_edit_mode_toolbar, update_current_edit_mode};
pub use temporary_mode::{handle_temporary_mode_switching, TemporaryModeState};

// Re-export legacy types for backward compatibility
pub use ui::{CurrentEditMode, EditMode};

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
///
/// # Example Implementation
///
/// ```rust
/// use bevy::prelude::*;
/// use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolId};
///
/// pub struct MyCustomTool {
///     // Tool-specific state can go here
///     pub some_setting: bool,
/// }
///
/// impl EditTool for MyCustomTool {
///     fn id(&self) -> ToolId { "my_custom_tool" }
///     fn name(&self) -> &'static str { "Custom Tool" }
///     fn icon(&self) -> &'static str { "\u{E020}" }
///     fn shortcut_key(&self) -> Option<char> { Some('c') }
///     fn default_order(&self) -> i32 { 75 }
///     
///     fn update(&self, commands: &mut Commands) {
///         // Implement tool behavior (handle input, modify entities, etc.)
///         // This runs every frame while the tool is active
///     }
///     
///     fn on_enter(&self) {
///         info!("Custom tool activated!");
///         // Setup tool state, change cursor, etc.
///     }
///     
///     fn on_exit(&self) {
///         info!("Custom tool deactivated!");
///         // Cleanup, restore cursor, etc.
///     }
/// }
/// ```
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
///
/// # Usage
///
/// Tools typically register themselves in their plugin's `build()` method:
///
/// ```rust
/// fn register_my_tool(mut tool_registry: ResMut<ToolRegistry>) {
///     tool_registry.register_tool(Box::new(MyTool));
/// }
/// ```
///
/// The registry automatically handles ordering and makes tools available to the UI system.
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
        info!("Registering tool: {}", id);
        self.tools.insert(id, tool);
        self.ordering_dirty = true;
    }
    
    /// Get a tool by its ID
    pub fn get_tool(&self, id: ToolId) -> Option<&dyn EditTool> {
        self.tools.get(id).map(|t| t.as_ref())
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
    
    /// Rebuild the tool ordering based on default_order() and custom overrides
    fn rebuild_ordering(&mut self) {
        let mut tools_with_order: Vec<(ToolId, i32)> = self.tools
            .values()
            .map(|tool| (tool.id(), tool.default_order()))
            .collect();
            
        // Sort by order, then by name for stable sorting
        tools_with_order.sort_by(|a, b| {
            a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
        });
        
        self.ordered_tool_ids = tools_with_order.into_iter().map(|(id, _)| id).collect();
        self.ordering_dirty = false;
    }

    /// Apply custom ordering if configured
    pub fn apply_custom_ordering(&mut self, custom_order: &ToolOrdering) {
        if !custom_order.custom_order.is_empty() {
            let mut ordered_tools = Vec::new();
            
            // First add tools in custom order
            for &tool_id in &custom_order.custom_order {
                if self.tools.contains_key(tool_id) {
                    ordered_tools.push(tool_id);
                }
            }
            
            // Then add any remaining tools not in custom order (using default ordering)
            let mut remaining_tools: Vec<(ToolId, i32)> = self.tools
                .values()
                .filter(|tool| !custom_order.custom_order.contains(&tool.id()))
                .map(|tool| (tool.id(), tool.default_order()))
                .collect();
                
            remaining_tools.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            
            for (tool_id, _) in remaining_tools {
                ordered_tools.push(tool_id);
            }
            
            self.ordered_tool_ids = ordered_tools;
            self.ordering_dirty = false;
        } else {
            // Use default ordering
            self.rebuild_ordering();
        }
    }
}

/// Resource tracking the currently active tool
#[derive(Resource, Default)]
pub struct CurrentTool {
    pub current: Option<ToolId>,
    pub previous: Option<ToolId>,
}

impl CurrentTool {
    pub fn switch_to(&mut self, new_tool: ToolId) {
        if self.current != Some(new_tool) {
            self.previous = self.current;
            self.current = Some(new_tool);
        }
    }
    
    pub fn get_current(&self) -> Option<ToolId> {
        self.current
    }
    
    pub fn get_previous(&self) -> Option<ToolId> {
        self.previous
    }
}

/// Custom tool ordering configuration (optional).
///
/// This resource allows you to override the default tool ordering in the toolbar.
/// By default, tools are ordered by their `default_order()` values, but you can
/// specify a custom order here.
///
/// # Usage Examples
///
/// ## Setting Custom Order
///
/// ```rust
/// fn setup_toolbar_order(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Put select first, then pen, then custom tools
///     tool_ordering.set_order(vec![
///         "select",
///         "pen",
///         "my_custom_tool",
///         "eraser",
///         // Any unspecified tools will appear after these in default order
///     ]);
/// }
/// ```
///
/// ## Using Preset Orders
///
/// ```rust
/// fn setup_design_workflow(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Optimized for design-focused work
///     tool_ordering.set_design_focused_order();
/// }
///
/// fn setup_annotation_workflow(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Optimized for annotation and markup work
///     tool_ordering.set_annotation_focused_order();
/// }
/// ```
///
/// ## Dynamic Ordering
///
/// ```rust
/// fn setup_user_preference_order(
///     mut tool_ordering: ResMut<ToolOrdering>,
///     user_prefs: Res<UserPreferences>
/// ) {
///     match user_prefs.workflow_type {
///         WorkflowType::Design => tool_ordering.set_design_focused_order(),
///         WorkflowType::Annotation => tool_ordering.set_annotation_focused_order(),
///         WorkflowType::Custom => tool_ordering.set_order(user_prefs.custom_tool_order.clone()),
///     }
/// }
/// ```
#[derive(Resource, Default)]
pub struct ToolOrdering {
    pub custom_order: Vec<ToolId>,
}

impl ToolOrdering {
    /// Set a custom order for tools (tools not listed will use default ordering after these)
    pub fn set_order(&mut self, order: Vec<ToolId>) {
        self.custom_order = order;
    }

    /// Convenient method to set common tool orders
    pub fn set_design_focused_order(&mut self) {
        self.custom_order = vec![
            "select",
            "pen", 
            "eraser",
            "primitives",
            "text",
            "measure",
            "knife",
            "hyper",
            "pan",
        ];
    }

    /// Convenient method to set annotation-focused order  
    pub fn set_annotation_focused_order(&mut self) {
        self.custom_order = vec![
            "select",
            "text",
            "pen",
            "primitives", 
            "measure",
            "eraser",
            "knife",
            "hyper",
            "pan",
        ];
    }
}

// New tool exports
pub use hyper::HyperModePlugin;
pub use knife::KnifeModePlugin;
pub use measure::MeasureToolPlugin;
pub use text::TextToolPlugin;

// Legacy compatibility exports (will be removed after migration)
pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use pan::{PanMode, PanToolPlugin};
pub use pen::{PenMode, PenToolPlugin};
pub use primitives::{
    handle_primitive_mouse_events, render_active_primitive_drawing,
    ActivePrimitiveDrawing, CurrentCornerRadius, UiInteractionState,
    handle_primitive_selection, spawn_primitives_submenu, 
    toggle_primitive_submenu_visibility, CurrentPrimitiveType, 
    PrimitiveType, PrimitivesToolPlugin,
};

pub use select::{SelectMode, SelectToolPlugin};
pub use text::TextMode;
pub use measure::MeasureMode;

// Legacy trait (will be removed after migration)
pub trait EditModeSystem: Send + Sync + 'static {
    fn update(&self, commands: &mut Commands);
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}

// Legacy compatibility - will be removed after migration
pub struct PrimitivesMode;
impl EditModeSystem for PrimitivesMode {
    fn update(&self, _commands: &mut Commands) {}
}

/// System to initialize the default tool (Select) on startup
fn initialize_default_tool(
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    info!("Attempting to initialize default tool...");
    // Set Select as the default tool
    if let Some(_) = tool_registry.get_tool("select") {
        current_tool.switch_to("select");
        info!("Initialized with default tool: select");
    } else {
        warn!("Select tool not found in registry!");
    }
}

/// Plugin that adds all the toolbar functionality
pub struct EditModeToolbarPlugin;

impl Plugin for EditModeToolbarPlugin {
    fn build(&self, app: &mut App) {
        println!("Building EditModeToolbarPlugin!");
        app
            // Initialize the new tool system
            .init_resource::<ToolRegistry>()
            .init_resource::<CurrentTool>()
            .init_resource::<ToolOrdering>()
            
            // Legacy resources (will be removed after migration)
            .init_resource::<CurrentPrimitiveType>()
            .init_resource::<ActivePrimitiveDrawing>()
            .init_resource::<CurrentCornerRadius>()
            .init_resource::<UiInteractionState>()
            .init_resource::<TemporaryModeState>()
            
            // Add tool plugins (they will register themselves)
            .add_plugins(SelectToolPlugin)
            .add_plugins(PanToolPlugin)
            .add_plugins(MeasureToolPlugin)
            .add_plugins(TextToolPlugin)
            .add_plugins(PenToolPlugin)
            .add_plugins(PrimitivesToolPlugin)
            .add_plugins(knife::KnifeModePlugin)
            .add_plugins(hyper::HyperModePlugin)
            
            .add_systems(PostStartup, (
                spawn_edit_mode_toolbar,
                initialize_default_tool.after(spawn_edit_mode_toolbar),
                handle_primitive_selection
            ))
            .add_systems(
                Update,
                (
                    // Temporary mode switching (should run first to potentially change current mode)
                    handle_temporary_mode_switching,
                    // UI systems
                    handle_toolbar_mode_selection,
                    update_current_edit_mode,
                    // Primitives sub-menu systems
                    handle_primitive_selection,
                    toggle_primitive_submenu_visibility,
                    // Mouse event handling for drawing shapes
                    handle_primitive_mouse_events,
                    // Render the active primitive shape while drawing
                    render_active_primitive_drawing,
                ),
            );
    }
}
