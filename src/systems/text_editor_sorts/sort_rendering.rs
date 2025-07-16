//! Sort rendering for text editor sorts

use crate::core::state::text_editor::{SortKind, SortLayoutMode};
use crate::core::state::{AppState, TextEditorState};
use crate::ui::theme::*;
use crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode;
use crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode;
use bevy::prelude::*;

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
        debug!(
            "Cursor not rendered: text tool not active (current: {:?})",
            current_tool.get_current()
        );
        return;
    }

    // Show cursor in all text modes for better UX
    debug!("Cursor mode: {:?}", current_placement_mode.0);

    debug!(
        "Cursor system running: text tool active, mode: {:?}",
        current_placement_mode.0
    );

    let Some(text_editor_state) = text_editor_state else {
        return;
    };

    // Calculate cursor visual position
    if let Some(cursor_world_pos) =
        calculate_cursor_visual_position(&text_editor_state, &app_state)
    {
        // Get font metrics for proper cursor height
        let font_metrics = &app_state.workspace.info.metrics;

        // Calculate cursor bounds based on font metrics
        let cursor_top = cursor_world_pos.y + font_metrics.units_per_em as f32; // UPM top
        let cursor_bottom = cursor_world_pos.y
            + font_metrics.descender.unwrap_or(-256.0) as f32; // Descender bottom
        let cursor_height = cursor_top - cursor_bottom;

        // Bright orange cursor color (like pre-refactor)
        let cursor_color = Color::srgb(1.0, 0.5, 0.0); // Bright orange

        // Draw thick cursor spanning full sort metrics height
        for offset in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            gizmos.line_2d(
                Vec2::new(cursor_world_pos.x + offset, cursor_bottom),
                Vec2::new(cursor_world_pos.x + offset, cursor_top),
                cursor_color,
            );
        }

        // Add layered circles at top and bottom of cursor
        let large_circle_radius = 8.0; // 4x the medium (2.0)
        let medium_circle_radius = 2.0; // Width of outer vertical lines
        let small_circle_radius = 0.5; // Width of inner lines

        // Draw circles at top
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_top),
            large_circle_radius,
            cursor_color,
        );
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_top),
            medium_circle_radius,
            cursor_color,
        );
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_top),
            small_circle_radius,
            cursor_color,
        );

        // Draw circles at bottom
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_bottom),
            large_circle_radius,
            cursor_color,
        );
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_bottom),
            medium_circle_radius,
            cursor_color,
        );
        gizmos.circle_2d(
            Vec2::new(cursor_world_pos.x, cursor_bottom),
            small_circle_radius,
            cursor_color,
        );

        debug!(
            "Text cursor rendered at ({:.1}, {:.1}), height: {:.1}",
            cursor_world_pos.x, cursor_world_pos.y, cursor_height
        );
    }
}

/// Calculate the visual world position of the cursor based on text buffer state
fn calculate_cursor_visual_position(
    text_editor_state: &TextEditorState,
    app_state: &AppState,
) -> Option<Vec2> {
    // Find the active buffer root
    let mut active_root_index = None;
    let mut cursor_pos_in_buffer = 0;
    let mut root_position = Vec2::ZERO;

    // Look for the active buffer root and get its cursor position
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            if sort.is_buffer_root && sort.is_active {
                active_root_index = Some(i);
                cursor_pos_in_buffer = sort.buffer_cursor_position.unwrap_or(0);
                root_position = sort.root_position;
                break;
            }
        }
    }

    // If no active root found, check for any buffer root with cursor position
    if active_root_index.is_none() {
        for i in 0..text_editor_state.buffer.len() {
            if let Some(sort) = text_editor_state.buffer.get(i) {
                if sort.is_buffer_root && sort.buffer_cursor_position.is_some()
                {
                    active_root_index = Some(i);
                    cursor_pos_in_buffer =
                        sort.buffer_cursor_position.unwrap_or(0);
                    root_position = sort.root_position;
                    break;
                }
            }
        }
    }

    let root_index = active_root_index?;

    // If cursor at position 0, place at root position
    if cursor_pos_in_buffer == 0 {
        return Some(root_position);
    }

    // Calculate position based on the glyphs in the buffer sequence, handling line breaks
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;
    let mut glyph_count = 0;

    // Get font metrics for line height calculation
    let font_metrics = &app_state.workspace.info.metrics;
    let upm = font_metrics.units_per_em as f32;
    let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
    let line_height = upm - descender;

    // Start from the root and accumulate advances
    for i in root_index..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get(i) {
            // Stop if we hit another buffer root
            if i != root_index && sort.is_buffer_root {
                break;
            }

            // Count glyphs in this buffer sequence
            if i == root_index || sort.layout_mode == SortLayoutMode::Text {
                // If we've reached the cursor position, return
                if glyph_count == cursor_pos_in_buffer {
                    return Some(Vec2::new(
                        root_position.x + x_offset,
                        root_position.y + y_offset,
                    ));
                }

                // Handle different sort types
                match &sort.kind {
                    SortKind::LineBreak => {
                        // Line break: reset x_offset and move down a line
                        x_offset = 0.0;
                        y_offset -= line_height;
                    }
                    SortKind::Glyph {
                        glyph_name: _,
                        advance_width,
                    } => {
                        if glyph_count < cursor_pos_in_buffer {
                            x_offset += advance_width;
                        }
                    }
                }

                glyph_count += 1;
            }
        }
    }

    // Cursor is at or beyond the end, position after last glyph
    Some(Vec2::new(
        root_position.x + x_offset,
        root_position.y + y_offset,
    ))
}
