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
                            font_size: 48.0,
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
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    mut text_query: Query<&mut TextColor>,
    children_query: Query<&Children>,
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    // First handle any new interactions
    for (interaction, _color, _border_color, tool_button, _entity) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let new_tool_id = tool_button.tool_id;

            // Only process if the tool is actually changing
            if current_tool.get_current() != Some(new_tool_id) {
                // Call on_exit for the current tool
                if let Some(current_id) = current_tool.get_current() {
                    if let Some(current_tool_impl) = tool_registry.get_tool(current_id) {
                        current_tool_impl.on_exit();
                    }
                }

                // Call on_enter for the new tool
                if let Some(new_tool_impl) = tool_registry.get_tool(new_tool_id) {
                    new_tool_impl.on_enter();
                }

                // Save the new tool
                current_tool.switch_to(new_tool_id);

                // Log only when tool actually changes
                info!("Switched to tool: {}", new_tool_id);
            }
        }
    }

    // Then update all button appearances based on the current tool
    for (interaction, mut background_color, mut border_color, tool_button, entity) in &mut interaction_query {
        let is_current_tool = current_tool.get_current() == Some(tool_button.tool_id);
        
        // Update button colors
        match (*interaction, is_current_tool) {
            (Interaction::Pressed, _) | (_, true) => {
                *background_color = BackgroundColor(PRESSED_BUTTON);
                *border_color = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
            }
            (Interaction::Hovered, false) => {
                *background_color = BackgroundColor(HOVERED_BUTTON);
                *border_color = BorderColor(HOVERED_BUTTON_OUTLINE_COLOR);
            }
            (Interaction::None, false) => {
                *background_color = BackgroundColor(NORMAL_BUTTON);
                *border_color = BorderColor(NORMAL_BUTTON_OUTLINE_COLOR);
            }
        }

        // Update text color for this button's children
        if let Ok(children) = children_query.get(entity) {
            for child in children {
                if let Ok(mut text_color) = text_query.get_mut(*child) {
                    text_color.0 = if is_current_tool {
                        PRESSED_BUTTON_ICON_COLOR
                    } else {
                        TOOLBAR_ICON_COLOR
                    };
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