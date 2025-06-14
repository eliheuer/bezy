//! Edit Mode Toolbar UI
//!
//! This module implements the user interface for the edit mode toolbar, which dynamically
//! generates toolbar buttons based on registered tools. The system automatically discovers
//! and displays all registered tools with proper ordering and visual feedback.
//!
//! ## Overview
//!
//! The toolbar consists of:
//! - A horizontal row of buttons, dynamically generated from registered tools
//! - Visual feedback for the currently active tool (highlighted button)
//! - Hover and press states for better user interaction
//! - Icon-based representation using tool-defined Unicode characters
//!
//! ## Dynamic System
//!
//! Unlike the previous hardcoded approach, this system:
//! - **Automatically discovers tools**: No need to update UI code when adding tools
//! - **Respects tool ordering**: Uses `default_order()` and `ToolOrdering` resource
//! - **Zero-maintenance**: Adding tools requires no changes to existing code
//! - **Proper lifecycle**: Automatic setup/cleanup when switching between tools
//! - **Extensible**: Support for shortcuts, descriptions, temporary modes, etc.
//!
//! ## Adding Tools
//!
//! The system makes adding tools incredibly simple:
//!
//! 1. **Create tool**: Implement `EditTool` trait in a new file
//! 2. **Register plugin**: Add your tool's plugin to the app
//! 3. **Done**: Tool appears automatically with proper behavior
//!
//! No need to modify enums, match statements, or UI generation code.
//!
//! ## Architecture
//!
//! Each tool implements the `EditTool` trait, providing:
//! - `id()`: Unique identifier for the tool
//! - `name()`: Display name for UI
//! - `icon()`: Unicode icon character
//! - `update()`: Called every frame while active
//! - `on_enter()`: Called when switching to this tool
//! - `on_exit()`: Called when switching away from this tool
//!
//! The UI automatically handles tool transitions and ensures proper cleanup
//! when switching between tools.

use crate::ui::theme::*;
use crate::ui::toolbars::edit_mode_toolbar::*;

#[derive(Component)]
pub struct EditModeToolbarButton;

#[derive(Component)]
pub struct ToolButton {
    pub tool_id: ToolId,
}

// Legacy types for backward compatibility during migration
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EditMode {
    #[default]
    Select,
    Pen,
    Hyper,
    Knife,
    Pan,
    Measure,
    Shapes,
    Text,
}

impl EditMode {
    pub fn get_system(&self) -> Box<dyn EditModeSystem> {
        match self {
            EditMode::Select => Box::new(SelectMode),
            EditMode::Pen => Box::new(PenMode),
            EditMode::Hyper => Box::new(HyperMode),
            EditMode::Knife => Box::new(KnifeMode),
            EditMode::Pan => Box::new(PanMode),
            EditMode::Measure => Box::new(MeasureMode),
            EditMode::Shapes => Box::new(ShapesMode),
            EditMode::Text => Box::new(TextMode),
        }
    }

    pub fn get_icon(&self) -> &'static str {
        match self {
            EditMode::Select => "\u{E010}",
            EditMode::Pen => "\u{E011}",
            EditMode::Hyper => "\u{E012}",
            EditMode::Knife => "\u{E013}",
            EditMode::Pan => "\u{E014}",
            EditMode::Measure => "\u{E015}",
            EditMode::Shapes => "\u{E016}",
            EditMode::Text => "\u{E017}",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EditMode::Select => "Select",
            EditMode::Pen => "Pen",
            EditMode::Hyper => "Hyper",
            EditMode::Knife => "Knife",
            EditMode::Pan => "Pan",
            EditMode::Measure => "Measure",
            EditMode::Shapes => "Shapes",
            EditMode::Text => "Text",
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentEditMode(pub EditMode);

/// Spawn the edit mode toolbar with dynamically registered tools.
///
/// This system automatically generates the toolbar UI based on all registered tools.
/// It respects tool ordering preferences and creates interactive buttons for each tool.
///
/// The toolbar is positioned at the top-left of the screen and displays tools in a
/// horizontal row. Each tool gets a button with its icon and proper interaction states.
///
/// # System Requirements
///
/// This system requires:
/// - `ToolRegistry`: Must contain registered tools
/// - `AssetServer`: For loading fonts and icons
/// - `ToolOrdering`: For custom tool ordering (optional)
///
/// # Automatic Features
///
/// - **Dynamic generation**: Toolbar updates automatically when tools are added
/// - **Proper ordering**: Tools appear in the correct order based on preferences
/// - **Visual feedback**: Buttons show hover, press, and active states
/// - **Icon support**: Uses Unicode icons defined by each tool
pub fn spawn_edit_mode_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tool_registry: ResMut<ToolRegistry>,
    tool_ordering: Res<ToolOrdering>,
) {
    // Apply custom ordering if configured
    tool_registry.apply_custom_ordering(&tool_ordering);
    
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

    // Also spawn the shapes sub-menu (it will start hidden)
    // TODO: Make this dynamic too - tools should be able to register their own submenus
            crate::ui::toolbars::edit_mode_toolbar::spawn_shapes_submenu(
        &mut commands,
        &asset_server,
    );
}

/// Helper function to spawn a single tool button
fn spawn_tool_button(
    parent: &mut ChildBuilder,
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
                        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(TOOLBAR_BORDER_COLOR),
                    BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),
                    BackgroundColor(TOOLBAR_BACKGROUND_COLOR),
                ))
                .with_children(|button| {
                    // Add the icon using the tool's icon
                    button.spawn((
                        Text::new(tool.icon().to_string()),
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

/// Handle toolbar tool selection with the new dynamic system.
///
/// This system manages all toolbar button interactions and tool switching logic.
/// It handles clicking, visual feedback, and proper tool lifecycle management.
///
/// # Behavior
///
/// - **Click handling**: Detects button clicks and switches to the selected tool
/// - **Visual feedback**: Updates button colors based on interaction state
/// - **Tool lifecycle**: Calls `on_exit()` on old tool and `on_enter()` on new tool
/// - **State management**: Updates the `CurrentTool` resource
/// - **Performance**: Only processes actual tool changes, ignoring redundant clicks
///
/// # Visual States
///
/// - **Normal**: Default button appearance
/// - **Hovered**: Highlighted when mouse is over button
/// - **Pressed**: Temporarily highlighted when clicked
/// - **Active**: Persistently highlighted for the current tool
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
    mut text_query: Query<(&Parent, &mut TextColor)>,
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
    for (interaction, mut color, mut border_color, tool_button, entity) in &mut interaction_query {
        let is_current_tool = current_tool.get_current() == Some(tool_button.tool_id);

        // Update button colors
        match (*interaction, is_current_tool) {
            (Interaction::Pressed, _) | (_, true) => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::Hovered, false) => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::None, false) => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }

        // Update text color for this button
        for (parent, mut text_color) in &mut text_query {
            if parent.get() == entity {
                text_color.0 = if is_current_tool {
                    PRESSED_BUTTON_ICON_COLOR
                } else {
                    TOOLBAR_ICON_COLOR
                };
            }
        }
    }
}

/// Update the current tool's behavior
pub fn update_current_edit_mode(
    mut commands: Commands,
    current_tool: Res<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    if let Some(tool_id) = current_tool.get_current() {
        if let Some(tool) = tool_registry.get_tool(tool_id) {
            tool.update(&mut commands);
        }
    }
}

// Legacy compatibility functions (will be removed after migration)
fn parse_edit_mode_from_button_name(button_name: &str) -> EditMode {
    match button_name {
        "Select" => EditMode::Select,
        "Pen" => EditMode::Pen,
        "Hyper" => EditMode::Hyper,
        "Knife" => EditMode::Knife,
        "Pan" => EditMode::Pan,
        "Measure" => EditMode::Measure,
        "Shapes" => EditMode::Shapes,
        "Text" => EditMode::Text,
        _ => {
            warn!(
                "Unknown edit mode button: {}, defaulting to Select",
                button_name
            );
            EditMode::Select
        }
    }
}

