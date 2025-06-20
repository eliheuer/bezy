use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "knife"
    }
    
    fn name(&self) -> &'static str {
        "Knife"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E017}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }
    
    fn default_order(&self) -> i32 {
        70 // After measure, before hyper
    }
    
    fn description(&self) -> &'static str {
        "Cut and slice paths"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implementation for knife tool update
    }
    
    fn on_enter(&self) {
        info!("Entered Knife tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Knife tool");
    }
}

/// Plugin for the Knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_knife_tool);
    }
}

fn register_knife_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(KnifeTool));
} 