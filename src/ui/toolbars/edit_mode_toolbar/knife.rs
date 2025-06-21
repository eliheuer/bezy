use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "knife"
    }
    
    fn name(&self) -> &'static str {
        "Knife"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E013}"
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
    
    fn update(&self, commands: &mut Commands) {
        // Mark knife mode as active while this tool is current
        commands.insert_resource(KnifeModeActive(true));
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
        app
            .init_resource::<KnifeModeActive>()
            .add_systems(Startup, register_knife_tool)
            .add_systems(Update, reset_knife_mode_when_inactive);
    }
}

fn register_knife_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(KnifeTool));
}

/// System to deactivate knife mode when another tool is selected
pub fn reset_knife_mode_when_inactive(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
) {
    if current_tool.get_current() != Some("knife") {
        // Mark knife mode as inactive when not the current tool
        commands.insert_resource(KnifeModeActive(false));
    }
} 