use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct MeasureTool;

impl EditTool for MeasureTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "measure"
    }
    
    fn name(&self) -> &'static str {
        "Measure"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E015}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }
    
    fn default_order(&self) -> i32 {
        60 // After text tool, before pan
    }
    
    fn description(&self) -> &'static str {
        "Measure distances and dimensions"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implementation for measure tool update
        // TODO: Add distance and angle measurement functionality
    }
    
    fn on_enter(&self) {
        info!("Entered Measure tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Measure tool");
    }
}

/// Plugin for the Measure tool
pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_measure_tool);
    }
}

fn register_measure_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(MeasureTool));
} 