//! Text editor-based sort management system
//!
//! This system treats sorts as a text buffer that can be dynamically edited,
//! similar to how text editors work. Sorts are stored in a linear buffer
//! and mapped to a visual grid for display.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use cosmic_text::{Attrs, AttrsList, Buffer, Edit, Family, FontSystem, Metrics, Shaping, SwashCache};


use crate::core::state::{AppState, TextEditorState, SortEntry, SortLayoutMode, SortBuffer, GridConfig};
use crate::core::state::GlyphNavigation;
use crate::core::pointer::PointerInfo;
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;
use crate::editing::sort::ActiveSortState;
use crate::systems::sort_manager::SortPointEntity;
use crate::editing::selection::components::{Selectable, PointType, GlyphPointReference};
use crate::geometry::point::{EditPoint, EntityId, EntityKind};
use kurbo::Point;
use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use crate::rendering::sort_visuals::{render_sort_visuals, SortRenderStyle};
use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR, SORT_ACTIVE_OUTLINE_COLOR};

/// System to initialize text editor sorts when the font is loaded
/// This creates an empty TextEditorState for the text tool to work with
pub fn initialize_text_editor_sorts(
    mut commands: Commands,
    app_state: Res<AppState>,
    text_editor_state: Option<ResMut<TextEditorState>>,
    mut has_initialized: Local<bool>,
) {
    // Only run once on startup
    if *has_initialized {
        return;
    }
    
    if let Some(mut existing_state) = text_editor_state {
        // FORCE CLEAR all existing sorts completely - this prevents old glyph grid
        existing_state.buffer.clear();
        existing_state.cursor_position = 0;
        existing_state.selection = None;
        existing_state.viewport_offset = Vec2::ZERO;
        existing_state.grid_config = GridConfig::default();
        info!("FORCE CLEARED all existing sorts and text editor state for clean workspace");
    } else {
        // Create a completely empty text editor state with no sorts
        // Always create this, even if no font is loaded yet
        let empty_buffer = SortBuffer::new();
        
        let text_editor_state = TextEditorState {
            buffer: empty_buffer,
            cursor_position: 0,
            selection: None,
            viewport_offset: Vec2::ZERO,
            grid_config: GridConfig::default(),
        };
        
        commands.insert_resource(text_editor_state);
        info!("Created completely empty TextEditorState for clean workspace (font loaded: {})", 
              !app_state.workspace.font.glyphs.is_empty());
    }
    
    *has_initialized = true;
}

/// Handle mouse clicks on sorts in the text editor
pub fn handle_text_editor_sort_clicks(
    mut text_editor_state: ResMut<TextEditorState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ui_hover_state: Res<UiHoverState>,
    app_state: Res<crate::core::state::AppState>,
    pointer_info: Res<crate::core::pointer::PointerInfo>,
) {
    // Early return if hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Early return if no mouse button was pressed
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    // DEBUG: Log that we detected a left click
    debug!("LEFT MOUSE CLICKED - handle_text_editor_sort_clicks processing");
    debug!("Buffer has {} sorts", text_editor_state.buffer.len());
    
    debug!("Left mouse button pressed and no UI hover - processing click");

    // Use the centralized pointer info for coordinate conversion (same as sort placement)
    let world_position = pointer_info.design.to_raw();

    debug!(
        "Click at design position: ({:.1}, {:.1})", 
        world_position.x, 
        world_position.y
    );

    // Check for handle clicks first (more precise) 
    // Now using consistent coordinate system, so we can use a reasonable tolerance
    let handle_tolerance = 50.0; // Reasonable tolerance for handle clicks
    
    // Debug: Log what sorts we're checking against
    debug!("Checking {} sorts for handle clicks with tolerance {}", 
           text_editor_state.buffer.len(), handle_tolerance);
    
    // Debug: Log what we're looking for before the search
    debug!("Looking for sort at world position ({:.1}, {:.1}) with tolerance {}", 
           world_position.x, world_position.y, handle_tolerance);
    
    if let Some(clicked_sort_index) = text_editor_state.find_sort_handle_at_position(
        world_position, 
        handle_tolerance, 
        Some(&app_state.workspace.info.metrics)
    ) {
        debug!("Handle click detected on sort at index {}", clicked_sort_index);
        
        // Handle click - manage selection and active state relationship
        let is_ctrl_held = keyboard_input.pressed(KeyCode::ControlLeft) || 
                          keyboard_input.pressed(KeyCode::ControlRight);
        
        if is_ctrl_held {
            // Ctrl+click toggles selection without affecting other selections
            debug!("Ctrl+click: toggling selection for sort {}", clicked_sort_index);
            text_editor_state.toggle_sort_selection(clicked_sort_index);
            
            // Update active state based on selection count
            let selected_sorts = text_editor_state.get_selected_sorts();
            if selected_sorts.len() == 1 {
                // Only one sort selected → make it active
                let (selected_index, _) = selected_sorts[0];
                text_editor_state.activate_sort(selected_index);
                debug!("Single selection: activated sort {}", selected_index);
            } else {
                // Multiple sorts selected → clear active state
                text_editor_state.clear_active_state();
                debug!("Multiple selections: cleared active state");
            }
        } else {
            // Regular click: clear other selections, select this one, and make it active
            debug!("Regular click: clearing selections and selecting sort {}", clicked_sort_index);
            text_editor_state.clear_selections();
            text_editor_state.select_sort(clicked_sort_index);
            text_editor_state.activate_sort(clicked_sort_index);
        }
        
        if let Some(sort) = 
            text_editor_state.get_sort_at_position(clicked_sort_index) {
            let selected_count = text_editor_state.get_selected_sorts().len();
            let is_active = sort.is_active;
            let selection_action = if is_ctrl_held { "toggled" } else { "selected" };
            
            info!(
                "Handle click: {} sort '{}' at position {} ({} selected, active: {})", 
                selection_action,
                sort.glyph_name, 
                clicked_sort_index,
                selected_count,
                is_active
            );
        }
    } else {
        debug!("No handle click detected, checking for sort area clicks");
        
        // Check for general sort area clicks (larger tolerance)
        let sort_tolerance = 250.0; 
        if let Some(clicked_sort_index) = text_editor_state.find_sort_body_at_position(
            world_position, 
            sort_tolerance, 
        ) {
            debug!("Sort area click detected on sort at index {}", clicked_sort_index);
            
            // Sort area click - just activate for editing
            text_editor_state.activate_sort(clicked_sort_index);
            
            if let Some(sort) = 
                text_editor_state.get_sort_at_position(clicked_sort_index) {
                info!(
                    "Sort area click: activated '{}' for editing", 
                    sort.glyph_name
                );
            }
        } else {
            debug!("No sort clicked - clearing selections");
            
            // Empty area click - clear selections but DO NOT clear active state
            text_editor_state.clear_selections();
            if let Some(buffer_position) = text_editor_state
                .get_buffer_position_for_world_position(world_position) {
                // Move cursor to clicked position in buffer
                text_editor_state.move_cursor_to(buffer_position);
                
                debug!(
                    "Empty area click: cleared all selections, moved cursor to buffer position {} at ({:.1}, {:.1})", 
                    buffer_position, 
                    world_position.x, 
                    world_position.y
                );
            } else {
                debug!("Empty area click: cleared all selections");
            }
        }
    }
}

/// Render the text editor sorts
pub fn render_text_editor_sorts(
    mut gizmos: Gizmos,
    text_editor_state: Res<TextEditorState>,
    app_state: Res<AppState>,
) {
    let font_metrics = &app_state.workspace.info.metrics;

    // Render sort outlines
    for (i, entry) in text_editor_state.buffer.iter().enumerate() {
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&entry.glyph_name) {
            if let Some(position) = text_editor_state.get_sort_visual_position(i) {
                let metrics_color = if entry.is_active {
                    SORT_ACTIVE_METRICS_COLOR
                } else {
                    SORT_INACTIVE_METRICS_COLOR
                };
                render_sort_visuals(
                    &mut gizmos,
                    &glyph_data.outline,
                    glyph_data.advance_width as f32,
                    &app_state.workspace.info.metrics,
                    position,
                    metrics_color,
                    SortRenderStyle::TextBuffer,
                    entry.is_selected,
                );
            }
        }
    }
    
    // Render points for the active sort
    if let Some((active_sort_idx, _)) = text_editor_state.get_active_sort() {
        if let Some(entry) = text_editor_state.buffer.get(active_sort_idx) {
            if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&entry.glyph_name) {
                if let Some(outline_data) = &glyph_data.outline {
                    if let Some(position) = text_editor_state.get_sort_visual_position(active_sort_idx) {
                        crate::rendering::glyph_outline::draw_glyph_points_and_handles_at_position(
                            &mut gizmos,
                            outline_data,
                            position,
                        );
                    }
                }
            }
        }
    }

    // Render cursor for the active text buffer
    if let Some((root_index, root_sort)) = text_editor_state.get_active_sort() {
        if root_sort.is_buffer_root && root_sort.buffer_cursor_position.is_some() {
            let cursor_pos_in_text = root_sort.buffer_cursor_position.unwrap();
            let cursor_world_pos = calculate_cursor_position_in_text(&text_editor_state, root_index, cursor_pos_in_text);
            let ascender = font_metrics.ascender.unwrap_or(800.0) as f32;
            let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
            let cursor_color = SORT_ACTIVE_OUTLINE_COLOR;
            let circle_radius = 12.0;

            // Draw vertical line from descender to ascender
            gizmos.line_2d(
                Vec2::new(cursor_world_pos.x, cursor_world_pos.y + descender),
                Vec2::new(cursor_world_pos.x, cursor_world_pos.y + ascender),
                cursor_color,
            );
            // Draw circle at top (ascender)
            gizmos.circle_2d(
                Vec2::new(cursor_world_pos.x, cursor_world_pos.y + ascender),
                circle_radius,
                cursor_color,
            );
            // Draw circle at bottom (descender)
            gizmos.circle_2d(
                Vec2::new(cursor_world_pos.x, cursor_world_pos.y + descender),
                circle_radius,
                cursor_color,
            );
        }
    }
}

/// Check if a key is used as a tool shortcut
#[allow(dead_code)]
fn is_tool_shortcut_key(key: KeyCode) -> bool {
    matches!(key, 
        KeyCode::KeyT |  // Text tool
        KeyCode::KeyP |  // Pen tool  
        KeyCode::KeyV |  // Select tool
        KeyCode::KeyK |  // Knife tool
        KeyCode::KeyH |  // Hyper tool
        KeyCode::KeyR |  // Shapes tool
        KeyCode::KeyM    // Measure/Metaballs tool
    )
}

/// Convert key code to character, considering shift state
fn key_code_to_char(key: KeyCode, keyboard_input: &ButtonInput<KeyCode>) -> Option<char> {
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || 
                       keyboard_input.pressed(KeyCode::ShiftRight);
    
    match key {
        KeyCode::KeyA => Some(if shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift_pressed { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift_pressed { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift_pressed { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift_pressed { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift_pressed { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift_pressed { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift_pressed { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift_pressed { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift_pressed { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift_pressed { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift_pressed { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift_pressed { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift_pressed { '+' } else { '=' }),
        KeyCode::BracketLeft => Some(if shift_pressed { '{' } else { '[' }),
        KeyCode::BracketRight => Some(if shift_pressed { '}' } else { ']' }),
        KeyCode::Backslash => Some(if shift_pressed { '|' } else { '\\' }),
        KeyCode::Semicolon => Some(if shift_pressed { ':' } else { ';' }),
        KeyCode::Quote => Some(if shift_pressed { '"' } else { '\'' }),
        KeyCode::Comma => Some(if shift_pressed { '<' } else { ',' }),
        KeyCode::Period => Some(if shift_pressed { '>' } else { '.' }),
        KeyCode::Slash => Some(if shift_pressed { '?' } else { '/' }),
        KeyCode::Backquote => Some(if shift_pressed { '~' } else { '`' }),
        _ => None,
    }
}

/// Convert Unicode character to glyph name using font data
fn unicode_to_glyph_name(unicode_char: char, app_state: &AppState) -> Option<String> {
    // First try to find the glyph by Unicode codepoint
    let _codepoint_hex = format!("{:04X}", unicode_char as u32);
    
    // Look for glyph with this Unicode codepoint
    for (glyph_name, glyph_data) in &app_state.workspace.font.glyphs {
        if glyph_data.unicode_values.contains(&unicode_char) {
            return Some(glyph_name.clone());
        }
    }
    
    // If no exact match, try to find a glyph with the same name as the character
    let char_name = unicode_char.to_string();
    if app_state.workspace.font.glyphs.contains_key(&char_name) {
        return Some(char_name);
    }
    
    // For common characters, try some fallback mappings
    let fallback_mapping = match unicode_char {
        ' ' => "space",
        '0' => "zero",
        '1' => "one", 
        '2' => "two",
        '3' => "three",
        '4' => "four",
        '5' => "five",
        '6' => "six",
        '7' => "seven",
        '8' => "eight",
        '9' => "nine",
        _ => return None,
    };
    
    if app_state.workspace.font.glyphs.contains_key(fallback_mapping) {
        Some(fallback_mapping.to_string())
    } else {
        None
    }
}

/// Handle keyboard input for text editing with proper Unicode support
pub fn handle_text_editor_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
) {
    // Only handle keyboard input when text tool is active AND in Insert mode
    if current_tool.get_current() != Some("text") || 
       current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        return;
    }
    
    // Check if we have buffer sorts for various operations
    let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
    
    // Move cursor with arrow keys - only work if we have text sorts
    if has_text_sorts {
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_editor_state.move_cursor_right();
            debug!("Insert mode: moved cursor right to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_editor_state.move_cursor_left();
            debug!("Insert mode: moved cursor left to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            text_editor_state.move_cursor_up();
            debug!("Insert mode: moved cursor up to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down();
            debug!("Insert mode: moved cursor down to position {}", text_editor_state.cursor_position);
        }
        
        // Home/End keys
        if keyboard_input.just_pressed(KeyCode::Home) {
            text_editor_state.move_cursor_to(0);
            debug!("Insert mode: moved cursor to beginning");
        }
        
        if keyboard_input.just_pressed(KeyCode::End) {
            let buffer_len = text_editor_state.buffer.len();
            text_editor_state.move_cursor_to(buffer_len);
            debug!("Insert mode: moved cursor to end");
        }
    }
    
    // Ctrl+T to create a new text buffer - avoid conflict with 'T' tool shortcut
    if keyboard_input.just_pressed(KeyCode::KeyT) && 
       (keyboard_input.pressed(KeyCode::ControlLeft) || 
        keyboard_input.pressed(KeyCode::ControlRight)) {
        // For now, create buffer root at center of screen
        // TODO: Use actual mouse/click position from text tool
        text_editor_state.create_text_root(Vec2::new(500.0, 0.0));
        info!("Insert mode: created new text buffer");
    }
    
    // Delete/Backspace - only work if we have text sorts
    if has_text_sorts {
        if keyboard_input.just_pressed(KeyCode::Delete) {
            text_editor_state.delete_sort_at_cursor();
            info!("Insert mode: deleted sort at cursor position");
        }
        
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            // The delete_sort_at_cursor function handles all logic,
            // including checking if deletion is possible and moving the cursor.
            text_editor_state.delete_sort_at_cursor();
            info!("Insert mode: backspace pressed");
        }
    }
    
    // Handle text input by mapping key codes to characters
    // This is a simplified approach for now - in a real implementation,
    // you'd want to use Bevy's text input system or a third-party crate
    for key in keyboard_input.get_just_pressed() {
        if let Some(unicode_char) = key_code_to_char(*key, &keyboard_input) {
            if let Some(glyph_name) = unicode_to_glyph_name(unicode_char, &app_state) {
                // Check if the glyph exists in the font
                if let Some(glyph_data) = 
                    app_state.workspace.font.glyphs.get(&glyph_name) {
                    let advance_width = glyph_data.advance_width as f32;
                    
                    // Check if any text sorts exist
                    if !has_text_sorts {
                        // No buffer sorts exist, create a buffer root at center of screen
                        // TODO: Use actual mouse/click position if available
                        let center_position = Vec2::new(500.0, 0.0);
                        text_editor_state.create_text_root(center_position);
                        
                        // Now insert the character at the new buffer root
                        text_editor_state.insert_sort_at_cursor(
                            glyph_name.clone(), 
                            advance_width
                        );
                        
                        info!(
                            "Insert mode: created new buffer root and inserted character '{}' (glyph: '{}') at center", 
                            unicode_char, glyph_name
                        );
                    } else {
                        // Buffer sorts exist, use normal insertion logic
                        text_editor_state.insert_sort_at_cursor(
                            glyph_name.clone(), 
                            advance_width
                        );
                        info!(
                            "Insert mode: inserted character '{}' (glyph: '{}') at cursor position {}", 
                            unicode_char, glyph_name, text_editor_state.cursor_position
                        );
                    }
                } else {
                    info!("Insert mode: glyph '{}' for character '{}' not found in font", 
                          glyph_name, unicode_char);
                }
            } else {
                info!("Insert mode: no glyph mapping found for character '{}' (U+{:04X})", 
                      unicode_char, unicode_char as u32);
            }
        }
    }
}

/// Calculate cursor position within a text sequence
fn calculate_cursor_position_in_text(
    text_editor_state: &TextEditorState, 
    root_index: usize, 
    cursor_pos_in_text: usize
) -> Vec2 {
    if let Some(root_sort) = text_editor_state.buffer.get(root_index) {
        let root_position = root_sort.freeform_position;
        
        if cursor_pos_in_text == 0 {
            // Cursor is at the root position (for empty roots or at the start)
            root_position
        } else {
            // Cursor is after one or more sorts - calculate cumulative x offset
            let mut x_offset = 0.0;
            
            // Sum up advance widths from the root up to the cursor position
            for i in 0..cursor_pos_in_text {
                let sort_index = root_index + i;
                if let Some(sort) = text_editor_state.buffer.get(sort_index) {
                    if sort.layout_mode == SortLayoutMode::Text && !sort.glyph_name.is_empty() {
                        x_offset += sort.advance_width;
                    }
                }
            }
            
            root_position + Vec2::new(x_offset, 0.0)
        }
    } else {
        Vec2::ZERO
    }
}

/// Debug system to log text editor state
pub fn debug_text_editor_state(
    text_editor_state: Res<TextEditorState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        info!("=== Text Editor Debug ===");
        info!("Buffer length: {}", text_editor_state.buffer.len());
        info!("Cursor position: {}", text_editor_state.cursor_position);
        
        if let Some((active_pos, active_sort)) = 
            text_editor_state.get_active_sort() {
            info!(
                "Active sort: '{}' at position {}", 
                active_sort.glyph_name, 
                active_pos
            );
        } else {
            info!("No active sort");
        }
        
        // Log first few sorts
        for (i, sort) in text_editor_state.buffer.iter().take(5).enumerate() {
            info!(
                "Sort {}: '{}' (active: {}, buffer_root: {}, cursor: {:?})", 
                i, 
                sort.glyph_name, 
                sort.is_active,
                sort.is_buffer_root,
                sort.buffer_cursor_position
            );
        }
    }
    
    // F2: Debug selection states
    if keyboard_input.just_pressed(KeyCode::F2) {
        info!("=== Selection Debug ===");
        let selected_sorts = text_editor_state.get_selected_sorts();
        info!("Total sorts: {}", text_editor_state.buffer.len());
        info!("Selected sorts: {}", selected_sorts.len());
        
        for (index, sort) in text_editor_state.buffer.iter().enumerate() {
            if sort.is_selected || sort.is_active {
                info!(
                    "Sort {}: '{}' - Selected: {}, Active: {}, Layout: {:?}", 
                    index, 
                    sort.glyph_name, 
                    sort.is_selected, 
                    sort.is_active,
                    sort.layout_mode
                );
            }
        }
        
        if selected_sorts.is_empty() {
            info!("No sorts are currently selected");
        }
    }
}

/// System to sync TextEditorState active sort with ActiveSortState and create ECS entities
/// This bridges the gap between the text editor system and the selection system
pub fn sync_text_editor_active_sort(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    mut active_sort_state: ResMut<ActiveSortState>,
    sort_point_entities: Query<(Entity, &mut SortPointEntity)>,
    _selectable_points: Query<Entity, With<Selectable>>,
    app_state: Res<AppState>,
) {
    debug!("=== SYNC TEXT EDITOR ACTIVE SORT ===");
    debug!("TextEditorState buffer length: {}", text_editor_state.buffer.len());
    
    // Find the active sort in TextEditorState
    let active_sort = text_editor_state.get_active_sort();
    
    if let Some((index, sort_entry)) = active_sort {
        debug!("Found active sort: '{}' at index {}", sort_entry.glyph_name, index);
        
        // Create a placeholder entity for the active sort if we don't have one
        let sort_entity = if let Some(entity) = active_sort_state.active_sort_entity {
            debug!("Using existing sort entity: {:?}", entity);
            entity
        } else {
            let entity = commands.spawn_empty().id();
            active_sort_state.active_sort_entity = Some(entity);
            info!("Created new sort entity {:?} for active sort '{}'", entity, sort_entry.glyph_name);
            entity
        };
        
        // Clear existing sort point entities
        let existing_count = sort_point_entities.iter().count();
        debug!("Clearing {} existing sort point entities", existing_count);
        for (entity, _) in sort_point_entities.iter() {
            commands.entity(entity).despawn();
        }
        
        // Create ECS entities for the active sort's points
        debug!("Looking for glyph '{}' in font", sort_entry.glyph_name);
        if let Some(glyph) = app_state.workspace.font.get_glyph(&sort_entry.glyph_name) {
            debug!("Found glyph '{}', checking for outline", sort_entry.glyph_name);
            if let Some(outline) = &glyph.outline {
                debug!("Found outline with {} contours", outline.contours.len());
                for (contour_index, contour) in outline.contours.iter().enumerate() {
                    debug!("Processing contour {} with {} points", contour_index, contour.points.len());
                    for (point_index, point) in contour.points.iter().enumerate() {
                        let _entity_id = EntityId::point(index as u32, point_index as u16);
                        let is_on_curve = matches!(point.point_type, crate::core::state::font_data::PointTypeData::Move | crate::core::state::font_data::PointTypeData::Line);
                        // Calculate the world position: sort position + point offset
                        let point_world_pos = sort_entry.freeform_position + Vec2::new(point.x as f32, point.y as f32);
                        
                        let point_entity = commands.spawn((
                            EditPoint {
                                position: Point::new(point.x, point.y),
                                point_type: match point.point_type {
                                    crate::core::state::font_data::PointTypeData::Move => norad::PointType::Move,
                                    crate::core::state::font_data::PointTypeData::Line => norad::PointType::Line,
                                    crate::core::state::font_data::PointTypeData::OffCurve => norad::PointType::OffCurve,
                                    crate::core::state::font_data::PointTypeData::Curve => norad::PointType::Curve,
                                    crate::core::state::font_data::PointTypeData::QCurve => norad::PointType::QCurve,
                                },
                            },
                            GlyphPointReference {
                                glyph_name: sort_entry.glyph_name.clone(),
                                contour_index: contour_index,
                                point_index: point_index,
                            },
                            PointType {
                                is_on_curve,
                            },
                            Selectable,
                            SortPointEntity { sort_entity },
                            Transform::from_translation(point_world_pos.extend(0.0)),
                            GlobalTransform::default(),
                        )).id();
                        debug!("Created point entity {:?} for sort '{}' point ({}, {}) at world position ({:.1}, {:.1})", 
                               point_entity, sort_entry.glyph_name, contour_index, point_index, point_world_pos.x, point_world_pos.y);
                    }
                }
                debug!("Finished creating ECS entities for sort '{}'", sort_entry.glyph_name);
            } else {
                debug!("Glyph '{}' has no outline", sort_entry.glyph_name);
            }
        } else {
            debug!("Glyph '{}' not found in font", sort_entry.glyph_name);
        }
    } else {
        debug!("No active sort found in TextEditorState");
        // No active sort - clear the active sort state and remove point entities
        if active_sort_state.active_sort_entity.is_some() {
            info!("Clearing active sort state - no active sort in TextEditorState");
            active_sort_state.active_sort_entity = None;
        }
        
        // Remove all sort point entities
        let existing_count = sort_point_entities.iter().count();
        debug!("Removing {} sort point entities (no active sort)", existing_count);
        for (entity, _) in sort_point_entities.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle sort placement using the centralized input system
pub fn handle_sort_placement_input(
    mut input_events: EventReader<crate::core::input::InputEvent>,
    _input_state: Res<crate::core::input::InputState>,
    text_editor_state: Option<ResMut<TextEditorState>>,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    pointer_info: Res<crate::core::pointer::PointerInfo>,
    camera_query: Query<&Projection, With<DesignCamera>>,
) {
    // Debug: Log that this system is running
    debug!("Sort placement system: checking for input events");
    
    // Only handle input if text tool is active
    if current_tool.get_current() != Some("text") {
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
    
    // Get the camera zoom scale
    let projection = match camera_query.single() {
        Ok(proj) => proj,
        Err(_) => return,
    };
    let zoom_scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    let grid_size = calculate_dynamic_grid_size(zoom_scale);

    // Use the centralized pointer info for coordinate conversion
    let raw_cursor_world_pos = pointer_info.design.to_raw();
    // Snap to checkerboard grid
    let snapped_position = (raw_cursor_world_pos / grid_size).round() * grid_size;
    
    // Process input events
    for event in input_events.read() {
        match event {
            crate::core::input::InputEvent::MouseClick { button, position, modifiers: _ } => {
                if *button == bevy::input::mouse::MouseButton::Left {
                    debug!("Sort placement: MouseClick at position {:?}", position);
                    
                    // Check if there's already a sort at this position (handle or body)
                    let handle_tolerance = 50.0;
                    let body_tolerance = 250.0;
                    
                    let has_existing_sort = text_editor_state.find_sort_handle_at_position(
                        snapped_position, 
                        handle_tolerance, 
                        Some(&app_state.workspace.info.metrics)
                    ).is_some() || text_editor_state.find_sort_body_at_position(
                        snapped_position, 
                        body_tolerance
                    ).is_some();
                    
                    if has_existing_sort {
                        debug!("Sort placement: Clicked on existing sort, skipping placement");
                        return; // Exit early to let the click detection system handle this
                    }
                    
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
                                continue;
                            }
                        }
                    };
                    
                    // Get advance width for the glyph
                    let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                        glyph_data.advance_width as f32
                    } else {
                        600.0 // Default width
                    };
                    
                    // Position calculation: sort should be at baseline, handle at descender
                    let descender = app_state.workspace.info.metrics.descender.unwrap() as f32;
                    // Sort position should be at baseline (cursor position), not offset by descender
                    let raw_sort_position = snapped_position;
                    
                    match current_placement_mode.0 {
                        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Text => {
                            // Buffer mode: Create buffer sort at the calculated position
                            text_editor_state.create_text_sort_at_position(glyph_name.clone(), raw_sort_position, advance_width);
                            info!("Placed sort '{}' in buffer mode at position ({:.1}, {:.1}) with descender offset {:.1}", 
                                  glyph_name, raw_sort_position.x, raw_sort_position.y, descender);
                            // Automatically switch to Insert mode after placing a buffer sort
                            // Note: This would need to be handled by the text toolbar system
                        }
                        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert => {
                            // Insert mode: Don't place new sorts, this mode is for editing existing buffer sorts
                            // User should use arrow keys and typing to edit buffer content
                            info!("Insert mode: Use keyboard to edit buffer sorts, not mouse clicks");
                        }
                        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Freeform => {
                            // Freeform mode: Add sort at the calculated position
                            text_editor_state.add_freeform_sort(glyph_name.clone(), raw_sort_position, advance_width);
                            info!("Placed sort '{}' in freeform mode at position ({:.1}, {:.1}) with descender offset {:.1}", 
                                  glyph_name, raw_sort_position.x, raw_sort_position.y, descender);
                        }
                    }
                }
            }
            _ => {
                // Ignore other event types
            }
        }
    }
}

/// System to handle text input using cosmic-text for proper Unicode support
pub fn handle_text_input_with_cosmic(
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_buffer: Local<Option<Buffer>>,
    mut font_system: Local<Option<FontSystem>>,
    mut swash_cache: Local<Option<SwashCache>>,
) {
    // Only handle text input when text tool is active AND in Insert mode
    if current_tool.get_current() != Some("text") || 
       current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        return;
    }

    // Initialize cosmic-text components if not already done
    if font_system.is_none() {
        *font_system = Some(FontSystem::new());
    }
    if swash_cache.is_none() {
        *swash_cache = Some(SwashCache::new());
    }
    if text_buffer.is_none() {
        let mut buffer = Buffer::new(
            font_system.as_mut().unwrap(),
            Metrics::new(16.0, 20.0)
        );
        buffer.set_text(font_system.as_mut().unwrap(), "", &Attrs::new(), Shaping::Advanced);
        *text_buffer = Some(buffer);
    }

    let _buffer = text_buffer.as_mut().unwrap();
    let _font_system = font_system.as_mut().unwrap();
    let _swash_cache = swash_cache.as_mut().unwrap();

    // Handle keyboard input for text editing
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                // Handle backspace in the text editor state
                text_editor_state.delete_sort_at_cursor();
                info!("Text input: Backspace pressed");
            }
            KeyCode::Enter => {
                // Handle enter key
                info!("Text input: Enter pressed");
            }
            KeyCode::Space => {
                // Add space character immediately
                if let Some(glyph_name) = unicode_to_glyph_name(' ', &app_state) {
                    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                        let advance_width = glyph_data.advance_width as f32;
                        
                        // Check if any text sorts exist
                        let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
                        
                        if !has_text_sorts {
                            // Create a buffer root at center of screen
                            let center_position = Vec2::new(500.0, 0.0);
                            text_editor_state.create_text_root(center_position);
                        }
                        
                        text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                        info!("Text input: Added space");
                    }
                }
            }
            _ => {
                // Process each character immediately for instant feedback
                if let Some(ch) = key_code_to_char(*key, &keyboard_input) {
                    // Convert character to glyph and insert immediately
                    if let Some(glyph_name) = unicode_to_glyph_name(ch, &app_state) {
                        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                            let advance_width = glyph_data.advance_width as f32;
                            
                            // Check if any text sorts exist
                            let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
                            
                            if !has_text_sorts {
                                // Create a buffer root at center of screen
                                let center_position = Vec2::new(500.0, 0.0);
                                text_editor_state.create_text_root(center_position);
                            }
                            
                            // Insert the character immediately
                            text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                            info!("Text input: Inserted character '{}' (glyph: '{}') immediately", ch, glyph_name);
                        } else {
                            info!("Text input: Glyph '{}' not found in font for character '{}'", glyph_name, ch);
                        }
                    } else {
                        info!("Text input: No glyph mapping found for character '{}' (U+{:04X})", ch, ch as u32);
                    }
                }
            }
        }
    }
}

/// System to handle Arabic and Unicode text input using cosmic-text
/// This system can handle any Unicode character including Arabic, Hebrew, etc.
pub fn handle_arabic_text_input(
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_buffer: Local<Option<Buffer>>,
    mut font_system: Local<Option<FontSystem>>,
    mut swash_cache: Local<Option<SwashCache>>,
    mut input_text: Local<String>,
    mut last_input_time: Local<f64>,
    time: Res<Time>,
) {
    // Only handle text input when text tool is active AND in Insert mode
    if current_tool.get_current() != Some("text") || 
       current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        return;
    }

    // Initialize cosmic-text components if not already done
    if font_system.is_none() {
        *font_system = Some(FontSystem::new());
    }
    if swash_cache.is_none() {
        *swash_cache = Some(SwashCache::new());
    }
    if text_buffer.is_none() {
        let mut buffer = Buffer::new(
            font_system.as_mut().unwrap(),
            Metrics::new(16.0, 20.0)
        );
        buffer.set_text(font_system.as_mut().unwrap(), "", &Attrs::new(), Shaping::Advanced);
        *text_buffer = Some(buffer);
    }

    let _buffer = text_buffer.as_mut().unwrap();
    let _font_system = font_system.as_mut().unwrap();
    let _swash_cache = swash_cache.as_mut().unwrap();

    // Handle keyboard input for text editing
    for key in keyboard_input.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                // Handle backspace in the text editor state
                text_editor_state.delete_sort_at_cursor();
                info!("Arabic text input: Backspace pressed");
            }
            KeyCode::Enter => {
                // Process the accumulated text and convert to sorts
                if !input_text.is_empty() {
                    process_unicode_text_to_sorts(&mut text_editor_state, &app_state, &input_text);
                    input_text.clear();
                }
                info!("Arabic text input: Enter pressed");
            }
            KeyCode::Space => {
                // Process current buffer and add space
                if !input_text.is_empty() {
                    process_unicode_text_to_sorts(&mut text_editor_state, &app_state, &input_text);
                    input_text.clear();
                }
                // Add space character
                if let Some(glyph_name) = unicode_to_glyph_name(' ', &app_state) {
                    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                        let advance_width = glyph_data.advance_width as f32;
                        text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                        info!("Arabic text input: Added space");
                    }
                }
            }
            _ => {
                // For now, we'll use the key_code_to_char function for basic input
                // In a full implementation, you'd use cosmic-text's IME support or platform APIs
                if let Some(ch) = key_code_to_char(*key, &keyboard_input) {
                    // Add character to input buffer
                    input_text.push(ch);
                    *last_input_time = time.elapsed_secs_f64();
                    info!("Arabic text input: Added character '{}' to buffer", ch);
                }
            }
        }
    }

    // Process text buffer when it gets long enough or after a delay
    let current_time = time.elapsed_secs_f64();
    if input_text.len() >= 3 || (current_time - *last_input_time > 0.5 && !input_text.is_empty()) {
        if !input_text.is_empty() {
            process_unicode_text_to_sorts(&mut text_editor_state, &app_state, &input_text);
            input_text.clear();
        }
    }
}

/// Process Unicode text (including Arabic) and convert to sorts
/// This function handles the conversion from Unicode text to glyph sorts
fn process_unicode_text_to_sorts(
    text_editor_state: &mut TextEditorState,
    app_state: &AppState,
    text: &str,
) {
    info!("Processing Unicode text: '{}'", text);
    
    // Check if any text sorts exist
    let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
    
    if !has_text_sorts {
        // Create a buffer root at center of screen
        let center_position = Vec2::new(500.0, 0.0);
        text_editor_state.create_text_root(center_position);
    }
    
    // Process each character in the text
    for ch in text.chars() {
        if let Some(glyph_name) = unicode_to_glyph_name(ch, app_state) {
            if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                let advance_width = glyph_data.advance_width as f32;
                
                // Insert the character
                text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                info!("Arabic text input: Inserted Unicode character '{}' (U+{:04X}) as glyph '{}'", 
                      ch, ch as u32, glyph_name);
            } else {
                info!("Arabic text input: Glyph '{}' not found in font for character '{}'", 
                      glyph_name, ch);
            }
        } else {
            info!("Arabic text input: No glyph mapping found for Unicode character '{}' (U+{:04X})", 
                  ch, ch as u32);
        }
    }
}

/// Handle Unicode character input using Bevy's KeyboardInput events
/// This system processes the text field from KeyboardInput events for Unicode characters
pub fn handle_unicode_text_input(
    mut keyboard_input_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
) {
    // Only handle keyboard input when text tool is active AND in Insert mode
    if current_tool.get_current() != Some("text") || 
       current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        return;
    }
    
    // Check if we have buffer sorts for various operations
    let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
    
    // Handle navigation keys (arrow keys, home, end, etc.)
    if has_text_sorts {
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_editor_state.move_cursor_right();
            debug!("Unicode input: moved cursor right to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_editor_state.move_cursor_left();
            debug!("Unicode input: moved cursor left to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            text_editor_state.move_cursor_up();
            debug!("Unicode input: moved cursor up to position {}", text_editor_state.cursor_position);
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down();
            debug!("Unicode input: moved cursor down to position {}", text_editor_state.cursor_position);
        }
        
        // Home/End keys
        if keyboard_input.just_pressed(KeyCode::Home) {
            text_editor_state.move_cursor_to(0);
            debug!("Unicode input: moved cursor to beginning");
        }
        
        if keyboard_input.just_pressed(KeyCode::End) {
            let buffer_len = text_editor_state.buffer.len();
            text_editor_state.move_cursor_to(buffer_len);
            debug!("Unicode input: moved cursor to end");
        }
        
        // Delete/Backspace
        if keyboard_input.just_pressed(KeyCode::Delete) {
            text_editor_state.delete_sort_at_cursor();
            info!("Unicode input: deleted sort at cursor position");
        }
        
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            text_editor_state.delete_sort_at_cursor();
            info!("Unicode input: backspace pressed");
        }
    }
    
    // Ctrl+T to create a new text buffer
    if keyboard_input.just_pressed(KeyCode::KeyT) && 
       (keyboard_input.pressed(KeyCode::ControlLeft) || 
        keyboard_input.pressed(KeyCode::ControlRight)) {
        text_editor_state.create_text_root(Vec2::new(500.0, 0.0));
        info!("Unicode input: created new text buffer");
    }
    
    for event in keyboard_input_events.read() {
        // Only process key press events (not releases)
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        
        // Check if this event has text content (Unicode characters)
        if let Some(text) = &event.text {
            if !text.is_empty() {
                debug!("Unicode input received: '{}' (U+{:04X})", text, text.chars().next().unwrap() as u32);
                
                // Process each character in the text
                for unicode_char in text.chars() {
                    if let Some(glyph_name) = unicode_to_glyph_name(unicode_char, &app_state) {
                        // Check if the glyph exists in the font
                        if let Some(glyph_data) = 
                            app_state.workspace.font.glyphs.get(&glyph_name) {
                            let advance_width = glyph_data.advance_width as f32;
                            
                            // Check if any text sorts exist
                            if !has_text_sorts {
                                // No buffer sorts exist, create a buffer root at center of screen
                                // TODO: Use actual mouse/click position if available
                                let center_position = Vec2::new(500.0, 0.0);
                                text_editor_state.create_text_root(center_position);
                                
                                // Now insert the character at the new buffer root
                                text_editor_state.insert_sort_at_cursor(
                                    glyph_name.clone(), 
                                    advance_width
                                );
                                
                                info!(
                                    "Unicode input: created new buffer root and inserted character '{}' (glyph: '{}') at center", 
                                    unicode_char, glyph_name
                                );
                            } else {
                                // Buffer sorts exist, use normal insertion logic
                                text_editor_state.insert_sort_at_cursor(
                                    glyph_name.clone(), 
                                    advance_width
                                );
                                info!(
                                    "Unicode input: inserted character '{}' (glyph: '{}') at cursor position {}", 
                                    unicode_char, glyph_name, text_editor_state.cursor_position
                                );
                            }
                        } else {
                            info!("Unicode input: glyph '{}' for character '{}' not found in font", 
                                  glyph_name, unicode_char);
                        }
                    } else {
                        info!("Unicode input: no glyph mapping found for character '{}' (U+{:04X})", 
                              unicode_char, unicode_char as u32);
                    }
                }
            }
        }
    }
} 