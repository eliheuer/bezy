use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "shapes"
    }
    
    fn name(&self) -> &'static str {
        "Shapes"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E012}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('s')
    }
    
    fn default_order(&self) -> i32 {
        30 // Third tool after pen
    }
    
    fn description(&self) -> &'static str {
        "Draw basic shapes like rectangles and ellipses"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implementation for shapes tool update
    }
    
    fn on_enter(&self) {
        info!("Entered Shapes tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Shapes tool");
    }
}

/// Plugin for the Shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_shapes_tool);
    }
}

fn register_shapes_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(ShapesTool));
} 