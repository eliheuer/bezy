use bevy::prelude::*;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry, EditModeSystem};

/// Plugin to register selection mode systems
pub struct SelectModePlugin;

impl Plugin for SelectModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectModeActive>();
    }
}

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

pub struct SelectTool;

impl EditTool for SelectTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "select"
    }
    
    fn name(&self) -> &'static str {
        "Select"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E010}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('v')
    }
    
    fn default_order(&self) -> i32 {
        10 // First tool in the toolbar
    }
    
    fn description(&self) -> &'static str {
        "Select and manipulate objects"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implement select tool behavior
        // This will contain the actual selection logic
    }
    
    fn on_enter(&self) {
        info!("Entered Select tool");
        // Setup selection mode
    }
    
    fn on_exit(&self) {
        info!("Exited Select tool");
        // Cleanup selection mode
    }
}

// Legacy compatibility struct
pub struct SelectMode;

impl EditModeSystem for SelectMode {
    fn update(&self, _commands: &mut Commands) {
        // Legacy implementation
    }
}

/// Plugin for the Select tool
pub struct SelectToolPlugin;

impl Plugin for SelectToolPlugin {
    fn build(&self, app: &mut App) {
        // Register the select tool
        app.add_systems(Startup, register_select_tool);
    }
}

fn register_select_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(SelectTool));
}
