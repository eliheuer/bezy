//! Text editor-based sort management system
//!
//! This system treats sorts as a text buffer that can be dynamically edited,
//! similar to how text editors work. Sorts are stored in a linear buffer
//! and mapped to a visual grid for display.

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

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
                // The world_pos is the baseline position (already offset correctly during placement)
                // No additional descender offset needed here since placement already handled it
                let sort_baseline_position = world_pos;
                
                // --- DEBUG MARKERS ---
                // 1. Blue X at baseline position
                let baseline_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::from(sort_baseline_position));
                let x_size = 18.0;
                gizmos.line_2d(
                    baseline_screen + Vec2::new(-x_size, -x_size),
                    baseline_screen + Vec2::new(x_size, x_size),
                    Color::srgb(0.2, 0.6, 1.0), // Blue
                );
                gizmos.line_2d(
                    baseline_screen + Vec2::new(-x_size, x_size),
                    baseline_screen + Vec2::new(x_size, -x_size),
                    Color::srgb(0.2, 0.6, 1.0), // Blue
                );

                // 2. Magenta diamond at glyph (0,0) (first point in first contour)
                if let Some(outline_data) = &glyph_data.outline {
                    if let Some(first_contour) = outline_data.contours.first() {
                        if let Some(first_point) = first_contour.points.first() {
                            let glyph_zero = sort_baseline_position + Vec2::new(first_point.x as f32, first_point.y as f32);
                            let zero_screen = viewport.to_screen(crate::ui::panes::design_space::DPoint::from(glyph_zero));
                            let d_size = 14.0;
                            gizmos.line_2d(
                                zero_screen + Vec2::new(0.0, -d_size),
                                zero_screen + Vec2::new(-d_size, 0.0),
                                Color::srgb(1.0, 0.0, 1.0), // Magenta
                            );
                            gizmos.line_2d(
                                zero_screen + Vec2::new(-d_size, 0.0),
                                zero_screen + Vec2::new(0.0, d_size),
                                Color::srgb(1.0, 0.0, 1.0),
                            );
                            gizmos.line_2d(
                                zero_screen + Vec2::new(0.0, d_size),
                                zero_screen + Vec2::new(d_size, 0.0),
                                Color::srgb(1.0, 0.0, 1.0),
                            );
                            gizmos.line_2d(
                                zero_screen + Vec2::new(d_size, 0.0),
                                zero_screen + Vec2::new(0.0, -d_size),
                                Color::srgb(1.0, 0.0, 1.0),
                            );
                            // Log the first point's coordinates
                            info!("DEBUG: Glyph '{}' first point at ({:.1}, {:.1})", sort.glyph_name, first_point.x, first_point.y);
                        }
                    }
                }
                
                // Convert to norad glyph for proper rendering
                let norad_glyph = glyph_data.to_norad_glyph();
                
                // Render proper metrics box at the baseline position
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
                    sort_baseline_position,
                    metrics_color,
                );
                
                // Then render the glyph outline at the baseline position
                if let Some(outline_data) = &glyph_data.outline {
                    if sort.is_active {
                        // Active sorts: full outline with control handles 
                        // and points
                        crate::rendering::glyph_outline::
                            draw_glyph_outline_at_position(
                                &mut gizmos,
                                &viewport,
                                outline_data,
                                sort_baseline_position,
                            );
                        
                        crate::rendering::glyph_outline::
                            draw_glyph_points_at_position(
                                &mut gizmos,
                                &viewport,
                                outline_data,
                                sort_baseline_position,
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
                                        sort_baseline_position,
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
            // The handle should be at the descender line relative to the baseline (world_pos)
            let descender = app_state.workspace.info.metrics.descender.unwrap() as f32;
            let handle_position = world_pos + Vec2::new(0.0, descender);
            
            debug!("Sort '{}' handle: sort_pos=({:.1}, {:.1}), handle=({:.1}, {:.1})", 
                   sort.glyph_name, world_pos.x, world_pos.y, handle_position.x, handle_position.y);
            
            // Determine handle colors based on state
            let (outer_color, inner_color, handle_size) = if sort.is_buffer_root {
                // Buffer root handles are larger and have special colors
                if sort.is_selected {
                    // BRIGHT, obvious selection colors for buffer roots
                    (Color::srgb(0.0, 1.0, 0.0), Color::srgb(1.0, 1.0, 1.0), 32.0) // Bright green with white center
                } else {
                    (Color::srgb(0.0, 0.6, 0.0), Color::srgb(0.4, 0.8, 0.4), 24.0) // Dark green
                }
            } else if sort.is_selected {
                // BRIGHT ORANGE selection colors for freeform sorts as requested
                (Color::srgb(1.0, 0.5, 0.0), Color::srgb(1.0, 0.7, 0.2), 24.0) // Bright orange outer, lighter orange inner
            } else if sort.is_active {
                // Active but not selected - blue
                (Color::srgb(0.0, 0.5, 1.0), Color::srgb(0.6, 0.8, 1.0), 20.0) // Blue
            } else {
                // Unselected handles are subtle
                (Color::srgb(0.6, 0.6, 0.6), Color::srgb(0.8, 0.8, 0.8), 16.0) // Gray
            };
            
            // Convert handle position to screen space (same as metrics)
            let handle_screen_pos = viewport.to_screen(
                crate::ui::panes::design_space::DPoint::from((handle_position.x, handle_position.y))
            );
            
            // Debug coordinate transformation when handle is selected
            if sort.is_selected {
                debug!(
                    "Handle coordinate transform: world=({:.1}, {:.1}) -> screen=({:.1}, {:.1})",
                    handle_position.x, handle_position.y, handle_screen_pos.x, handle_screen_pos.y
                );
            }
            
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
            
            // Add extra visual feedback for selected handles - pulsing ring
            if sort.is_selected {
                // Use orange for the selection ring to match the handle colors
                let ring_color = if sort.is_buffer_root {
                    Color::srgb(1.0, 1.0, 1.0).with_alpha(0.8) // White for buffer roots
                } else {
                    Color::srgb(1.0, 0.6, 0.1).with_alpha(0.8) // Orange for freeform sorts
                };
                
                gizmos.circle_2d(
                    handle_screen_pos,
                    handle_size + 8.0, // Larger outer ring
                    ring_color,
                );
            }
            
            // Draw buffer root indicator (small square) for buffer mode
            // Make it smaller and more subtle to reduce visual clutter
            if sort.is_buffer_root {
                gizmos.rect_2d(
                    handle_screen_pos,
                    Vec2::new(6.0, 6.0), // Slightly larger for better visibility
                    Color::srgb(1.0, 1.0, 1.0).with_alpha(0.9), // More opaque white square
                );
            }
            
            // Debug: Log handle state for troubleshooting
            if sort.is_selected || sort.is_active {
                debug!(
                    "Rendering handle for sort '{}': selected={}, active={}, position=({:.1}, {:.1})", 
                    sort.glyph_name, sort.is_selected, sort.is_active, handle_position.x, handle_position.y
                );
            }

            // 3. Green circle at handle position (already present, but make color distinctive)
            gizmos.circle_2d(
                handle_screen_pos,
                10.0,
                Color::srgb(0.0, 1.0, 0.0), // Bright green
            );
        }
    }
    
    // Render cursor for buffer mode (only show if we have buffer sorts)
    let text_sorts = text_editor_state.get_text_sorts();
    if !text_sorts.is_empty() {
        debug!("Rendering cursor: {} text sorts found", text_sorts.len());
        
        // Find the active text root and calculate cursor position within that text
        let cursor_world_pos = if let Some(active_text_root) = find_active_text_root(&text_editor_state) {
            let (root_index, root_sort) = active_text_root;
            let cursor_pos_in_buffer = root_sort.buffer_cursor_position.unwrap_or(0);
            
            debug!("Active text root '{}' at index {}, cursor position in text: {}", 
                   root_sort.glyph_name, root_index, cursor_pos_in_buffer);
            
            // Calculate position within this text sequence
            calculate_cursor_position_in_text(&text_editor_state, root_index, cursor_pos_in_buffer)
        } else {
            debug!("No active text root found, positioning cursor at zero");
            Vec2::ZERO
        };
        
        // Get font metrics for proper cursor height spanning
        let ascender = font_metrics.ascender.unwrap_or(800.0) as f32;
        let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
        
        // Convert cursor position from design space to screen space (like the handles and metrics)
        let cursor_top_design = cursor_world_pos + Vec2::new(0.0, ascender);
        let cursor_bottom_design = cursor_world_pos + Vec2::new(0.0, descender);
        
        let cursor_top_screen = viewport.to_screen(
            crate::ui::panes::design_space::DPoint::from((cursor_top_design.x, cursor_top_design.y))
        );
        let cursor_bottom_screen = viewport.to_screen(
            crate::ui::panes::design_space::DPoint::from((cursor_bottom_design.x, cursor_bottom_design.y))
        );
        
        // Draw a thicker cursor line using a rectangle for better visibility
        let line_thickness = 2.0;
        let line_center_x = (cursor_top_screen.x + cursor_bottom_screen.x) / 2.0;
        let line_center_y = (cursor_top_screen.y + cursor_bottom_screen.y) / 2.0;
        let line_height = (cursor_top_screen.y - cursor_bottom_screen.y).abs();
        
        gizmos.rect_2d(
            Vec2::new(line_center_x, line_center_y),
            Vec2::new(line_thickness, line_height),
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        
        // Draw larger outer circles (16px) for better visibility
        gizmos.circle_2d(
            cursor_top_screen,    // Top circle (at ascender)
            16.0,
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        gizmos.circle_2d(
            cursor_bottom_screen, // Bottom circle (at descender)
            16.0,
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        
        // Draw smaller inner circles (8px) on top
        gizmos.circle_2d(
            cursor_top_screen,    // Top circle (at ascender)
            8.0,
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        gizmos.circle_2d(
            cursor_bottom_screen, // Bottom circle (at descender)
            8.0,
            Color::srgb(1.0, 1.0, 0.0), // Yellow cursor
        );
        
        // Draw a circle indicator for the cursor position in screen space (for debugging)
        let cursor_baseline_screen = viewport.to_screen(
            crate::ui::panes::design_space::DPoint::from((cursor_world_pos.x, cursor_world_pos.y))
        );
        // Larger outer circle
        gizmos.circle_2d(
            cursor_baseline_screen, 
            16.0, 
            Color::srgb(1.0, 1.0, 0.0)
        );
        // Smaller inner circle
        gizmos.circle_2d(
            cursor_baseline_screen, 
            8.0, 
            Color::srgb(1.0, 1.0, 0.0)
        );
    } else {
        debug!("No cursor rendered: {} text sorts found", text_sorts.len());
    }
}

/// Handle keyboard input for text editing
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
            if text_editor_state.cursor_position > 0 {
                text_editor_state.move_cursor_left();
                text_editor_state.delete_sort_at_cursor();
                info!("Insert mode: backspaced sort at cursor position");
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
            KeyCode::KeyT => Some("t".to_string()), // Allow T for typing when not conflicting
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
        // Skip T key if pressed without modifiers to avoid conflict with tool shortcut
        if key == KeyCode::KeyT && !(keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) {
            continue;
        }
        
        if keyboard_input.just_pressed(key) {
            if let Some(glyph_name) = character_to_glyph(key) {
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
                            "Insert mode: created new buffer root and inserted glyph '{}' at center", 
                            glyph_name
                        );
                    } else {
                        // Buffer sorts exist, use normal insertion logic
                        text_editor_state.insert_sort_at_cursor(
                            glyph_name.clone(), 
                            advance_width
                        );
                        info!(
                            "Insert mode: inserted glyph '{}' at cursor position {}", 
                            glyph_name,
                            text_editor_state.cursor_position
                        );
                    }
                } else {
                    info!("Insert mode: glyph '{}' not found in font", glyph_name);
                }
            }
        }
    }
}

/// Find the currently active text root (selected or in insert mode)
fn find_active_text_root(text_editor_state: &TextEditorState) -> Option<(usize, &SortEntry)> {
    // FIXED: Use more robust logic to find active text root
    // First try to find a selected text root
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            if sort.is_buffer_root && sort.is_selected {
                return Some((i, sort));
            }
        }
    }
    
    // If no selected text root, look for any text root with a cursor position
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            if sort.is_buffer_root && sort.buffer_cursor_position.is_some() {
                return Some((i, sort));
            }
        }
    }
    
    // If still no text root found, look for the most recently added text root
    for i in (0..text_editor_state.buffer.len()).rev() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            if sort.is_buffer_root {
                return Some((i, sort));
            }
        }
    }
    
    None
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
            if root_sort.glyph_name.is_empty() {
                // Empty root - cursor at root position for replacement
                root_position
            } else {
                // Non-empty root - cursor at left edge
                root_position
            }
        } else {
            // Cursor is after one or more sorts - calculate cumulative x offset
            let mut x_offset = 0.0;
            
            // Sum up advance widths from the root up to the cursor position
            for i in 0..cursor_pos_in_text {
                let sort_index = root_index + i;
                if let Some(sort) = text_editor_state.buffer.get(sort_index) {
                    if sort.layout_mode == SortLayoutMode::Text && !sort.glyph_name.is_empty() {
                        x_offset += sort.advance_width;
                        debug!("Adding advance width {:.1} for sort '{}' at index {}", 
                               sort.advance_width, sort.glyph_name, sort_index);
                    }
                }
            }
            
            let cursor_pos = root_position + Vec2::new(x_offset, 0.0);
            debug!("Cursor position: root=({:.1}, {:.1}), offset={:.1}, final=({:.1}, {:.1})", 
                   root_position.x, root_position.y, x_offset, cursor_pos.x, cursor_pos.y);
            cursor_pos
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