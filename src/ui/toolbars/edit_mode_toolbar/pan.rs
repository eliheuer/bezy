use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use bevy::prelude::*;

pub struct PanTool;

impl EditTool for PanTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "pan"
    }
    
    fn name(&self) -> &'static str {
        "Pan"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E014}" // Pan/hand icon from the UI font
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some(' ') // Spacebar for temporary pan mode
    }
    
    fn default_order(&self) -> i32 {
        90 // Near the end, utility tool
    }
    
    fn description(&self) -> &'static str {
        "Pan and navigate the canvas"
    }
    
    fn supports_temporary_mode(&self) -> bool {
        true // Pan tool supports temporary activation with spacebar
    }
    
    fn update(&self, commands: &mut Commands) {
        // Ensure select mode is disabled while in pan mode
        commands.insert_resource(SelectModeActive(false));
    }
    
    fn on_enter(&self) {
        info!("Entered Pan tool");
    }
    
    fn on_exit(&self) {
        info!("Exited Pan tool");
    }
}

/// Plugin for the Pan tool
pub struct PanToolPlugin;

impl Plugin for PanToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_pan_tool);
    }
}

fn register_pan_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PanTool));
} 