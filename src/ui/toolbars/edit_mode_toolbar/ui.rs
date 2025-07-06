//! Edit Mode Toolbar UI
//!
//! This module implements the user interface for the edit mode toolbar, which 
//! dynamically generates toolbar buttons based on registered tools. The system 
//! automatically discovers and displays all registered tools with proper 
//! ordering and visual feedback. To add a new tool, implement the `EditTool` 
//! trait and register it with `ToolRegistry::register_tool()`.

use bevy::prelude::*;
use crate::ui::theme::{
    GROTESK_FONT_PATH, NORMAL_BUTTON_COLOR, HOVERED_BUTTON_COLOR, PRESSED_BUTTON_COLOR,
    NORMAL_BUTTON_OUTLINE_COLOR, HOVERED_BUTTON_OUTLINE_COLOR, 
    PRESSED_BUTTON_OUTLINE_COLOR, TOOLBAR_ICON_COLOR, PRESSED_BUTTON_ICON_COLOR,
    TOOLBAR_CONTAINER_MARGIN, TOOLBAR_PADDING, TOOLBAR_ITEM_SPACING, TOOLBAR_BORDER_WIDTH,
    BUTTON_SIZE, BUTTON_ICON_SIZE,
};
use crate::ui::toolbars::edit_mode_toolbar::*;

// COMPONENTS -----------------------------------------------------------------

/// Component marker for toolbar buttons - used for querying toolbar entities
#[derive(Component)]
pub struct EditModeToolbarButton;

/// Component that stores the tool ID for each toolbar button
#[derive(Component)]
pub struct ToolButton {
    pub tool_id: ToolId,
}

// TOOLBAR CREATION ------------------------------------------------------------

/// Creates the main edit mode toolbar with all registered tools
pub fn spawn_edit_mode_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tool_registry: ResMut<ToolRegistry>,
) {
    let ordered_tools = get_ordered_tools(&mut tool_registry);
    info!(
        "Spawning edit-mode toolbar with {} tools: {:?}", 
        ordered_tools.len(), 
        ordered_tools
    );
    let toolbar_entity = create_toolbar_container(&mut commands);
    add_toolbar_buttons(
        &mut commands,
        toolbar_entity,
        &ordered_tools,
        &tool_registry,
        &asset_server
    );
}

/// Gets all tools in their proper display order
fn get_ordered_tools(tool_registry: &mut ToolRegistry) -> Vec<ToolId> {
    tool_registry.get_ordered_tools().to_vec()
}

/// Creates the main toolbar container with proper positioning and styling
fn create_toolbar_container(commands: &mut Commands) -> Entity {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(TOOLBAR_CONTAINER_MARGIN),
            left: Val::Px(TOOLBAR_CONTAINER_MARGIN),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
            margin: UiRect::all(Val::ZERO),
            row_gap: Val::ZERO,
            ..default()
        })
        .id() // Extract entity ID for return
}

/// Adds buttons for each registered tool to the toolbar
fn add_toolbar_buttons(
    commands: &mut Commands,
    toolbar_entity: Entity,
    ordered_tools: &[ToolId],
    tool_registry: &ToolRegistry,
    asset_server: &AssetServer,
) {
    commands.entity(toolbar_entity).with_children(|parent| {
        for tool_id in ordered_tools {
            if let Some(tool) = tool_registry.get_tool(tool_id) {
                create_tool_button(parent, tool, asset_server);
            } else {
                warn!("Tool '{}' not found in registry", tool_id);
            }
        }
    });
}

// BUTTON CREATION -------------------------------------------------------------

/// Creates a single tool button with proper styling and components
fn create_tool_button(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
) {
    // Create container with spacing
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)),
            ..default()
        })
        .with_children(|button_container| {
            create_button_entity(button_container, tool, asset_server);
        });
}

/// Creates the button entity with all required components
fn create_button_entity(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
) {
    parent
        .spawn((
            Button,
            EditModeToolbarButton,
            ToolButton { tool_id: tool.id() },
            create_button_styling(),
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
        ))
        .with_children(|button| {
            create_button_text(button, tool, asset_server);
        });
}

/// Creates the button styling configuration
fn create_button_styling() -> Node {
    Node {
        width: Val::Px(BUTTON_SIZE),
        height: Val::Px(BUTTON_SIZE),
        padding: UiRect::all(Val::ZERO),
        margin: UiRect::all(Val::ZERO),
        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates the button text with the tool's icon
fn create_button_text(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
) {
    parent.spawn((
        Text::new(tool.icon()),
        TextFont {
            font: asset_server.load(GROTESK_FONT_PATH),
            font_size: BUTTON_ICON_SIZE,
            ..default()
        },
        TextColor(TOOLBAR_ICON_COLOR),
    ));
}

// INTERACTION HANDLING --------------------------------------------------------

/// Handles toolbar button interactions and tool switching
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
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    handle_button_clicks(
        &mut interaction_query,
        &mut current_tool,
        &tool_registry,
    );
}

/// Updates button visual states
pub fn update_toolbar_button_appearances(
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
    current_tool: Res<CurrentTool>,
) {
    let current_tool_id = current_tool.get_current();
    for (interaction, mut background_color, mut border_color, tool_button, entity) in 
        interaction_query 
    {
        let is_current_tool = current_tool_id == Some(tool_button.tool_id);
        update_button_colors(
            *interaction,
            is_current_tool,
            &mut background_color,
            &mut border_color,
        );
        update_button_text_color(
            entity,
            is_current_tool,
            &children_query,
            &mut text_query,
        );
    }
}

/// Handles button click events and tool switching
fn handle_button_clicks(
    interaction_query: &mut Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ToolButton,
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    for (interaction, _bg_color, _border_color, tool_button, _entity) in 
        interaction_query 
    {
        if *interaction == Interaction::Pressed {
            switch_to_tool(tool_button.tool_id, current_tool, tool_registry);
        }
    }
}

/// Switches to a new tool, handling lifecycle methods
fn switch_to_tool(
    new_tool_id: ToolId,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    // Early return if tool is already active
    if current_tool.get_current() == Some(new_tool_id) {
        return;
    }
    // Exit current tool if any
    exit_current_tool(current_tool, tool_registry);
    // Enter new tool
    enter_new_tool(new_tool_id, current_tool, tool_registry);
}

/// Exits the currently active tool
fn exit_current_tool(
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if let Some(current_id) = current_tool.get_current() {
        if let Some(current_tool_impl) = tool_registry.get_tool(current_id) {
            current_tool_impl.on_exit();
        }
    }
}

/// Enters a new tool and updates the current tool state
fn enter_new_tool(
    new_tool_id: ToolId,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if let Some(new_tool_impl) = tool_registry.get_tool(new_tool_id) {
        new_tool_impl.on_enter();
    }
    
    current_tool.switch_to(new_tool_id);
    info!("Switched to tool: {}", new_tool_id);
}

/// Updates button visual states based on interaction and current tool
fn update_button_appearances(
    interaction_query: &mut Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ToolButton,
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut TextColor>,
    current_tool: &ResMut<CurrentTool>,
) {
    // Cache current tool ID to avoid repeated lookups
    let current_tool_id = current_tool.get_current();
    
    for (interaction, mut background_color, mut border_color, tool_button, entity) in 
        interaction_query 
    {
        let is_current_tool = current_tool_id == Some(tool_button.tool_id);
        
        // Update button colors (background and border)
        update_button_colors(
            *interaction,
            is_current_tool,
            &mut background_color,
            &mut border_color,
        );
        
        // Update button text color
        update_button_text_color(
            entity,
            is_current_tool,
            children_query,
            text_query,
        );
    }
}

// VISUAL UPDATES --------------------------------------------------------------

/// Updates button colors based on interaction state and current tool
fn update_button_colors(
    interaction: Interaction,
    is_current_tool: bool,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
) {
    match (interaction, is_current_tool) {
        (Interaction::Pressed, _) | (_, true) => {
            *background_color = BackgroundColor(PRESSED_BUTTON_COLOR);
            *border_color = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
        }
        (Interaction::Hovered, false) => {
            *background_color = BackgroundColor(HOVERED_BUTTON_COLOR);
            *border_color = BorderColor(HOVERED_BUTTON_OUTLINE_COLOR);
        }
        (Interaction::None, false) => {
            *background_color = BackgroundColor(NORMAL_BUTTON_COLOR);
            *border_color = BorderColor(NORMAL_BUTTON_OUTLINE_COLOR);
        }
    }
}

/// Updates text color for button children based on current tool state
fn update_button_text_color(
    entity: Entity,
    is_current_tool: bool,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut TextColor>,
) {
    // Early return if no children found
    let children = match children_query.get(entity) {
        Ok(children) => children,
        Err(_) => return,
    };
    
    // Calculate color once
    let new_color = if is_current_tool {
        PRESSED_BUTTON_ICON_COLOR
    } else {
        TOOLBAR_ICON_COLOR
    };
    
    // Batch update all text colors for this button's children
    for &child_entity in children {
        if let Ok(mut text_color) = text_query.get_mut(child_entity) {
            text_color.0 = new_color;
        }
    }
}

// TOOL UPDATES ---------------------------------------------------------------

/// Updates the current edit mode by calling the active tool's update method
///
/// This system runs every frame and calls the current tool's update method,
/// allowing tools to perform their active behavior (input handling, rendering, etc.)
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