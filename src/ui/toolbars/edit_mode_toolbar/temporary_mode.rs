//! Temporary Mode Switching
//!
//! This module handles temporary mode switching using the spacebar key.
//! When spacebar is held down, the mode switches to Pan mode. When released,
//! it switches back to the previous mode. This provides a better UX than
//! the hybrid approach.

use super::{CurrentTool, ToolRegistry, ToolId};
use crate::ui::toolbars::edit_mode_toolbar::text::{
    TextPlacementMode, CurrentTextPlacementMode
};
use bevy::prelude::*;

/// Resource to track temporary mode switching state
#[derive(Resource, Default)]
pub struct TemporaryModeState {
    /// The tool that was active before switching to temporary mode
    pub previous_tool: Option<ToolId>,
    /// Whether we're currently in temporary mode
    pub in_temporary_mode: bool,
}

/// System to handle spacebar for temporary Pan mode switching
///
/// This provides a clean UX where:
/// - Hold spacebar: Switch to Pan mode (shows in toolbar)
/// - Release spacebar: Switch back to previous mode
///
/// This works with the new dynamic tool system by switching between
/// registered tools in the ToolRegistry.
///
/// **Special handling for text tool**: When the text tool is active and 
/// in insert mode, spacebar is used for typing spaces and temporary 
/// mode switching is disabled.
pub fn handle_temporary_mode_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_tool: ResMut<CurrentTool>,
    mut temp_state: ResMut<TemporaryModeState>,
    tool_registry: Res<ToolRegistry>,
    current_text_placement_mode: Option<Res<CurrentTextPlacementMode>>,
) {
    // Check if text tool is active and in insert mode
    let is_text_insert_mode = current_tool.get_current() == Some("text") 
        && current_text_placement_mode
            .as_ref()
            .map(|mode| mode.0 == TextPlacementMode::Insert)
            .unwrap_or(false);

    // Skip temporary mode switching if text tool is in insert mode
    // This allows spacebar to be used for typing spaces
    if is_text_insert_mode {
        return;
    }

    handle_spacebar_press(
        &keyboard,
        &mut current_tool,
        &mut temp_state,
        &tool_registry,
    );

    handle_spacebar_release(
        &keyboard,
        &mut current_tool,
        &mut temp_state,
        &tool_registry,
    );
}

/// Handle spacebar press - switch to temporary Pan mode
fn handle_spacebar_press(
    keyboard: &Res<ButtonInput<KeyCode>>,
    current_tool: &mut ResMut<CurrentTool>,
    temp_state: &mut ResMut<TemporaryModeState>,
    tool_registry: &Res<ToolRegistry>,
) {
    if !keyboard.just_pressed(KeyCode::Space) || temp_state.in_temporary_mode {
        return;
    }

    // Check if pan tool is available
    let Some(pan_tool) = tool_registry.get_tool("pan") else {
        warn!("Pan tool not found in registry for temporary mode switching");
        return;
    };

    // Store the current tool and switch to Pan
    temp_state.previous_tool = current_tool.get_current();
    temp_state.in_temporary_mode = true;

    // Exit the current tool if any
    exit_current_tool(current_tool, tool_registry);

    // Switch to Pan tool
    current_tool.switch_to("pan");
    pan_tool.on_enter();

    info!("Temporarily switched to Pan tool (spacebar held)");
}

/// Handle spacebar release - switch back to previous tool
fn handle_spacebar_release(
    keyboard: &Res<ButtonInput<KeyCode>>,
    current_tool: &mut ResMut<CurrentTool>,
    temp_state: &mut ResMut<TemporaryModeState>,
    tool_registry: &Res<ToolRegistry>,
) {
    if !keyboard.just_released(KeyCode::Space) || !temp_state.in_temporary_mode {
        return;
    }

    // Exit Pan tool
    if let Some(pan_tool) = tool_registry.get_tool("pan") {
        pan_tool.on_exit();
    }

    // Switch back to the previous tool
    switch_to_previous_tool(current_tool, temp_state, tool_registry);

    // Reset temporary mode state
    temp_state.previous_tool = None;
    temp_state.in_temporary_mode = false;
}

/// Exit the currently active tool
fn exit_current_tool(
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if let Some(current_tool_id) = current_tool.get_current() {
        if let Some(current_tool_impl) = tool_registry.get_tool(current_tool_id) {
            current_tool_impl.on_exit();
        }
    }
}

/// Switch back to the previous tool or default to select
fn switch_to_previous_tool(
    current_tool: &mut ResMut<CurrentTool>,
    temp_state: &TemporaryModeState,
    tool_registry: &Res<ToolRegistry>,
) {
    if let Some(previous_tool_id) = temp_state.previous_tool {
        if let Some(previous_tool) = tool_registry.get_tool(previous_tool_id) {
            current_tool.switch_to(previous_tool_id);
            previous_tool.on_enter();

            info!(
                "Switched back to {} tool (spacebar released)",
                previous_tool.name()
            );
            return;
        }
    }

    // If no previous tool, default to select
    if let Some(select_tool) = tool_registry.get_tool("select") {
        current_tool.switch_to("select");
        select_tool.on_enter();
        info!("Switched back to Select tool (no previous tool)");
    }
} 