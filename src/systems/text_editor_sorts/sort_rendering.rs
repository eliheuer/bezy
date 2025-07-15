//! Sort rendering for text editor sorts

use bevy::prelude::*;
use crate::core::state::{TextEditorState, AppState};
use crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode;
use crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode;
use crate::ui::theme::*;

/// Render text editor sorts
pub fn render_text_editor_sorts() {
    // TODO: Implement text editor sorts rendering
}

/// Render the visual cursor for Insert mode
pub fn render_text_editor_cursor(
    mut gizmos: Gizmos,
    text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    app_state: Res<AppState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Only render cursor when text tool is active and in Insert mode
    if current_tool.get_current() != Some("text") {
        debug!("Cursor not rendered: text tool not active (current: {:?})", current_tool.get_current());
        return;
    }
    
    // Show cursor in all text modes for better UX
    debug!("Cursor mode: {:?}", current_placement_mode.0);
    
    debug!("Cursor system running: text tool active, mode: {:?}", current_placement_mode.0);
    
    let Some(text_editor_state) = text_editor_state else {
        return;
    };
    
    // Calculate cursor visual position
    if let Some(cursor_world_pos) = calculate_cursor_visual_position(&text_editor_state, &app_state) {
        // Draw cursor as a vertical line - make it more prominent
        let cursor_height = 48.0; // Taller cursor line for visibility
        let cursor_color = Color::srgb(1.0, 0.2, 0.2); // Bright red cursor
        
        gizmos.line_2d(
            Vec2::new(cursor_world_pos.x, cursor_world_pos.y - cursor_height / 2.0),
            Vec2::new(cursor_world_pos.x, cursor_world_pos.y + cursor_height / 2.0),
            cursor_color,
        );
        
        // Add small indicator at bottom
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_world_pos.y - cursor_height / 2.0),
            2.0,
            cursor_color,
        );
        
        debug!("Text cursor rendered at ({:.1}, {:.1})", cursor_world_pos.x, cursor_world_pos.y);
    }
}

/// Calculate the visual world position of the cursor based on text buffer state
fn calculate_cursor_visual_position(
    text_editor_state: &TextEditorState,
    app_state: &AppState,
) -> Option<Vec2> {
    let cursor_pos = text_editor_state.cursor_position;
    
    // If cursor is at position 0, place it at the start of text flow
    if cursor_pos == 0 {
        // Use the text mode origin
        return Some(Vec2::new(0.0, 0.0)); // Start at origin for now
    }
    
    // Calculate position based on preceding sorts
    let mut x_offset = 0.0;
    let y_position = 0.0; // Single line for now
    
    // Sum up advance widths of all sorts before cursor position
    for i in 0..cursor_pos.min(text_editor_state.buffer.len()) {
        if let Some(sort_entry) = text_editor_state.buffer.get(i) {
            let glyph_name = sort_entry.kind.glyph_name();
            
            // Get advance width from font data
            let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
                glyph_data.advance_width as f32
            } else {
                600.0 // Default width
            };
            
            x_offset += advance_width;
        }
    }
    
    Some(Vec2::new(x_offset, y_position))
}
