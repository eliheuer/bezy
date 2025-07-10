//! Text editor-based sort management system
//!
//! This system treats sorts as a text buffer that can be dynamically edited,
//! similar to how text editors work. Sorts are stored in a linear buffer
//! and mapped to a visual grid for display.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use cosmic_text::{Attrs, AttrsList, Buffer, Edit, Family, FontSystem, Metrics, Shaping, SwashCache};

use std::collections::HashMap;

use crate::core::state::{AppState, TextEditorState, SortEntry, SortLayoutMode, SortBuffer, GridConfig, SortKind, ActiveSortEntity};
use crate::core::state::GlyphNavigation;
use crate::core::pointer::PointerInfo;
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;
use crate::editing::sort::{ActiveSort, InactiveSort, ActiveSortState};
use crate::systems::sort_manager::SortPointEntity;
use crate::editing::selection::components::{GlyphPointReference, PointType, Selectable};
use crate::geometry::point::EditPoint;
use kurbo::Point;
use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use crate::rendering::sort_visuals::{render_sort_visuals, SortRenderStyle};
use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR, SORT_ACTIVE_OUTLINE_COLOR};

/// System to manage sort activation in ECS
/// This ensures only one sort is active at a time and syncs with TextEditorState
pub fn manage_sort_activation(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    mut active_sort_entity: ResMut<ActiveSortEntity>,
    mut active_sort_state: ResMut<ActiveSortState>,
    sort_entities: Query<(Entity, &crate::editing::sort::Sort)>,
    active_sorts: Query<Entity, With<ActiveSort>>,
    _inactive_sorts: Query<Entity, With<InactiveSort>>,
) {
    // Check if TextEditorState has an active sort that doesn't match ECS
    if let Some((buffer_index, sort_entry)) = text_editor_state.get_active_sort() {
        // Find the corresponding ECS entity for this buffer position
        let mut found_entity = None;
        for (entity, sort) in sort_entities.iter() {
            // Match by glyph name and approximate position
            if sort.glyph_name == sort_entry.kind.glyph_name().to_string() {
                // Additional position check to ensure we get the right sort instance
                let sort_pos = text_editor_state.get_sort_visual_position(buffer_index);
                if let Some(expected_pos) = sort_pos {
                    let distance = (sort.position - expected_pos).length();
                    if distance < 10.0 { // Small tolerance for position matching
                        found_entity = Some(entity);
                        break;
                    }
                }
            }
        }
        
        // If we found an entity and it's not currently active, activate it
        if let Some(entity) = found_entity {
            if active_sort_entity.entity != Some(entity) {
                // Deactivate all currently active sorts
                for active_entity in active_sorts.iter() {
                    commands
                        .entity(active_entity)
                        .remove::<ActiveSort>()
                        .insert(InactiveSort);
                }
                
                // Activate the new sort
                commands
                    .entity(entity)
                    .remove::<InactiveSort>()
                    .insert(ActiveSort);
                
                // Update our tracking resources
                active_sort_entity.entity = Some(entity);
                active_sort_entity.buffer_index = Some(buffer_index);
                active_sort_state.active_sort_entity = Some(entity);
                
                info!("[manage_sort_activation] Activated sort entity {:?} for buffer index {} (glyph: '{}')", 
                      entity, buffer_index, sort_entry.kind.glyph_name());
            }
        } else {
            // No matching ECS entity found - this might be a new sort that hasn't been spawned yet
            debug!("[manage_sort_activation] No ECS entity found for active sort at buffer index {} (glyph: '{}')", 
                   buffer_index, sort_entry.kind.glyph_name());
        }
    } else {
        // No active sort in TextEditorState, deactivate all ECS sorts
        if active_sort_entity.entity.is_some() {
            for active_entity in active_sorts.iter() {
                commands
                    .entity(active_entity)
                    .remove::<ActiveSort>()
                    .insert(InactiveSort);
            }
            
            active_sort_entity.entity = None;
            active_sort_entity.buffer_index = None;
            active_sort_state.active_sort_entity = None;
            
            info!("[manage_sort_activation] Deactivated all sorts (no active sort in TextEditorState)");
        }
    }
}

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

/// Render the text editor sorts
pub fn render_text_editor_sorts(
    mut gizmos: Gizmos,
    text_editor_state: Res<TextEditorState>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    let _line_height = (font_metrics.ascender.unwrap_or(1024.0) - font_metrics.descender.unwrap_or(-256.0)) as f32;

    // Always render all sorts
    for (index, entry) in text_editor_state.buffer.iter().enumerate() {
        match &entry.kind {
            SortKind::Glyph { glyph_name, advance_width } => {
                if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
                    // Get the visual position for this sort
                    let position = if entry.is_buffer_root {
                        // Buffer roots use their exact stored position
                        entry.root_position
                    } else {
                        // Non-root text sorts flow from their text root
                        if let Some(flow_pos) = text_editor_state.get_text_sort_flow_position(index, &app_state.workspace.info.metrics, crate::ui::theme::LINE_LEADING) {
                            flow_pos
                        } else {
                            // Fallback to stored position
                            entry.root_position
                        }
                    };
                    let metrics_color = if entry.is_active {
                        SORT_ACTIVE_METRICS_COLOR
                    } else {
                        SORT_INACTIVE_METRICS_COLOR
                    };
                    render_sort_visuals(
                        &mut gizmos,
                        &glyph_data.outline,
                        *advance_width,
                        font_metrics,
                        position,
                        metrics_color,
                        SortRenderStyle::TextBuffer,
                    );
                }
            }
            SortKind::LineBreak => {
                // Line breaks don't render visually
            }
        }
    }

    // Only render the cursor in Insert mode
    if current_tool.get_current() != Some("text") ||
       current_placement_mode.0 != crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert {
        return;
    }

    // Render cursor for active buffer root
    if let Some((root_index, root_sort)) = text_editor_state.get_active_sort() {
        if root_sort.is_buffer_root {
            let root_pos = root_sort.root_position;
            let cursor_pos_in_text = root_sort.buffer_cursor_position.unwrap_or(0);
            let upm = font_metrics.units_per_em as f32;
            let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
            let line_height = upm - descender;
            let mut x = 0.0;
            let mut glyph_idx = 0;
            let mut line_number = 0;
            let mut cursor_x = 0.0;
            let mut cursor_line = 0;
            for entry in text_editor_state.buffer.iter().skip(root_index) {
                if glyph_idx == cursor_pos_in_text {
                    cursor_x = x;
                    cursor_line = line_number;
                    break;
                }
                match &entry.kind {
                    SortKind::Glyph { advance_width, .. } => {
                        x += *advance_width;
                        glyph_idx += 1;
                    }
                    SortKind::LineBreak => {
                        x = 0.0;
                        line_number += 1;
                        glyph_idx += 1;
                    }
                }
            }
            if glyph_idx == cursor_pos_in_text {
                cursor_x = x;
                cursor_line = line_number;
            }
            let baseline_y = root_pos.y + (cursor_line as f32) * -line_height;
            let cursor_x = root_pos.x + cursor_x;
            debug!(
                "[CURSOR DEBUG] line_number: {}, baseline_y: {:.1}, cursor_x: {:.1}, upm: {:.1}, descender: {:.1}, cursor_top_y: {:.1}, cursor_bottom_y: {:.1}",
                cursor_line, baseline_y, cursor_x, upm, descender, baseline_y + upm, baseline_y + descender
            );
            let cursor_color = SORT_ACTIVE_OUTLINE_COLOR;
            let circle_radius = 12.0;
            gizmos.line_2d(
                Vec2::new(cursor_x, baseline_y + descender),
                Vec2::new(cursor_x, baseline_y + upm),
                cursor_color,
            );
            gizmos.circle_2d(
                Vec2::new(cursor_x, baseline_y + upm),
                circle_radius,
                cursor_color,
            );
            gizmos.circle_2d(
                Vec2::new(cursor_x, baseline_y + descender),
                circle_radius,
                cursor_color,
            );
        }
    } else {
        // If no active sort, look for any buffer root with a cursor position
        for (index, entry) in text_editor_state.buffer.iter().enumerate() {
            if entry.is_buffer_root && entry.buffer_cursor_position.is_some() {
                let root_pos = entry.root_position;
                let cursor_pos_in_text = entry.buffer_cursor_position.unwrap_or(0);
                let upm = font_metrics.units_per_em as f32;
                let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
                let line_height = upm - descender;
                let mut x = 0.0;
                let mut glyph_idx = 0;
                let mut line_number = 0;
                let mut cursor_x = 0.0;
                let mut cursor_line = 0;
                for entry in text_editor_state.buffer.iter().skip(index) {
                    if glyph_idx == cursor_pos_in_text {
                        cursor_x = x;
                        cursor_line = line_number;
                        break;
                    }
                    match &entry.kind {
                        SortKind::Glyph { advance_width, .. } => {
                            x += *advance_width;
                            glyph_idx += 1;
                        }
                        SortKind::LineBreak => {
                            x = 0.0;
                            line_number += 1;
                            glyph_idx += 1;
                        }
                    }
                }
                if glyph_idx == cursor_pos_in_text {
                    cursor_x = x;
                    cursor_line = line_number;
                }
                let baseline_y = root_pos.y + (cursor_line as f32) * -line_height;
                let cursor_x = root_pos.x + cursor_x;
                debug!(
                    "[CURSOR DEBUG] line_number: {}, baseline_y: {:.1}, cursor_x: {:.1}, upm: {:.1}, descender: {:.1}, cursor_top_y: {:.1}, cursor_bottom_y: {:.1}",
                    cursor_line, baseline_y, cursor_x, upm, descender, baseline_y + upm, baseline_y + descender
                );
                let cursor_color = SORT_ACTIVE_OUTLINE_COLOR;
                let circle_radius = 12.0;
                gizmos.line_2d(
                    Vec2::new(cursor_x, baseline_y + descender),
                    Vec2::new(cursor_x, baseline_y + upm),
                    cursor_color,
                );
                gizmos.circle_2d(
                    Vec2::new(cursor_x, baseline_y + upm),
                    circle_radius,
                    cursor_color,
                );
                gizmos.circle_2d(
                    Vec2::new(cursor_x, baseline_y + descender),
                    circle_radius,
                    cursor_color,
                );
                break; // Only render cursor for the first buffer root with a cursor
            }
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
            text_editor_state.move_cursor_up_multiline();
            debug!("Insert mode: moved cursor up (multi-line)");
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down_multiline();
            debug!("Insert mode: moved cursor down (multi-line)");
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
        
        // Enter key - insert line break
        if keyboard_input.just_pressed(KeyCode::Enter) {
            text_editor_state.insert_line_break_at_cursor();
            info!("Insert mode: inserted line break (new line)");
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
                active_sort.kind.glyph_name(), 
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
                sort.kind.glyph_name(), 
                sort.is_active,
                sort.is_buffer_root,
                sort.buffer_cursor_position
            );
        }
    }
    
    // F2: Debug selection states
    if keyboard_input.just_pressed(KeyCode::F2) {
        info!("=== Selection Debug ===");
        // Remove all references to is_selected and buffer-based selection logic
        // for (index, sort) in text_editor_state.buffer.iter().enumerate() {
        //     if sort.is_selected || sort.is_active {
        //         info!(
        //             "Sort {}: '{}' - Selected: {}, Active: {}, Layout: {:?}", 
        //             index, 
        //             sort.kind.glyph_name(), 
        //             sort.is_selected, 
        //             sort.is_active,
        //             sort.layout_mode
        //         );
        //     }
        // }
        
        // if selected_sorts.is_empty() {
        //     info!("No sorts are currently selected");
        // }
    }
}

/// System to respawn all point entities when the active sort changes
pub fn respawn_active_sort_points(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    sort_point_entities: Query<(Entity, &SortPointEntity)>,
    app_state: Res<AppState>,
    mut sort_entities: Local<HashMap<(String, (i32, i32)), Entity>>, // (glyph_name, (x, y)) -> ECS entity
    mut last_active_sort: Local<Option<(usize, String, Vec2)>>, // Track last active sort to avoid respawning
) {
    // Find the active sort in TextEditorState
    let active_sort = text_editor_state.get_active_sort();
    
    // Check if the active sort has changed
    let current_active_sort = active_sort.map(|(index, sort_entry)| {
        (index, sort_entry.kind.glyph_name().to_string(), sort_entry.root_position)
    });
    
    // Only respawn if the active sort has actually changed
    if current_active_sort == *last_active_sort {
        return; // No change, skip respawning
    }
    
    info!("[respawn_active_sort_points] Called, active_sort={:?}, changed={:?}", 
          active_sort.is_some(), current_active_sort != *last_active_sort);

    // Despawn all existing point entities (for previous active sort)
    let existing_count = sort_point_entities.iter().count();
    if existing_count > 0 {
        info!("[respawn_active_sort_points] Despawning {} existing point entities", existing_count);
    }
    for (entity, _) in sort_point_entities.iter() {
        commands.entity(entity).despawn();
    }

    if let Some((index, sort_entry)) = active_sort {
        let glyph_name = sort_entry.kind.glyph_name().to_string();
        let root_position = sort_entry.root_position;
        info!("[respawn_active_sort_points] Active sort index: {}, glyph: '{}', root_position: {:?}", index, glyph_name, root_position);
        let position = sort_entry.root_position;
        let pos_key = (position.x.round() as i32, position.y.round() as i32);
        let key = (glyph_name.clone(), pos_key);
        // Get or create ECS entity for this sort
        let sort_entity = *sort_entities.entry(key.clone()).or_insert_with(|| {
            commands.spawn((
                crate::editing::sort::Sort::new(
                    glyph_name.clone(),
                    position,
                    sort_entry.kind.glyph_advance_width(),
                ),
                crate::editing::sort::ActiveSort,
                Transform::from_translation(position.extend(0.0)),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Selectable,
            )).id()
        });
        // Get glyph data for the active sort
        if let Some(glyph_data) = app_state.workspace.font.get_glyph(sort_entry.kind.glyph_name()) {
            if let Some(outline) = &glyph_data.outline {
                // Get the visual position of the sort
                if let Some(sort_world_pos) = text_editor_state.get_sort_visual_position(index) {
                    info!("[respawn_active_sort_points] Spawning points for sort '{}' at position {:?}", 
                          sort_entry.kind.glyph_name(), sort_world_pos);
                    let mut point_count = 0;
                    for (contour_index, contour) in outline.contours.iter().enumerate() {
                        for (point_index, point) in contour.points.iter().enumerate() {
                            let point_world_pos = sort_world_pos + Vec2::new(point.x as f32, point.y as f32);
                            point_count += 1;
                            
                            // Debug: Print first few point positions
                            if point_count <= 5 {
                                info!("[respawn_active_sort_points] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                                      point_count, point.x, point.y, point_world_pos.x, point_world_pos.y);
                            }
                            
                            commands.spawn((
                                EditPoint {
                                    position: Point::new(point.x, point.y),
                                    point_type: point.point_type, // Use our internal type directly
                                },
                                GlyphPointReference {
                                    glyph_name: sort_entry.kind.glyph_name().to_string(),
                                    contour_index,
                                    point_index,
                                },
                                PointType {
                                    is_on_curve: matches!(point.point_type, 
                                        crate::core::state::font_data::PointTypeData::Move | 
                                        crate::core::state::font_data::PointTypeData::Line |
                                        crate::core::state::font_data::PointTypeData::Curve),
                                },
                                Transform::from_translation(point_world_pos.extend(0.0)),
                                GlobalTransform::default(),
                                Visibility::Visible,
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                Selectable,
                                SortPointEntity { sort_entity },
                            ));
                        }
                    }
                    info!("[respawn_active_sort_points] Spawned {} point entities", point_count);
                } else {
                    warn!("[respawn_active_sort_points] No visual position found for sort at index {}", index);
                }
            } else {
                warn!("[respawn_active_sort_points] No outline found for glyph '{}'", sort_entry.kind.glyph_name());
            }
        } else {
            warn!("[respawn_active_sort_points] No glyph data found for '{}'", sort_entry.kind.glyph_name());
        }
    } else {
        info!("[respawn_active_sort_points] No active sort found");
    }
    
    // Update the last active sort tracking
    *last_active_sort = current_active_sort;
}

/// System to spawn ECS entities for sorts that don't have them yet
/// This ensures that every sort in the buffer has a corresponding ECS entity
pub fn spawn_missing_sort_entities(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    _app_state: Res<AppState>,
    mut sort_entities: Local<HashMap<(String, (i32, i32)), Entity>>, // (glyph_name, (x, y)) -> ECS entity
) {
    // Check all sorts in the buffer
    for (buffer_index, sort_entry) in text_editor_state.get_all_sorts() {
        if let Some(sort_pos) = text_editor_state.get_sort_visual_position(buffer_index) {
            let glyph_name = sort_entry.kind.glyph_name().to_string();
            let pos_key = (sort_pos.x.round() as i32, sort_pos.y.round() as i32);
            let key = (glyph_name.clone(), pos_key);
            
            // If we don't have an ECS entity for this sort, create one
            if !sort_entities.contains_key(&key) {
                let entity = commands.spawn((
                    crate::editing::sort::Sort::new(
                        glyph_name.clone(),
                        sort_pos,
                        sort_entry.kind.glyph_advance_width(),
                    ),
                    crate::editing::sort::InactiveSort, // Start as inactive
                    Transform::from_translation(sort_pos.extend(0.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Selectable,
                )).id();
                
                sort_entities.insert(key, entity);
                
                debug!("[spawn_missing_sort_entities] Created ECS entity {:?} for sort '{}' at buffer index {}", 
                       entity, glyph_name, buffer_index);
            }
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
    mut current_placement_mode: ResMut<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
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
                        // Don't return here - let the click detection system handle it
                        continue;
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
                            text_editor_state.create_text_sort_at_position(glyph_name.clone(), raw_sort_position, advance_width);
                            info!("Placed sort '{}' in buffer mode at position ({:.1}, {:.1}) with descender offset {:.1}", 
                                  glyph_name, raw_sort_position.x, raw_sort_position.y, descender);
                            // Automatically switch to Insert mode after placing a buffer sort
                            current_placement_mode.0 = crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert;
                        }
                        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Insert => {
                            info!("Insert mode: Use keyboard to edit buffer sorts, not mouse clicks");
                        }
                        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::Freeform => {
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
            text_editor_state.move_cursor_up_multiline();
            debug!("Unicode input: moved cursor up (multi-line)");
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down_multiline();
            debug!("Unicode input: moved cursor down (multi-line)");
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
        
        // Enter key - create new line
        if keyboard_input.just_pressed(KeyCode::Enter) {
            text_editor_state.create_new_line(&app_state.workspace.info.metrics);
            info!("Unicode input: created new line");
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

/// Optimized system to spawn point entities instantly when a sort becomes active
/// Uses Bevy's change detection for immediate response
pub fn spawn_active_sort_points_optimized(
    mut commands: Commands,
    // Detect when sorts become active (instant response)
    added_active_sorts: Query<(Entity, &crate::editing::sort::Sort), Added<crate::editing::sort::ActiveSort>>,
    app_state: Res<AppState>,
) {
    // Spawn points immediately for newly active sorts
    for (sort_entity, sort) in added_active_sorts.iter() {
        info!("[spawn_active_sort_points_optimized] INSTANT: Spawning points for newly active sort: '{}' at position {:?}", 
              sort.glyph_name, sort.position);
        
        // Get glyph data for the active sort
        if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
            if let Some(outline) = &glyph_data.outline {
                let mut point_count = 0;
                
                for (contour_index, contour) in outline.contours.iter().enumerate() {
                    for (point_index, point) in contour.points.iter().enumerate() {
                        // Calculate world position: sort position + point offset
                        let point_world_pos = sort.position + Vec2::new(point.x as f32, point.y as f32);
                        point_count += 1;
                        
                        // Debug: Print first few point positions
                        if point_count <= 5 {
                            info!("[spawn_active_sort_points_optimized] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                                  point_count, point.x, point.y, point_world_pos.x, point_world_pos.y);
                        }
                        
                        let glyph_point_ref = GlyphPointReference {
                            glyph_name: sort.glyph_name.clone(),
                            contour_index,
                            point_index,
                        };
                        
                        commands.spawn((
                            EditPoint {
                                position: Point::new(point.x, point.y),
                                point_type: point.point_type,
                            },
                            glyph_point_ref,
                            PointType {
                                is_on_curve: matches!(point.point_type, 
                                    crate::core::state::font_data::PointTypeData::Move | 
                                    crate::core::state::font_data::PointTypeData::Line |
                                    crate::core::state::font_data::PointTypeData::Curve),
                            },
                            Transform::from_translation(point_world_pos.extend(0.0)),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Selectable,
                            SortPointEntity { sort_entity },
                        ));
                    }
                }
                info!("[spawn_active_sort_points_optimized] INSTANT: Successfully spawned {} point entities", point_count);
            } else {
                warn!("[spawn_active_sort_points_optimized] No outline found for glyph '{}'", sort.glyph_name);
            }
        } else {
            warn!("[spawn_active_sort_points_optimized] No glyph data found for '{}'", sort.glyph_name);
        }
    }
}

/// Optimized system to despawn point entities instantly when a sort becomes inactive
/// Uses Bevy's change detection for immediate response
pub fn despawn_inactive_sort_points_optimized(
    mut commands: Commands,
    // Detect when sorts become inactive (instant response)
    mut removed_active_sorts: RemovedComponents<crate::editing::sort::ActiveSort>,
    point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Get all sort entities that just became inactive
    let inactive_sort_entities: Vec<Entity> = removed_active_sorts.read().collect();
    
    if !inactive_sort_entities.is_empty() {
        info!("[despawn_inactive_sort_points_optimized] INSTANT: Despawning points for {} inactive sorts", inactive_sort_entities.len());
    }
    
    // Despawn points for sorts that just became inactive
    for (point_entity, sort_point) in point_entities.iter() {
        if inactive_sort_entities.contains(&sort_point.sort_entity) {
            // Remove from selection state if selected
            if selection_state.selected.contains(&point_entity) {
                selection_state.selected.remove(&point_entity);
                info!("[despawn_inactive_sort_points_optimized] Removed despawned entity {:?} from selection", point_entity);
            }
            
            commands.entity(point_entity).despawn();
            debug!("[despawn_inactive_sort_points_optimized] INSTANT: Despawned point entity {:?} for inactive sort {:?}", 
                   point_entity, sort_point.sort_entity);
        }
    }
} 