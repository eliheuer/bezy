//! Text editor-based sort management system
//!
//! This system treats sorts as a text buffer that can be dynamically edited,
//! similar to how text editors work. Sorts are stored in a linear buffer
//! and mapped to a visual grid for display.

use crate::core::state::{AppState, TextEditorState, SortEntry, SortLayoutMode};
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
    
    let text_editor_state = 
        TextEditorState::from_font_data(&app_state.workspace.font);
    
    // Debug: Check how many buffer vs freeform sorts we have
    let buffer_sorts = text_editor_state.get_buffer_sorts();
    let freeform_sorts = text_editor_state.get_freeform_sorts();
    info!(
        "Initialized text editor with {} total sorts ({} buffer, {} freeform)", 
        app_state.workspace.font.glyphs.len(),
        buffer_sorts.len(),
        freeform_sorts.len()
    );
    
    commands.insert_resource(text_editor_state);
}

/// Handle mouse clicks on sorts in the text editor
pub fn handle_text_editor_sort_clicks(
    mut text_editor_state: ResMut<TextEditorState>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    ui_hover_state: Res<UiHoverState>,
    app_state: Res<crate::core::state::AppState>,
) {
    // Only handle clicks when not hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Check for left mouse button press
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = 
        camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    debug!(
        "Click at world position: ({:.1}, {:.1})", 
        world_position.x, 
        world_position.y
    );

    // Check for handle clicks first (more precise)
    let handle_tolerance = 30.0; // Smaller tolerance for precise handle clicks
    if let Some(clicked_sort_index) = text_editor_state.find_sort_at_position(
        world_position, 
        handle_tolerance, 
        Some(&app_state.workspace.info.metrics)
    ) {
        // Handle click - manage selection and active state relationship
        let is_ctrl_held = keyboard_input.pressed(KeyCode::ControlLeft) || 
                          keyboard_input.pressed(KeyCode::ControlRight);
        
        if is_ctrl_held {
            // Ctrl+click toggles selection without affecting other selections
            text_editor_state.toggle_sort_selection(clicked_sort_index);
            
            // Update active state based on selection count
            let selected_sorts = text_editor_state.get_selected_sorts();
            if selected_sorts.len() == 1 {
                // Only one sort selected → make it active
                let (selected_index, _) = selected_sorts[0];
                text_editor_state.activate_sort(selected_index);
            } else {
                // Multiple sorts selected → clear active state
                text_editor_state.clear_active_state();
            }
        } else {
            // Regular click: clear other selections, select this one, and make it active
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
        // Check for general sort area clicks (larger tolerance)
        let sort_tolerance = 250.0; 
        if let Some(clicked_sort_index) = text_editor_state.find_sort_at_position(
            world_position, 
            sort_tolerance, 
            Some(&app_state.workspace.info.metrics)
        ) {
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
            // Empty area click - clear selections and active state
            text_editor_state.clear_selections();
            text_editor_state.clear_active_state();
            
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
                debug!("Empty area click: cleared all selections and active state");
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
        debug!(
            "Rendering {} sorts from text editor buffer", 
            text_editor_state.buffer.len()
        );
    }
    
    // Render each sort in the buffer (both buffer and freeform)
    for buffer_position in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(buffer_position) {
            // Skip empty glyph names unless it's a buffer root (which we want to show)
            if sort.glyph_name.is_empty() && !sort.is_buffer_root {
                continue;
            }
            
            // Get the visual position based on the sort's layout mode
            let world_pos = match text_editor_state
                .get_sort_visual_position(buffer_position) {
                Some(pos) => pos,
                None => continue,
            };
            
            debug!(
                "Rendering sort {} '{}' at world pos ({:.1}, {:.1}) in {:?} mode", 
                buffer_position, 
                sort.glyph_name, 
                world_pos.x, 
                world_pos.y, 
                sort.layout_mode
            );
            
            // Handle empty buffer roots (show placeholder)
            if sort.glyph_name.is_empty() && sort.is_buffer_root {
                // Draw a placeholder rectangle for empty buffer root
                let placeholder_size = Vec2::new(50.0, 100.0);
                gizmos.rect_2d(
                    world_pos,
                    placeholder_size,
                    Color::srgba(0.5, 0.5, 0.5, 0.3), // Semi-transparent gray
                );
                
                                // Draw text "Empty" as indicator
                // Note: We'll skip glyph outline rendering for empty buffer roots
            } else if let Some(glyph_data) = 
                app_state.workspace.font.glyphs.get(&sort.glyph_name) {
                // Convert to norad glyph for proper rendering
                let norad_glyph = glyph_data.to_norad_glyph();
                
                // Render proper metrics box first
                let metrics_color = if sort.is_active { 
                    Color::srgba(0.3, 1.0, 0.5, 0.5) // Green for active
                } else {
                    Color::srgba(0.5, 0.5, 0.5, 0.5) // Gray for inactive
                };
                
                crate::rendering::metrics::draw_metrics_at_position_with_color(
                    &mut gizmos,
                    &viewport,
                    &norad_glyph,
                    font_metrics,
                    world_pos,
                    metrics_color,
                );
                
                // Then render the glyph outline properly using our internal 
                // outline data
                if let Some(outline_data) = &glyph_data.outline {
                    if sort.is_active {
                        // Active sorts: full outline with control handles 
                        // and points
                        crate::rendering::glyph_outline::
                            draw_glyph_outline_at_position(
                                &mut gizmos,
                                &viewport,
                                outline_data,
                                world_pos,
                            );
                        
                        crate::rendering::glyph_outline::
                            draw_glyph_points_at_position(
                                &mut gizmos,
                                &viewport,
                                outline_data,
                                world_pos,
                            );
                    } else {
                        // Inactive sorts: just the outline path 
                        // (no control handles)
                        for contour in &outline_data.contours {
                            if !contour.points.is_empty() {
                                crate::rendering::glyph_outline::
                                    draw_contour_path_at_position(
                                        &mut gizmos,
                                        &viewport,
                                        contour,
                                        world_pos,
                                    );
                            }
                        }
                    }
                } else {
                    debug!(
                        "Glyph '{}' has no outline data", 
                        sort.glyph_name
                    );
                }
            } else if !sort.glyph_name.is_empty() {
                debug!(
                    "Glyph '{}' not found in font data", 
                    sort.glyph_name
                );
            }
            // Note: Empty buffer roots are handled above and don't need glyph data
            
            // Draw handles for all sorts (regardless of glyph data)
            let descender = app_state.workspace.info.metrics
                .descender.unwrap_or(-200.0) as f32;
            let handle_position = world_pos + Vec2::new(0.0, descender);
            
            // Determine handle colors based on state
            let (outer_color, inner_color, handle_size) = if sort.is_buffer_root {
                // Buffer root handles are larger and have special colors
                if sort.is_selected {
                    (Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.8, 1.0, 0.8), 28.0) // Green
                } else {
                    (Color::srgb(0.0, 0.8, 0.0), Color::srgb(0.6, 0.8, 0.6), 24.0) // Dark green
                }
            } else if sort.is_selected {
                // Selected handles are highlighted
                (Color::srgb(1.0, 0.5, 0.0), Color::srgb(1.0, 0.8, 0.4), 20.0) // Orange
            } else {
                // Unselected handles are subtle
                (Color::srgb(0.6, 0.6, 0.6), Color::srgb(0.8, 0.8, 0.8), 16.0) // Gray
            };
            
            // Draw the main handle circle
            gizmos.circle_2d(
                handle_position,
                handle_size,
                outer_color,
            );
            
            // Draw the inner circle for visual clarity
            gizmos.circle_2d(
                handle_position,
                handle_size * 0.6,
                inner_color,
            );
            
            // Draw buffer root indicator (small square)
            if sort.is_buffer_root {
                gizmos.rect_2d(
                    handle_position,
                    Vec2::new(8.0, 8.0),
                    Color::srgb(1.0, 1.0, 1.0), // White square
                );
            }
        }
    }
    
    // Render cursor for buffer mode (only show if we have buffer sorts)
    let buffer_sorts = text_editor_state.get_buffer_sorts();
    if !buffer_sorts.is_empty() {
        debug!("Rendering cursor: {} buffer sorts found", buffer_sorts.len());
        
        // Calculate cursor position based on buffer text flow
        let cursor_world_pos = if text_editor_state.cursor_position < text_editor_state.buffer.len() {
            // Cursor is between sorts - use the position of the sort at cursor
            text_editor_state.get_sort_visual_position(text_editor_state.cursor_position)
                .unwrap_or(Vec2::ZERO)
        } else if text_editor_state.cursor_position > 0 {
            // Cursor is at end - position after last buffer sort
            if let Some(last_sort_pos) = text_editor_state.get_sort_visual_position(text_editor_state.cursor_position - 1) {
                if let Some(last_sort) = text_editor_state.buffer.get(text_editor_state.cursor_position - 1) {
                    if last_sort.layout_mode == SortLayoutMode::Buffer {
                        last_sort_pos + Vec2::new(last_sort.advance_width, 0.0)
                    } else {
                        last_sort_pos
                    }
                } else {
                    last_sort_pos
                }
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };
        
        // Draw a blinking cursor line at the insertion point
        gizmos.line_2d(
            cursor_world_pos + Vec2::new(-5.0, 400.0),  // Top of cursor
            cursor_world_pos + Vec2::new(-5.0, -200.0), // Bottom of cursor
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        
        // Draw a small circle indicator for the cursor position 
        // (for debugging)
        gizmos.circle_2d(
            cursor_world_pos + Vec2::new(0.0, 20.0), 
            6.0, 
            Color::srgb(1.0, 1.0, 0.0)
        );
    } else {
        debug!("No cursor rendered: {} buffer sorts found", buffer_sorts.len());
    }
}

/// Handle keyboard input for text editing
pub fn handle_text_editor_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
) {
    // Check if we have buffer sorts for various operations
    let has_buffer_sorts = !text_editor_state.get_buffer_sorts().is_empty();
    
    // Move cursor with arrow keys - only work if we have buffer sorts
    if has_buffer_sorts {
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
    }
    
    // Ctrl+T to create a new text buffer
    if keyboard_input.just_pressed(KeyCode::KeyT) && 
       (keyboard_input.pressed(KeyCode::ControlLeft) || 
        keyboard_input.pressed(KeyCode::ControlRight)) {
        // For now, create buffer root at center of screen
        // TODO: Use actual mouse/click position from text tool
        text_editor_state.create_buffer_root(Vec2::new(500.0, 0.0));
        info!("Created new text buffer");
    }
    
    // Delete/Backspace - only work if we have buffer sorts
    if has_buffer_sorts {
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
        KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, 
        KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, 
        KeyCode::KeyI, KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL, 
        KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP, 
        KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
        KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, 
        KeyCode::KeyY, KeyCode::KeyZ, KeyCode::Digit1, KeyCode::Digit2, 
        KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, 
        KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, 
        KeyCode::Digit9, KeyCode::Digit0, 
        KeyCode::Space,
    ] {
        if keyboard_input.just_pressed(key) {
            if let Some(glyph_name) = character_to_glyph(key) {
                // Check if the glyph exists in the font
                if let Some(glyph_data) = 
                    app_state.workspace.font.glyphs.get(&glyph_name) {
                    let advance_width = glyph_data.advance_width as f32;
                    
                    // Check if any buffer sorts exist
                    if !has_buffer_sorts {
                        // No buffer sorts exist, create a buffer root at center of screen
                        // TODO: Use actual mouse/click position if available
                        let center_position = Vec2::new(500.0, 0.0);
                        text_editor_state.create_buffer_root(center_position);
                        
                        // Now insert the character at the new buffer root
                        text_editor_state.insert_sort_at_cursor(
                            glyph_name.clone(), 
                            advance_width
                        );
                        
                        info!(
                            "Created new buffer root and inserted glyph '{}' at center", 
                            glyph_name
                        );
                    } else {
                        // Buffer sorts exist, use normal insertion logic
                        text_editor_state.insert_sort_at_cursor(
                            glyph_name.clone(), 
                            advance_width
                        );
                        info!(
                            "Inserted glyph '{}' at cursor position", 
                            glyph_name
                        );
                    }
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
                "Sort {}: '{}' (active: {})", 
                i, 
                sort.glyph_name, 
                sort.is_active
            );
        }
    }
} 