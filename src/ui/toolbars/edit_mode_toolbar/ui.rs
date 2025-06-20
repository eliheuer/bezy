//! Edit Mode Toolbar UI
//!
//! This module implements the user interface for the edit mode toolbar, which dynamically
//! generates toolbar buttons based on registered tools. The system automatically discovers
//! and displays all registered tools with proper ordering and visual feedback.

use bevy::prelude::*;
use crate::ui::theme::{*, DEFAULT_FONT_PATH};
use crate::ui::toolbars::edit_mode_toolbar::*;

#[derive(Component)]
pub struct EditModeToolbarButton;

#[derive(Component)]
pub struct ToolButton {
    pub tool_id: ToolId,
}

/// Spawn the edit mode toolbar with dynamically registered tools.
///
/// This system automatically generates the toolbar UI based on all registered tools.
/// It respects tool ordering preferences and creates interactive buttons for each tool.
pub fn spawn_edit_mode_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tool_registry: ResMut<ToolRegistry>,
) {
    // Get all tools in their proper order
    let ordered_tools = tool_registry.get_ordered_tools().to_vec();
    
    info!("Spawning edit mode toolbar with {} tools: {:?}", ordered_tools.len(), ordered_tools);
    
    // Spawn a container for the edit mode toolbar buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(TOOLBAR_MARGIN),
            left: Val::Px(TOOLBAR_MARGIN),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
            margin: UiRect::all(Val::ZERO),
            row_gap: Val::Px(TOOLBAR_ROW_GAP),
            ..default()
        })
        .with_children(|parent| {
            // Create a button for each registered tool in order
            for tool_id in ordered_tools {
                if let Some(tool) = tool_registry.get_tool(tool_id) {
                    spawn_tool_button(parent, tool, &asset_server);
                }
            }
        });
}

/// Helper function to spawn a single tool button
fn spawn_tool_button(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)),
            ..default()
        })
        .with_children(|button_container| {
            button_container
                .spawn((
                    Button,
                    EditModeToolbarButton,
                    ToolButton { tool_id: tool.id() },
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        padding: UiRect::all(Val::ZERO),
                        margin: UiRect::all(Val::ZERO),
                        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(tool.icon()),
                        TextFont {
                            font: asset_server.load(DEFAULT_FONT_PATH),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(TOOLBAR_ICON_COLOR),
                    ));
                });
        });
}

/// Handle toolbar button interactions and tool switching
pub fn handle_toolbar_mode_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ToolButton,
        ),
        With<EditModeToolbarButton>,
    >,
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    for (interaction, mut background_color, mut border_color, tool_button) in
        interaction_query.iter_mut()
    {
        let is_current_tool = current_tool.get_current() == Some(tool_button.tool_id);
        
        match *interaction {
            Interaction::Pressed => {
                // Switch to this tool
                if let Some(previous_tool_id) = current_tool.get_current() {
                    if let Some(previous_tool) = tool_registry.get_tool(previous_tool_id) {
                        previous_tool.on_exit();
                    }
                }
                
                current_tool.switch_to(tool_button.tool_id);
                
                if let Some(new_tool) = tool_registry.get_tool(tool_button.tool_id) {
                    new_tool.on_enter();
                }
                
                // Visual feedback
                *background_color = BackgroundColor(PRESSED_BUTTON);
                *border_color = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
            }
            Interaction::Hovered => {
                if is_current_tool {
                    *background_color = BackgroundColor(PRESSED_BUTTON);
                    *border_color = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
                } else {
                    *background_color = BackgroundColor(HOVERED_BUTTON);
                    *border_color = BorderColor(HOVERED_BUTTON_OUTLINE_COLOR);
                }
            }
            Interaction::None => {
                if is_current_tool {
                    *background_color = BackgroundColor(PRESSED_BUTTON);
                    *border_color = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
                } else {
                    *background_color = BackgroundColor(NORMAL_BUTTON);
                    *border_color = BorderColor(NORMAL_BUTTON_OUTLINE_COLOR);
                }
            }
        }
    }
}

/// Update the current edit mode by calling the active tool's update method
pub fn update_current_edit_mode(
    mut commands: Commands,
    current_tool: Res<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    if let Some(current_tool_id) = current_tool.get_current() {
        if let Some(tool) = tool_registry.get_tool(current_tool_id) {
            tool.update(&mut commands);
        }
    }
} 