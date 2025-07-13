//! Pan tool for the edit mode toolbar
//!
//! This tool provides camera panning functionality, allowing users to navigate around the design space.
//! It integrates with the bevy_pancam system and supports temporary activation via spacebar.

use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use crate::ui::toolbars::edit_mode_toolbar::{
    EditModeSystem, EditTool, ToolRegistry,
};
use bevy::prelude::*;
use bevy_pancam::PanCam;

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
        // Note: PanCam enabling is handled by the toggle_pancam_on_mode_change system
        info!("Entered Pan tool - camera panning should be enabled");
    }

    fn on_exit(&self) {
        // Note: PanCam disabling is handled by the toggle_pancam_on_mode_change system
        info!("Exited Pan tool - camera panning should be disabled");
    }
}

// Legacy compatibility struct
pub struct PanMode;

impl EditModeSystem for PanMode {
    fn update(&self, commands: &mut Commands) {
        // Ensure select mode is disabled while in pan mode
        commands.insert_resource(SelectModeActive(false));
    }

    fn on_enter(&self) {
        // Enable panning on all PanCam components
        info!("Entering pan mode - enabling camera panning");
    }

    fn on_exit(&self) {
        // Disable panning on all PanCam components
        info!("Exiting pan mode - disabling camera panning");
    }
}

// System to enable/disable the PanCam component when entering/exiting pan mode
pub fn toggle_pancam_on_mode_change(
    mut query: Query<&mut PanCam>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Only run this system when the current tool changes
    if current_tool.is_changed() {
        let should_enable = current_tool.get_current() == Some("pan");

        for mut pancam in query.iter_mut() {
            // Only log if we're actually changing the state
            if pancam.enabled != should_enable {
                pancam.enabled = should_enable;
                if should_enable {
                    info!("PanCam enabled");
                } else {
                    info!("PanCam disabled");
                }
            }
        }
    }
}

/// Plugin for the Pan tool
pub struct PanToolPlugin;

impl Plugin for PanToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_pan_tool)
            .add_systems(Update, toggle_pancam_on_mode_change);
    }
}

fn register_pan_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PanTool));
}
