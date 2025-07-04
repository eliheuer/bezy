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
use std::sync::atomic::{AtomicU64, Ordering};

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
        info!("Entered Text tool - Enhanced features:");
        info!("• Click to place sorts, type letters to add glyphs");
        info!("• Tab to switch Buffer/Freeform modes");
        info!("• 1-9 keys to switch glyphs, F1 for help");
        info!("• Arrow keys for navigation, Ctrl+S to show buffer");
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
            TextPlacementMode::Buffer => "\u{E004}", // Buffer sorts icon
            TextPlacementMode::Freeform => "\u{E006}", // Freeform sorts icon
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
                    handle_text_mode_keyboard,
                    handle_text_tool_shortcuts,
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
                left: Val::Px(TOOLBAR_MARGIN), // Align to the left like main toolbar
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
                margin: UiRect::all(Val::ZERO),
                row_gap: Val::Px(TOOLBAR_PADDING), // Use 8.0 pixel spacing like horizontal spacing
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
                                Node {
                                    width: Val::Px(64.0), // Same size as main toolbar buttons
                                    height: Val::Px(64.0), // Same size as main toolbar buttons
                                    padding: UiRect::all(Val::ZERO),
                                    border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),
                                BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
                                BackgroundColor(NORMAL_BUTTON),
                                TextSubMenuButton,
                                TextModeButton { mode },
                            ))
                            .with_children(|button| {
                                // Add the icon using the mode's icon
                                button.spawn((
                                    Text::new(mode.get_icon().to_string()),
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

/// System to handle mouse cursor tracking in text mode
/// 
/// The cursor position is tracked for two different purposes:
/// 1. Snapped position - used for sort placement and metrics preview (grid-aligned)
/// 2. Raw position - used for the handle preview (exact mouse tracking)
/// 
/// This dual-tracking approach ensures:
/// - Sort placement happens on a predictable grid
/// - Handle preview follows the mouse pointer exactly
/// - Visual feedback is clear and responsive
pub fn handle_text_mode_cursor(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    viewport: Res<crate::ui::panes::design_space::ViewPort>,
) {
    if !text_mode_active.0 {
        return;
    }

    // Don't update cursor position when hovering over UI
    if ui_hover_state.is_hovering_ui {
        text_mode_state.showing_preview = false;
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
            // Apply sort-specific grid snapping for placement position
            // This ensures sorts are placed on a predictable grid for alignment
            let settings = BezySettings::default();
            let snapped_position = settings.apply_sort_grid_snap(world_position);

            // Update state with snapped position for sort placement
            let position_changed = text_mode_state.cursor_position != Some(snapped_position);
            text_mode_state.cursor_position = Some(snapped_position);
            text_mode_state.showing_preview = true;
            
            // Debug logging (only when position changes or cursor moved)
            if cursor_moved || position_changed {
                debug!("Text mode cursor updated: snapped=({:.1}, {:.1}), raw=({:.1}, {:.1})", 
                       snapped_position.x, snapped_position.y, world_position.x, world_position.y);
            }
        } else {
            debug!("Failed to convert cursor position to world coordinates");
        }
    } else {
        debug!("No cursor position available");
    }

    // Get the actual mouse cursor position in world coordinates (unsnapped)
    let raw_cursor_world_pos = {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.get_single(), camera_query.get_single()) {
            if let Some(cursor_position) = window.cursor_position() {
                let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_position).ok();
                // Debug logging
                if let Some(pos) = world_pos {
                    info!("Raw cursor: screen=({:.1}, {:.1}) -> world=({:.1}, {:.1})", 
                           cursor_position.x, cursor_position.y, pos.x, pos.y);
                }
                world_pos
            } else {
                None
            }
        } else {
            None
        }
    };
    
    if let (Some(cursor_pos), Some(raw_cursor_world_pos)) = (text_mode_state.cursor_position, raw_cursor_world_pos) {
        // For preview mode, the handle should always be directly under the mouse cursor in screen space
        // Use screen coordinates directly for the handle
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
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    viewport: Res<crate::ui::panes::design_space::ViewPort>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if !text_mode_active.0 {
        return;
    }

    // Don't handle clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Early exit if text editor state isn't ready yet
    let Some(mut text_editor_state) = text_editor_state else {
        return;
    };

    // Get the actual mouse cursor position in world coordinates (same as preview)
    let raw_cursor_world_pos = {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.get_single(), camera_query.get_single()) {
            if let Some(cursor_position) = window.cursor_position() {
                camera.viewport_to_world_2d(camera_transform, cursor_position).ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    // Only handle left mouse button presses
    for event in mouse_button_input.read() {
        if event.button == MouseButton::Left && event.state == ButtonState::Pressed {
            if let Some(raw_cursor_world_pos) = raw_cursor_world_pos {
                // Get the current glyph name, with fallback to 'a' or first available glyph
                let glyph_name = match &glyph_navigation.current_glyph {
                    Some(name) => name.clone(),
                    None => {
                        // Try to use 'a' as default, or first available glyph
                        if app_state.workspace.font.glyphs.contains_key("a") {
                            "a".to_string()
                        } else if let Some(first_glyph) = app_state.workspace.font.glyphs.keys().next() {
                            first_glyph.clone()
                        } else {
                            warn!("No glyphs available in font");
                            return;
                        }
                    }
                };
                
                // Get advance width for the glyph
                let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                    glyph_data.advance_width as f32
                } else {
                    600.0 // Default width
                };
                
                // Position calculation must match preview exactly
                let descender = app_state.workspace.info.metrics.descender.unwrap() as f32;
                let cursor_design_pos = viewport.from_screen(raw_cursor_world_pos);
                // Use the exact same calculation as preview: cursor - descender
                let raw_sort_position = Vec2::new(cursor_design_pos.x, cursor_design_pos.y) - Vec2::new(0.0, descender);
                
                // Apply grid snapping to the final sort position  
                let settings = crate::core::settings::BezySettings::default();
                let sort_position = settings.apply_sort_grid_snap(raw_sort_position);
                
                match current_placement_mode.0 {
                    TextPlacementMode::Buffer => {
                        // Buffer mode: Create buffer sort at the calculated position
                        text_editor_state.create_buffer_sort_at_position(glyph_name.clone(), sort_position, advance_width);
                        info!("Placed sort '{}' in buffer mode at position ({:.1}, {:.1}) with descender offset {:.1}", 
                              glyph_name, sort_position.x, sort_position.y, descender);
                    }
                    TextPlacementMode::Freeform => {
                        // Freeform mode: Add sort at the calculated position
                        text_editor_state.add_freeform_sort(glyph_name.clone(), sort_position, advance_width);
                        info!("Placed sort '{}' in freeform mode at position ({:.1}, {:.1}) with descender offset {:.1}", 
                              glyph_name, sort_position.x, sort_position.y, descender);
                    }
                }
            }
        }
    }
}

/// Render a preview of the sort that would be placed at the cursor position
pub fn render_sort_preview(
    mut gizmos: Gizmos,
    text_mode_active: Res<TextModeActive>,
    text_mode_state: Res<TextModeState>,
    text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    viewport: Res<crate::ui::panes::design_space::ViewPort>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    // Only show preview when text mode is active
    if !text_mode_active.0 {
        return;
    }

    // Don't show preview when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Early exit if text editor state isn't ready yet
    let Some(_text_editor_state) = text_editor_state else {
        return;
    };
    
    // Get the actual mouse cursor position in world coordinates (unsnapped)
    let raw_cursor_world_pos = {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.get_single(), camera_query.get_single()) {
            if let Some(cursor_position) = window.cursor_position() {
                let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_position).ok();
                // Debug logging
                if let Some(pos) = world_pos {
                    info!("Raw cursor: screen=({:.1}, {:.1}) -> world=({:.1}, {:.1})", 
                           cursor_position.x, cursor_position.y, pos.x, pos.y);
                }
                world_pos
            } else {
                None
            }
        } else {
            None
        }
    };
    
    if let (Some(cursor_pos), Some(raw_cursor_world_pos)) = (text_mode_state.cursor_position, raw_cursor_world_pos) {
        let glyph_name = match &glyph_navigation.current_glyph {
            Some(name) => name.clone(),
            None => {
                // Try to use 'a' as default, or first available glyph
                if app_state.workspace.font.glyphs.contains_key("a") {
                    "a".to_string()
                } else if let Some(first_glyph) = app_state.workspace.font.glyphs.keys().next() {
                    first_glyph.clone()
                } else {
                    return; // No glyphs available, no preview
                }
            }
        };
        
        // Preview position calculation:
        // The handle should be directly under the cursor (at descender line)
        // So the sort baseline should be positioned so that baseline + descender = cursor
        // Therefore: baseline = cursor - descender
        let descender = app_state.workspace.info.metrics.descender.unwrap() as f32;
        let cursor_design_pos = viewport.from_screen(raw_cursor_world_pos);
        let raw_preview_pos = Vec2::new(cursor_design_pos.x, cursor_design_pos.y) - Vec2::new(0.0, descender);
        
        // Apply grid snapping to preview position to match final placement
        let settings = crate::core::settings::BezySettings::default();
        let preview_pos = settings.apply_sort_grid_snap(raw_preview_pos);
        
        info!("Preview positioning: cursor=({:.1}, {:.1}), descender={:.1}, preview_pos=({:.1}, {:.1})", 
              raw_cursor_world_pos.x, raw_cursor_world_pos.y, descender, preview_pos.x, preview_pos.y);
        
        // Use orange color for active preview (consistent with active sorts)
        let preview_color = Color::srgb(1.0, 0.5, 0.0).with_alpha(0.8); // Orange for active
        
        // Try to get glyph data for preview
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
            // Draw glyph outline preview if available
            if let Some(outline_data) = &glyph_data.outline {
                crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                    &mut gizmos,
                    &viewport,
                    outline_data,
                    preview_pos,
                );
            }
            
            // Draw metrics preview in orange
            let norad_glyph = glyph_data.to_norad_glyph();
            crate::rendering::metrics::draw_metrics_at_position_with_color(
                &mut gizmos,
                &viewport,
                &norad_glyph,
                &app_state.workspace.info.metrics,
                preview_pos,
                preview_color,
            );
            
            // The handle should be at the descender line relative to the snapped baseline position
            // preview_pos is the snapped baseline, so handle = baseline + descender
            let handle_position = preview_pos + Vec2::new(0.0, descender);
            
            // Log every 60 frames (roughly once per second at 60 FPS)
            static FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
            let frame = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
            if frame % 60 == 0 {
                info!("Preview handle: cursor=({:.1}, {:.1}), handle=({:.1}, {:.1}), preview_pos=({:.1}, {:.1})", 
                       raw_cursor_world_pos.x, raw_cursor_world_pos.y, handle_position.x, handle_position.y, 
                       preview_pos.x, preview_pos.y);
            }
            
            // Draw handle for buffer root (larger size with green color when placing buffer sorts)
            let (outer_color, inner_color, handle_size) = match current_placement_mode.0 {
                TextPlacementMode::Buffer => {
                    // Buffer root handles are green and larger
                    (Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.6, 1.0, 0.6), 28.0)
                }
                TextPlacementMode::Freeform => {
                    // Freeform handles are orange and normal size
                    (Color::srgb(1.0, 0.5, 0.0), Color::srgb(1.0, 0.8, 0.4), 20.0)
                }
            };
            
            // Convert handle position to screen space (same coordinate system as metrics)
            let handle_screen_pos = viewport.to_screen(
                crate::ui::panes::design_space::DPoint::from((handle_position.x, handle_position.y))
            );
            
            // Draw the main handle circle in screen space
            gizmos.circle_2d(
                handle_screen_pos,
                handle_size,
                outer_color,
            );
            
            // Draw the inner circle for visual clarity
            gizmos.circle_2d(
                handle_screen_pos,
                handle_size * 0.6,
                inner_color,
            );
            
            // FIXED: Draw buffer root indicator (small square) for buffer mode
            // Make it much smaller and more subtle to avoid visual clutter
            if current_placement_mode.0 == TextPlacementMode::Buffer {
                gizmos.rect_2d(
                    handle_screen_pos,
                    Vec2::new(4.0, 4.0), // Small white square indicator
                    Color::srgb(1.0, 1.0, 1.0).with_alpha(0.8), // Semi-transparent white square
                );
            }
        }
    }
}

/// System to handle keyboard input in text mode for buffer navigation and quick sort placement
pub fn handle_text_mode_keyboard(
    text_mode_active: Res<TextModeActive>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    mut text_editor_state: Option<ResMut<TextEditorState>>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    text_mode_state: Res<TextModeState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Only handle keyboard input when text tool is active (prevents double sorts)
    if !text_mode_active.0 || current_tool.get_current() != Some("text") {
        return;
    }

    // Early exit if text editor state isn't ready yet
    let Some(mut text_editor_state) = text_editor_state else {
        return;
    };

    // Handle keyboard navigation for buffer mode
    if current_placement_mode.0 == TextPlacementMode::Buffer {
        // Arrow key navigation
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_editor_state.move_cursor_left();
            debug!("Text mode: moved cursor left to position {}", text_editor_state.cursor_position);
            keyboard_input.clear_just_pressed(KeyCode::ArrowLeft);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_editor_state.move_cursor_right();
            debug!("Text mode: moved cursor right to position {}", text_editor_state.cursor_position);
            keyboard_input.clear_just_pressed(KeyCode::ArrowRight);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            text_editor_state.move_cursor_up();
            debug!("Text mode: moved cursor up to position {}", text_editor_state.cursor_position);
            keyboard_input.clear_just_pressed(KeyCode::ArrowUp);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down();
            debug!("Text mode: moved cursor down to position {}", text_editor_state.cursor_position);
            keyboard_input.clear_just_pressed(KeyCode::ArrowDown);
        }

        // Home/End navigation
        if keyboard_input.just_pressed(KeyCode::Home) {
            text_editor_state.move_cursor_to(0);
            debug!("Text mode: moved cursor to beginning");
            keyboard_input.clear_just_pressed(KeyCode::Home);
        }
        if keyboard_input.just_pressed(KeyCode::End) {
            let end_position = text_editor_state.buffer.len();
            text_editor_state.move_cursor_to(end_position);
            debug!("Text mode: moved cursor to end");
            keyboard_input.clear_just_pressed(KeyCode::End);
        }

        // Delete/Backspace
        if keyboard_input.just_pressed(KeyCode::Delete) {
            text_editor_state.delete_sort_at_cursor();
            debug!("Text mode: deleted sort at cursor position");
            keyboard_input.clear_just_pressed(KeyCode::Delete);
        }
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            if text_editor_state.cursor_position > 0 {
                text_editor_state.move_cursor_left();
                text_editor_state.delete_sort_at_cursor();
                debug!("Text mode: backspace deleted sort");
            }
            keyboard_input.clear_just_pressed(KeyCode::Backspace);
        }
    }

    // Quick glyph switching with number keys (1-9)
    let mut glyph_switched = false;
    for (i, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, 
                     KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9].iter().enumerate() {
        if keyboard_input.just_pressed(*key) {
            let glyph_names: Vec<String> = app_state.workspace.font.glyphs.keys().cloned().collect();
            if let Some(glyph_name) = glyph_names.get(i) {
                // Switch to this glyph
                info!("Switched to glyph '{}' via number key {}", glyph_name, i + 1);
                glyph_switched = true;
                break;
            }
        }
    }
    
    if glyph_switched {
        return; // Early exit if we switched glyphs
    }

    // Quick sort placement with letter keys (works in both modes)
    let default_glyph_name = match &glyph_navigation.current_glyph {
        Some(name) => name.clone(),
        None => {
            // Try to use 'a' as default, or first available glyph
            if app_state.workspace.font.glyphs.contains_key("a") {
                "a".to_string()
            } else if let Some(first_glyph) = app_state.workspace.font.glyphs.keys().next() {
                first_glyph.clone()
            } else {
                return; // No glyphs available
            }
        }
    };

    // Get advance width for the default glyph (used for typing new glyphs)
    let default_advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&default_glyph_name) {
        glyph_data.advance_width as f32
    } else {
        600.0 // Default width
    };

    // Handle character input for quick placement
    // Collect keys to process and clear after the loop to avoid borrow checker issues
    let pressed_keys: Vec<KeyCode> = keyboard_input.get_just_pressed().cloned().collect();
    let mut keys_to_clear = Vec::new();
    
    for key in pressed_keys {
        let character_glyph = match key {
            KeyCode::KeyA => Some("a"),
            KeyCode::KeyB => Some("b"),
            KeyCode::KeyC => Some("c"),
            KeyCode::KeyD => Some("d"),
            KeyCode::KeyE => Some("e"),
            KeyCode::KeyF => Some("f"),
            KeyCode::KeyG => Some("g"),
            KeyCode::KeyH => Some("h"),
            KeyCode::KeyI => Some("i"),
            KeyCode::KeyJ => Some("j"),
            KeyCode::KeyK => Some("k"),
            KeyCode::KeyL => Some("l"),
            KeyCode::KeyM => Some("m"),
            KeyCode::KeyN => Some("n"),
            KeyCode::KeyO => Some("o"),
            KeyCode::KeyP => Some("p"),
            KeyCode::KeyQ => Some("q"),
            KeyCode::KeyR => Some("r"),
            KeyCode::KeyS => Some("s"),
            KeyCode::KeyT => Some("t"),
            KeyCode::KeyU => Some("u"),
            KeyCode::KeyV => Some("v"),
            KeyCode::KeyW => Some("w"),
            KeyCode::KeyX => Some("x"),
            KeyCode::KeyY => Some("y"),
            KeyCode::KeyZ => Some("z"),
            KeyCode::Digit0 => Some("zero"),
            KeyCode::Digit1 => Some("one"),
            KeyCode::Digit2 => Some("two"),
            KeyCode::Digit3 => Some("three"),
            KeyCode::Digit4 => Some("four"),
            KeyCode::Digit5 => Some("five"),
            KeyCode::Digit6 => Some("six"),
            KeyCode::Digit7 => Some("seven"),
            KeyCode::Digit8 => Some("eight"),
            KeyCode::Digit9 => Some("nine"),
            KeyCode::Space => Some("space"),
            _ => None,
        };

        if let Some(char_glyph) = character_glyph {
            // Check if this glyph exists in the font
            if app_state.workspace.font.glyphs.contains_key(char_glyph) {
                // Get the advance width for this specific glyph
                let char_advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(char_glyph) {
                    glyph_data.advance_width as f32
                } else {
                    default_advance_width // Fallback to default
                };
                
                match current_placement_mode.0 {
                    TextPlacementMode::Buffer => {
                        text_editor_state.insert_sort_at_cursor(char_glyph.to_string(), char_advance_width);
                        info!("Text mode: typed '{}' in buffer mode at position {}", 
                              char_glyph, text_editor_state.cursor_position - 1);
                    }
                    TextPlacementMode::Freeform => {
                        // For freeform mode with keyboard, place at cursor position or default
                        let freeform_pos = text_mode_state.cursor_position.unwrap_or(Vec2::new(0.0, 0.0));
                        text_editor_state.add_freeform_sort(char_glyph.to_string(), freeform_pos, char_advance_width);
                        info!("Text mode: typed '{}' in freeform mode at position {:?}", char_glyph, freeform_pos);
                        
                        // Move cursor to the right for next character
                        if let Some(current_pos) = text_mode_state.cursor_position {
                            // Update cursor position for next character (move right by advance width)
                            // Note: We would need to make text_mode_state mutable to update this
                            debug!("Next freeform position would be: {:?}", current_pos + Vec2::new(char_advance_width, 0.0));
                        }
                    }
                }
                
                // Mark key for clearing
                keys_to_clear.push(key);
            } else {
                debug!("Glyph '{}' not found in font, skipping", char_glyph);
            }
        }
    }
    
    // Clear all processed keys
    for key in keys_to_clear {
        keyboard_input.clear_just_pressed(key);
    }
}

/// System to handle global text tool shortcuts
pub fn handle_text_tool_shortcuts(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut current_tool: ResMut<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut current_placement_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
    text_editor_state: Option<Res<TextEditorState>>,
) {
    // Global shortcut to activate text tool
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        if current_tool.get_current() != Some("text") {
            current_tool.switch_to("text");
            info!("Activated text tool via keyboard shortcut");
        }
    }
    
    // When text tool is active, Tab to switch between buffer/freeform modes
    if current_tool.get_current() == Some("text") && keyboard_input.just_pressed(KeyCode::Tab) {
        let new_mode = match current_placement_mode.0 {
            TextPlacementMode::Buffer => TextPlacementMode::Freeform,
            TextPlacementMode::Freeform => TextPlacementMode::Buffer,
        };
        current_placement_mode.0 = new_mode;
        text_mode_config.default_placement_mode = new_mode.to_sort_layout_mode();
        info!("Switched text placement mode to: {:?}", new_mode);
    }
    
    // Ctrl+S to save current text layout
    if current_tool.get_current() == Some("text") && keyboard_input.just_pressed(KeyCode::KeyS) && 
       (keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) {
        if let Some(text_editor_state) = text_editor_state.as_ref() {
            let buffer_text: String = text_editor_state.buffer.iter()
                .map(|entry| entry.glyph_name.clone())
                .collect::<Vec<String>>()
                .join(" ");
            info!("Current text buffer: {}", buffer_text);
            info!("Buffer length: {} sorts", text_editor_state.buffer.len());
            info!("Cursor position: {}", text_editor_state.cursor_position);
            // In a real implementation, we could save this to a file or clipboard
        }
    }
    
    // F1 to show help
    if current_tool.get_current() == Some("text") && keyboard_input.just_pressed(KeyCode::F1) {
        info!("=== TEXT TOOL HELP ===");
        info!("T - Activate text tool");
        info!("Tab - Switch between Buffer/Freeform modes");
        info!("1-9 - Switch to glyph by number");
        info!("a-z - Type letters");
        info!("Space - Insert space");
        info!("Backspace - Delete character");
        info!("Arrow keys - Navigate cursor (Buffer mode)");
        info!("Home/End - Go to start/end (Buffer mode)");
        info!("Click - Place glyph at position");
        info!("Ctrl+S - Show current text buffer");
        info!("Escape - Exit text tool");
        info!("F1 - Show this help");
        info!("====================");
    }
    
    // Escape to exit text tool
    if current_tool.get_current() == Some("text") && keyboard_input.just_pressed(KeyCode::Escape) {
        if let Some(previous_tool) = current_tool.get_previous() {
            current_tool.switch_to(previous_tool);
            info!("Exited text tool via Escape key, returned to: {}", previous_tool);
        } else {
            current_tool.switch_to("select");
            info!("Exited text tool via Escape key, returned to select tool");
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