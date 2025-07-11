//! Edit Mode Toolbar UI
//!
//! This sub-module implements the user interface for the edit mode toolbar, 
//! which dynamically generates toolbar buttons based on registered tools.
//! The system automatically discovers and displays all registered tools with
//! proper ordering and visual feedback. To add a new tool, implement the
//! `EditTool` trait and register it with `ToolRegistry::register_tool()`.

use bevy::prelude::*;
use crate::ui::theme::{
    GROTESK_FONT_PATH, NORMAL_BUTTON_COLOR, HOVERED_BUTTON_COLOR, 
    PRESSED_BUTTON_COLOR, NORMAL_BUTTON_OUTLINE_COLOR, 
    HOVERED_BUTTON_OUTLINE_COLOR, PRESSED_BUTTON_OUTLINE_COLOR, 
    TOOLBAR_ICON_COLOR, PRESSED_BUTTON_ICON_COLOR,
    TOOLBAR_CONTAINER_MARGIN, TOOLBAR_PADDING, TOOLBAR_ITEM_SPACING, 
    TOOLBAR_BORDER_WIDTH, TOOLBAR_BUTTON_SIZE, BUTTON_ICON_SIZE,
};
use crate::ui::toolbars::edit_mode_toolbar::*;

// COMPONENTS ------------------------------------------------------------------

/// Component marker for toolbar buttons - used for querying toolbar entities
#[derive(Component)]
pub struct EditModeToolbarButton;

/// Component that stores the tool ID for each toolbar button
#[derive(Component)]
pub struct ToolButtonData {
    pub tool_id: ToolId,
}

// TOOLBAR CREATION ------------------------------------------------------------

/// Creates the main edit mode toolbar with all registered tools
pub fn spawn_edit_mode_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tool_registry: ResMut<ToolRegistry>,
) {
    let ordered_tool_ids = tool_registry.get_ordered_tools().to_vec();
    info!(
        "Spawning edit-mode toolbar with {} tools", ordered_tool_ids.len()
    );
    commands
        .spawn(create_toolbar_container())
        .with_children(|parent| {
            for tool_id in ordered_tool_ids {
                if let Some(tool) = tool_registry.get_tool(tool_id) {
                    create_tool_button(parent, tool, &asset_server);
                } else {
                    warn!("Tool '{}' not found in registry", tool_id);
                }
            }
        });
}

/// Creates the main toolbar container with proper positioning and styling
fn create_toolbar_container() -> impl Bundle {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN),
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::ZERO,
        ..default()
    }
}

// BUTTON CREATION -------------------------------------------------------------

/// Creates a single tool button with proper styling and components
fn create_tool_button(
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
            ToolButtonData { tool_id: tool.id() },
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
        width: Val::Px(TOOLBAR_BUTTON_SIZE),
        height: Val::Px(TOOLBAR_BUTTON_SIZE),
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
        Node {
            // TODO: This is a not great way to center the text vertically.
            // We should have a better way to center the text.
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
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
    interaction_query: Query<
        (&Interaction, &ToolButtonData),
        (Changed<Interaction>, With<EditModeToolbarButton>),
    >,
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    for (interaction, tool_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            switch_to_tool(
                tool_button.tool_id,
                &mut current_tool,
                &tool_registry,
            );
        }
    }
}

/// Updates button visual states based on interaction and current tool
pub fn update_toolbar_button_appearances(
    interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ToolButtonData,
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    mut text_query: Query<&mut TextColor>,
    children_query: Query<&Children>,
    current_tool: Res<CurrentTool>,
) {
    let current_tool_id = current_tool.get_current();
    for (
        interaction,
        mut background_color,
        mut border_color,
        tool_button,
        entity,
    ) in interaction_query
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

/// Switches to a new tool, handling lifecycle methods
fn switch_to_tool(
    new_tool_id: ToolId,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if current_tool.get_current() == Some(new_tool_id) {
        return;
    }
    exit_current_tool(current_tool, tool_registry);
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

// VISUAL UPDATES --------------------------------------------------------------

/// Updates button colors based on interaction state and current tool
fn update_button_colors(
    interaction: Interaction,
    is_current_tool: bool,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
) {
    let (bg_color, border_color_value) = match (interaction, is_current_tool) {
        (Interaction::Pressed, _) | (_, true) => {
            (PRESSED_BUTTON_COLOR, PRESSED_BUTTON_OUTLINE_COLOR)
        }
        (Interaction::Hovered, false) => {
            (HOVERED_BUTTON_COLOR, HOVERED_BUTTON_OUTLINE_COLOR)
        }
        (Interaction::None, false) => {
            (NORMAL_BUTTON_COLOR, NORMAL_BUTTON_OUTLINE_COLOR)
        }
    };
    
    *background_color = BackgroundColor(bg_color);
    *border_color = BorderColor(border_color_value);
}

/// Updates text color for button children based on current tool state
fn update_button_text_color(
    entity: Entity,
    is_current_tool: bool,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut TextColor>,
) {
    let children = match children_query.get(entity) {
        Ok(children) => children,
        Err(_) => return,
    };
    
    let new_color = if is_current_tool {
        PRESSED_BUTTON_ICON_COLOR
    } else {
        TOOLBAR_ICON_COLOR
    };
    
    // Update text colors for all children of this button
    for &child_entity in children {
        if let Ok(mut text_color) = text_query.get_mut(child_entity) {
            text_color.0 = new_color;
        }
    }
}

// ============================================================================
// TOOL UPDATES
// ============================================================================

/// Updates the current edit mode by calling the active tool's update method
///
/// This system runs every frame and calls the current tool's update method,
/// allowing tools to perform their active behavior (input handling, rendering, 
/// etc.)
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