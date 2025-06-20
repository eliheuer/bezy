use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct PenTool;

impl EditTool for PenTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "pen"
    }
    
    fn name(&self) -> &'static str {
        "Pen"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E011}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('p')
    }
    
    fn default_order(&self) -> i32 {
        20 // Second tool after select
    }
    
    fn description(&self) -> &'static str {
        "Draw and edit paths with bezier curves"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implementation for pen tool update
    }
    
    fn on_enter(&self) {
        info!("Entered Pen tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Pen tool");
    }
}

/// Plugin for the Pen tool
pub struct PenToolPlugin;

impl Plugin for PenToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_pen_tool);
    }
}

fn register_pen_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PenTool));
} 