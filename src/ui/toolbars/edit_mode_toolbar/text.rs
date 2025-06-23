//! Text Mode - Sort Placement Tool
//!
//! The text mode allows users to place sorts by clicking in the design space.
//! Sorts can be placed in two modes:
//! - Buffer mode: Sorts follow the gap buffer layout in a grid
//! - Freeform mode: Sorts are positioned freely in the design space

use crate::core::state::{TextEditorState, TextModeConfig, SortLayoutMode};
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::settings::BezySettings;
use crate::ui::panes::design_space::ViewPort;
use crate::rendering::cameras::DesignCamera;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::input::ButtonState;

pub struct TextTool;

impl EditTool for TextTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "text"
    }
    
    fn name(&self) -> &'static str {
        "Text"
    }
    
    fn icon(&self) -> &'static str {
        "\u{E017}"
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }
    
    fn default_order(&self) -> i32 {
        40 // After drawing tools, around position 5
    }
    
    fn description(&self) -> &'static str {
        "Place text and create sorts in buffer or freeform mode"
    }
    
    fn update(&self, _commands: &mut Commands) {
        // Text tool behavior is handled by dedicated systems
    }
    
    fn on_enter(&self) {
        info!("Entered Text tool - click to place sorts");
    }
    
    fn on_exit(&self) {
        info!("Exited Text tool");
    }
}

/// Resource to track if text mode is active
#[derive(Resource, Default)]
pub struct TextModeActive(pub bool);

/// Resource to track text mode state for sort placement
#[derive(Resource, Default)]
pub struct TextModeState {
    /// Current cursor position in world coordinates
    pub cursor_position: Option<Vec2>,
    /// Whether we're showing a preview
    pub showing_preview: bool,
}

/// Text placement modes for the submenu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum TextPlacementMode {
    /// Place sorts in the buffer (grid layout)
    #[default]
    Buffer,
    /// Place sorts freely in the design space
    Freeform,
}

impl TextPlacementMode {
    /// Get the icon for each placement mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            TextPlacementMode::Buffer => "\u{E021}", // Grid icon
            TextPlacementMode::Freeform => "\u{E022}", // Free placement icon
        }
    }

    /// Get the display name for each placement mode
    pub fn display_name(&self) -> &'static str {
        match self {
            TextPlacementMode::Buffer => "Buffer Mode",
            TextPlacementMode::Freeform => "Freeform Mode",
        }
    }
    
    /// Convert to SortLayoutMode
    pub fn to_sort_layout_mode(&self) -> SortLayoutMode {
        match self {
            TextPlacementMode::Buffer => SortLayoutMode::Buffer,
            TextPlacementMode::Freeform => SortLayoutMode::Freeform,
        }
    }
}

/// Component to mark text submenu buttons
#[derive(Component)]
pub struct TextSubMenuButton;

/// Component to associate a button with its placement mode
#[derive(Component)]
pub struct TextModeButton {
    pub mode: TextPlacementMode,
}

/// Resource to track the current text placement mode
#[derive(Resource, Default)]
pub struct CurrentTextPlacementMode(pub TextPlacementMode);

/// Plugin to add text mode functionality
pub struct TextModePlugin;

impl Plugin for TextModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextModeActive>()
            .init_resource::<TextModeState>()
            .init_resource::<CurrentTextPlacementMode>()
            .init_resource::<TextModeConfig>()
            .add_systems(
                Update,
                (
                    update_text_mode_active,
                    handle_text_mode_cursor,
                    handle_text_mode_clicks,
                    render_sort_preview,
                    reset_text_mode_when_inactive,
                    handle_text_mode_selection,
                    toggle_text_submenu_visibility,
                ),
            );
    }
}

/// Plugin for the Text tool
pub struct TextToolPlugin;

impl Plugin for TextToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_text_tool)
           .add_systems(PostStartup, spawn_text_submenu)
           .add_plugins(TextModePlugin);
    }
}

fn register_text_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(TextTool));
}

/// Spawn the text submenu (similar to shapes submenu)
pub fn spawn_text_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let modes = [
        TextPlacementMode::Buffer,
        TextPlacementMode::Freeform,
    ];

    // Spawn a container for the text submenu (initially hidden)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(TOOLBAR_MARGIN + 74.0), // Position below the main toolbar
                left: Val::Px(TOOLBAR_MARGIN + (40 * 4) as f32), // Position under the text tool (4th position)
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
                margin: UiRect::all(Val::ZERO),
                row_gap: Val::Px(TOOLBAR_ROW_GAP),
                display: Display::None, // Start hidden
                ..default()
            },
            Name::new("TextSubMenu"),
        ))
        .with_children(|parent| {
            // Create a button for each text placement mode
            for mode in modes {
                parent
                    .spawn(Node {
                        margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)),
                        ..default()
                    })
                    .with_children(|button_container| {
                        button_container
                            .spawn((
                                Button,
                                TextSubMenuButton,
                                TextModeButton { mode },
                                Node {
                                    width: Val::Px(48.0),
                                    height: Val::Px(48.0),
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
                                // Add the icon using the mode's icon
                                button.spawn((
                                    Text::new(mode.get_icon().to_string()),
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
        });

    info!("Spawned text submenu with {} modes", modes.len());
}

/// Handle text mode selection from the submenu
pub fn handle_text_mode_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &TextModeButton,
        ),
        With<TextSubMenuButton>,
    >,
    mut current_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
) {
    for (interaction, mut color, mut border_color, mode_button) in &mut interaction_query {
        let is_current_mode = current_mode.0 == mode_button.mode;

        // Handle button click
        if *interaction == Interaction::Pressed && !is_current_mode {
            current_mode.0 = mode_button.mode;
            text_mode_config.default_placement_mode = mode_button.mode.to_sort_layout_mode();
            info!("Switched to text placement mode: {:?}", mode_button.mode);
        }

        // Update button colors
        match (*interaction, is_current_mode) {
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
    }
}

/// Toggle the visibility of the text submenu based on current tool
pub fn toggle_text_submenu_visibility(
    mut submenu_query: Query<(&mut Node, &Name)>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_text_tool_active = current_tool.get_current() == Some("text");
    
    for (mut style, name) in submenu_query.iter_mut() {
        if name.as_str() == "TextSubMenu" {
            style.display = if is_text_tool_active {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

/// System to track when text mode is active
pub fn update_text_mode_active(
    mut text_mode_active: ResMut<TextModeActive>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_text_mode = current_tool.get_current() == Some("text");
    
    if text_mode_active.0 != is_text_mode {
        text_mode_active.0 = is_text_mode;
        debug!("Text mode active state changed: {}", is_text_mode);
    }
}

/// System to handle cursor movement in text mode
pub fn handle_text_mode_cursor(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if !text_mode_active.0 {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let cursor_moved = !cursor_moved_events.is_empty();
    cursor_moved_events.clear(); // Consume the events

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            // Apply sort-specific grid snapping
            let settings = BezySettings::default();
            let snapped_position = settings.apply_sort_grid_snap(world_position);

            // Update state
            let position_changed = text_mode_state.cursor_position != Some(snapped_position);
            text_mode_state.cursor_position = Some(snapped_position);
            text_mode_state.showing_preview = true;
            
            // Debug logging (only when position changes or cursor moved)
            if cursor_moved || position_changed {
                debug!("Text mode cursor updated: pos=({:.1}, {:.1}), showing_preview={}", 
                       snapped_position.x, snapped_position.y, text_mode_state.showing_preview);
            }
        } else {
            debug!("Failed to convert cursor position to world coordinates");
        }
    } else {
        debug!("No cursor position available");
    }
}

/// System to handle mouse clicks in text mode for sort placement
pub fn handle_text_mode_clicks(
    text_mode_active: Res<TextModeActive>,
    text_mode_state: Res<TextModeState>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    mut text_editor_state: Option<ResMut<TextEditorState>>,
    mut mouse_button_input: EventReader<bevy::input::mouse::MouseButtonInput>,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
) {
    if !text_mode_active.0 {
        return;
    }

    // Early exit if text editor state isn't ready yet
    let Some(mut text_editor_state) = text_editor_state else {
        return;
    };

    // Only handle left mouse button presses
    for event in mouse_button_input.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            if let Some(cursor_pos) = text_mode_state.cursor_position {
                // Get the current glyph name
                let glyph_name = match &glyph_navigation.current_glyph {
                    Some(name) => name.clone(),
                    None => {
                        warn!("No current glyph selected");
                        return;
                    }
                };
                
                // Get advance width for the glyph
                let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                    glyph_data.advance_width as f32
                } else {
                    600.0 // Default width
                };
                
                match current_placement_mode.0 {
                    TextPlacementMode::Buffer => {
                        // Buffer mode: Add sort to the gap buffer at cursor position
                        text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                        info!("Placed sort '{}' in buffer mode at cursor position {}", 
                              glyph_name, text_editor_state.cursor_position - 1);
                    }
                    TextPlacementMode::Freeform => {
                        // Freeform mode: Add sort at the clicked position
                        text_editor_state.add_freeform_sort(glyph_name.clone(), cursor_pos, advance_width);
                        info!("Placed sort '{}' in freeform mode at position ({:.1}, {:.1})", 
                              glyph_name, cursor_pos.x, cursor_pos.y);
                    }
                }
            }
        }
    }
}

/// Render a preview of the sort that would be placed at the cursor position
pub fn render_sort_preview(
    mut gizmos: Gizmos,
    text_mode_state: Res<TextModeState>,
    text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    viewport: Res<crate::ui::panes::design_space::ViewPort>,
) {
    // Early exit if text editor state isn't ready yet
    let Some(text_editor_state) = text_editor_state else {
        return;
    };
    
    if let Some(cursor_pos) = text_mode_state.cursor_position {
        let glyph_name = match &glyph_navigation.current_glyph {
            Some(name) => name,
            None => return, // No glyph selected, no preview
        };
        
        // Get the preview position based on placement mode
        let preview_pos = match current_placement_mode.0 {
            TextPlacementMode::Buffer => {
                // In buffer mode, show preview at the cursor position in the grid
                text_editor_state.get_world_position_for_buffer_position(text_editor_state.cursor_position)
            }
            TextPlacementMode::Freeform => {
                // In freeform mode, show preview at the mouse cursor position
                cursor_pos
            }
        };
        
        // Draw preview outline
        let preview_color = match current_placement_mode.0 {
            TextPlacementMode::Buffer => Color::srgb(0.0, 1.0, 1.0), // Cyan for buffer mode
            TextPlacementMode::Freeform => Color::srgb(1.0, 0.5, 0.0), // Orange for freeform mode
        };
        
        // Try to get glyph data for preview
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
            // Draw glyph outline preview if available
            if let Some(outline_data) = &glyph_data.outline {
                // Use the correct outline rendering function (full curves, not just points)
                crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                    &mut gizmos,
                    &viewport,
                    outline_data,
                    preview_pos,
                );
            }
            
            // Draw metrics preview (simplified since the function signature is complex)
            // TODO: Fix metrics rendering function signature
        }
        
        // Draw mode indicator
        match current_placement_mode.0 {
            TextPlacementMode::Buffer => {
                // Draw grid indicator for buffer mode
                gizmos.rect_2d(
                    preview_pos,
                    Vec2::new(100.0, 100.0),
                    Color::srgb(0.0, 1.0, 1.0).with_alpha(0.3),
                );
            }
            TextPlacementMode::Freeform => {
                // Draw circle indicator for freeform mode
                gizmos.circle_2d(
                    preview_pos + Vec2::new(0.0, 60.0),
                    12.0,
                    Color::srgb(1.0, 0.5, 0.0).with_alpha(0.7),
                );
            }
        }
    }
}

pub fn reset_text_mode_when_inactive(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
) {
    if !text_mode_active.0 {
        text_mode_state.cursor_position = None;
        text_mode_state.showing_preview = false;
    }
} 