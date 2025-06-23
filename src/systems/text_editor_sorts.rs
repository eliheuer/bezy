//! Text editor-based sort management system
//!
//! This system treats sorts as a text buffer that can be dynamically edited,
//! similar to how text editors work. Sorts are stored in a linear buffer
//! and mapped to a visual grid for display.

use crate::core::state::{AppState, TextEditorState, SortEntry};
use crate::rendering::cameras::DesignCamera;
use crate::systems::ui_interaction::UiHoverState;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Initialize the text editor state when a font is loaded
pub fn initialize_text_editor_sorts(
    mut commands: Commands,
    app_state: Res<AppState>,
    text_editor_state: Option<Res<TextEditorState>>,
) {
    // Only initialize if we don't already have a text editor state
    if text_editor_state.is_some() {
        return;
    }
    
    // Only initialize if we have font data
    if app_state.workspace.font.glyphs.is_empty() {
        return;
    }
    
    let text_editor_state = TextEditorState::from_font_data(&app_state.workspace.font);
    commands.insert_resource(text_editor_state);
    
    info!("Initialized text editor with {} sorts", app_state.workspace.font.glyphs.len());
}

/// Handle mouse clicks on sorts in the text editor
pub fn handle_text_editor_sort_clicks(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    mut text_editor_state: ResMut<TextEditorState>,
    ui_hover_state: Res<UiHoverState>,
) {
    // Only handle left mouse button clicks
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    // Don't handle clicks if hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }
    
    // Get the primary window
    let window = match window_query.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };
    
    // Get cursor position in window
    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    
    // Get camera and transform
    let (camera, camera_transform) = match camera_query.get_single() {
        Ok((camera, transform)) => (camera, transform),
        Err(_) => return,
    };
    
    // Convert cursor position to world coordinates
    let world_position = match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        Ok(pos) => pos,
        Err(_) => return,
    };
    
    info!("Click at world position: ({:.1}, {:.1})", world_position.x, world_position.y);
    
    // Find which sort was clicked based on world position
    if let Some(buffer_position) = text_editor_state.get_buffer_position_for_world_position(world_position) {
        // Debug: calculate expected grid position
        let relative_pos = world_position - text_editor_state.grid_config.grid_origin;
        let expected_col = (relative_pos.x / (600.0 + text_editor_state.grid_config.horizontal_spacing)).floor() as usize;
        let expected_row = if relative_pos.y <= 0.0 {
            ((-relative_pos.y) / (1000.0 + text_editor_state.grid_config.vertical_spacing)).floor() as usize
        } else {
            0
        };
        info!("Calculated grid position: row={}, col={}, buffer_position={}", expected_row, expected_col, buffer_position);
        if let Some(sort) = text_editor_state.get_sort_at_position(buffer_position) {
            let glyph_name = sort.glyph_name.clone(); // Clone to avoid borrow checker issues
            info!("Clicked on sort '{}' at buffer position {}", glyph_name, buffer_position);
            
            // Activate the clicked sort
            text_editor_state.activate_sort_at_position(buffer_position);
            
            info!("Activated sort '{}' at buffer position {}", glyph_name, buffer_position);
        }
    } else {
        info!("Click at ({:.1}, {:.1}) did not hit any sort", world_position.x, world_position.y);
        
        // Deactivate all sorts if clicking in empty space
        for i in 0..text_editor_state.buffer.len() {
            if let Some(sort) = text_editor_state.buffer.get_mut(i) {
                sort.is_active = false;
            }
        }
    }
}

/// Render the text editor sorts
pub fn render_text_editor_sorts(
    mut gizmos: Gizmos,
    text_editor_state: Res<TextEditorState>,
    app_state: Res<AppState>,
    viewport: Res<crate::ui::panes::design_space::ViewPort>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    
    // Debug: Log rendering info
    if text_editor_state.buffer.len() > 0 {
        debug!("Rendering {} sorts from text editor buffer", text_editor_state.buffer.len());
    }
    
    // Render each sort in the buffer
    for (buffer_position, sort) in text_editor_state.buffer.iter().enumerate() {
        // Skip empty glyph names (default entries in gap buffer)
        if sort.glyph_name.is_empty() {
            continue;
        }
        
        let world_pos = text_editor_state.get_world_position_for_buffer_position(buffer_position);
        
        // Debug: Log first few sorts being rendered
        if buffer_position < 5 {
            debug!("Rendering sort {} '{}' at world position ({:.1}, {:.1})", 
                   buffer_position, sort.glyph_name, world_pos.x, world_pos.y);
        }
        
        // Get the glyph data from the app state
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&sort.glyph_name) {
            // Convert to norad glyph for metrics rendering
            let norad_glyph = glyph_data.to_norad_glyph();
            
            // Choose colors based on active state
            let metrics_color = if sort.is_active {
                crate::ui::theme::SORT_ACTIVE_OUTLINE_COLOR // Orange for active
            } else {
                crate::ui::theme::SORT_INACTIVE_OUTLINE_COLOR // Gray for inactive
            };
            
            // Render the metrics box
            crate::rendering::metrics::draw_metrics_at_position_with_color(
                &mut gizmos,
                &viewport,
                &norad_glyph,
                font_metrics,
                world_pos,
                metrics_color,
            );
            
            // Render the glyph outline if it exists
            if let Some(outline_data) = &glyph_data.outline {
                if sort.is_active {
                    // For active sorts, draw full outline with control handles and points
                    crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                        &mut gizmos,
                        &viewport,
                        outline_data,
                        world_pos,
                    );
                    
                    crate::rendering::glyph_outline::draw_glyph_points_at_position(
                        &mut gizmos,
                        &viewport,
                        outline_data,
                        world_pos,
                    );
                } else {
                    // For inactive sorts, draw only the path outline
                    for contour_data in outline_data.contours.iter() {
                        crate::rendering::glyph_outline::draw_contour_path_at_position(
                            &mut gizmos,
                            &viewport,
                            contour_data,
                            world_pos,
                        );
                    }
                }
            } else {
                // Debug: Log glyphs without outlines
                if buffer_position < 5 {
                    debug!("Glyph '{}' has no outline data", sort.glyph_name);
                }
            }
        } else {
            // Debug: Log missing glyphs
            if buffer_position < 5 {
                warn!("Glyph '{}' not found in font data", sort.glyph_name);
            }
        }
        
        // Draw a small indicator for the buffer position (for debugging)
        if buffer_position < 10 {
            gizmos.circle_2d(world_pos + Vec2::new(10.0, 10.0), 5.0, Color::srgb(1.0, 0.0, 0.0));
        }
    }
    
    // Draw cursor position indicator
    let cursor_world_pos = text_editor_state.get_world_position_for_buffer_position(text_editor_state.cursor_position);
    
    // Draw a blinking cursor line at the insertion point
    gizmos.line_2d(
        cursor_world_pos + Vec2::new(-5.0, 400.0),  // Top of cursor line
        cursor_world_pos + Vec2::new(-5.0, -200.0), // Bottom of cursor line
        Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
    );
    
    // Draw a small circle indicator for the cursor position (for debugging)
    gizmos.circle_2d(cursor_world_pos + Vec2::new(0.0, 20.0), 6.0, Color::srgb(1.0, 1.0, 0.0));
}

/// Handle keyboard input for text editing
pub fn handle_text_editor_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
) {
    // Move cursor with arrow keys
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        text_editor_state.move_cursor_right();
    }
    
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        text_editor_state.move_cursor_left();
    }
    
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        text_editor_state.move_cursor_up();
    }
    
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        text_editor_state.move_cursor_down();
    }
    
    // Home/End keys
    if keyboard_input.just_pressed(KeyCode::Home) {
        text_editor_state.move_cursor_to(0);
        info!("Moved cursor to beginning");
    }
    
    if keyboard_input.just_pressed(KeyCode::End) {
        let buffer_len = text_editor_state.buffer.len();
        text_editor_state.move_cursor_to(buffer_len);
        info!("Moved cursor to end");
    }
    
    // Delete/Backspace
    if keyboard_input.just_pressed(KeyCode::Delete) {
        text_editor_state.delete_sort_at_cursor();
        info!("Deleted sort at cursor position");
    }
    
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        if text_editor_state.cursor_position > 0 {
            text_editor_state.move_cursor_left();
            text_editor_state.delete_sort_at_cursor();
            info!("Backspaced sort at cursor position");
        }
    }
    
    // Character input for typing new sorts
    // Map common characters to their glyph names
    let character_to_glyph = |key: KeyCode| -> Option<String> {
        match key {
            KeyCode::KeyA => Some("a".to_string()),
            KeyCode::KeyB => Some("b".to_string()),
            KeyCode::KeyC => Some("c".to_string()),
            KeyCode::KeyD => Some("d".to_string()),
            KeyCode::KeyE => Some("e".to_string()),
            KeyCode::KeyF => Some("f".to_string()),
            KeyCode::KeyG => Some("g".to_string()),
            KeyCode::KeyH => Some("h".to_string()),
            KeyCode::KeyI => Some("i".to_string()),
            KeyCode::KeyJ => Some("j".to_string()),
            KeyCode::KeyK => Some("k".to_string()),
            KeyCode::KeyL => Some("l".to_string()),
            KeyCode::KeyM => Some("m".to_string()),
            KeyCode::KeyN => Some("n".to_string()),
            KeyCode::KeyO => Some("o".to_string()),
            KeyCode::KeyP => Some("p".to_string()),
            KeyCode::KeyQ => Some("q".to_string()),
            KeyCode::KeyR => Some("r".to_string()),
            KeyCode::KeyS => Some("s".to_string()),
            KeyCode::KeyT => Some("t".to_string()),
            KeyCode::KeyU => Some("u".to_string()),
            KeyCode::KeyV => Some("v".to_string()),
            KeyCode::KeyW => Some("w".to_string()),
            KeyCode::KeyX => Some("x".to_string()),
            KeyCode::KeyY => Some("y".to_string()),
            KeyCode::KeyZ => Some("z".to_string()),
            KeyCode::Digit1 => Some("one".to_string()),
            KeyCode::Digit2 => Some("two".to_string()),
            KeyCode::Digit3 => Some("three".to_string()),
            KeyCode::Digit4 => Some("four".to_string()),
            KeyCode::Digit5 => Some("five".to_string()),
            KeyCode::Digit6 => Some("six".to_string()),
            KeyCode::Digit7 => Some("seven".to_string()),
            KeyCode::Digit8 => Some("eight".to_string()),
            KeyCode::Digit9 => Some("nine".to_string()),
            KeyCode::Digit0 => Some("zero".to_string()),
            KeyCode::Space => Some("space".to_string()),
            _ => None,
        }
    };
    
    // Handle character input
    for key in [
        KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE,
        KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
        KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO,
        KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
        KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY,
        KeyCode::KeyZ, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
        KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
        KeyCode::Digit0, KeyCode::Space,
    ] {
        if keyboard_input.just_pressed(key) {
            if let Some(glyph_name) = character_to_glyph(key) {
                // Check if the glyph exists in the font
                if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&glyph_name) {
                    let advance_width = glyph_data.advance_width as f32;
                    text_editor_state.insert_sort_at_cursor(glyph_name.clone(), advance_width);
                    info!("Inserted glyph '{}' at cursor position", glyph_name);
                } else {
                    info!("Glyph '{}' not found in font", glyph_name);
                }
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
        
        if let Some((active_pos, active_sort)) = text_editor_state.get_active_sort() {
            info!("Active sort: '{}' at position {}", active_sort.glyph_name, active_pos);
        } else {
            info!("No active sort");
        }
        
        // Log first few sorts
        for (i, sort) in text_editor_state.buffer.iter().take(5).enumerate() {
            info!("Sort {}: '{}' (active: {})", i, sort.glyph_name, sort.is_active);
        }
    }
} 