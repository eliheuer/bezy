use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct TextTool;

impl EditTool for TextTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "text"
    }
    
    fn name(&self) -> &'static str {
        "Text"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E016}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }
    
    fn default_order(&self) -> i32 {
        50 // After shapes, before measure
    }
    
    fn description(&self) -> &'static str {
        "Add and edit text"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Implementation for text tool update
    }
    
    fn on_enter(&self) {
        info!("Entered Text tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Text tool");
    }
}

/// Plugin for the Text tool
pub struct TextToolPlugin;

impl Plugin for TextToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_text_tool);
    }
}

fn register_text_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(TextTool));
} 