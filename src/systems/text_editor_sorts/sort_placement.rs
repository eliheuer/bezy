//! Sort placement handling for text editor
//!
//! This module handles mouse-based sort placement when using text tools.

#![allow(clippy::too_many_arguments)]

use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use bevy::prelude::*;

/// Handle sort placement input (mouse clicks in text modes)
pub fn handle_sort_placement_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: ResMut<
        crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode,
    >,
    mut text_editor_state: ResMut<crate::core::state::TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    use crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode;

    // Only handle input when text tool is active
    let current_tool_name = current_tool.get_current();
    if current_tool_name != Some("text") {
        // Always log when we have a click so user knows what's happening
        if mouse_button_input.just_pressed(MouseButton::Left) && !ui_hover_state.is_hovering_ui {
            warn!("🖱️ SORT PLACEMENT: ❌ Click ignored - current tool is {:?}, need 'text' tool to place sorts", current_tool_name);
        }
        return;
    }
    
    info!("🖱️ SORT PLACEMENT: ✅ Text tool is active, checking other conditions...");

    // Only handle text placement modes, not insert mode
    match current_placement_mode.0 {
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // Continue with placement
            info!("🖱️ SORT PLACEMENT: ✅ Text tool active with placement mode {:?} - READY TO PLACE SORTS!", current_placement_mode.0);
        }
        TextPlacementMode::Insert | TextPlacementMode::Freeform => {
            // These modes don't place sorts on click
            if mouse_button_input.just_pressed(MouseButton::Left) && !ui_hover_state.is_hovering_ui {
                info!("🖱️ SORT PLACEMENT: Click ignored - in {:?} mode (not placement mode)", current_placement_mode.0);
            }
            return;
        }
    }

    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            info!("🖱️ SORT PLACEMENT: ⚠️  Click ignored - hovering over UI");
        }
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }
    
    info!("🖱️ SORT PLACEMENT: ✅ Left mouse click detected, UI not hovering");

    info!("🖱️ SORT PLACEMENT: Left mouse click detected - processing placement");

    // Get camera, transform, and projection
    let Ok((camera, camera_transform, projection)) = camera_query.single()
    else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(raw_world_position) =
        camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Get zoom scale from camera projection for grid snapping
    let zoom_scale = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };

    // Apply grid snapping to match the preview
    let grid_size = calculate_dynamic_grid_size(zoom_scale);
    let snapped_position = (raw_world_position / grid_size).round() * grid_size;

    // Always create a new sort when clicking in placement mode
    // This allows placing multiple sorts of the same glyph or different glyphs
    let existing_sorts_count = text_editor_state.get_text_sorts().len();
    info!("🖱️ SORT PLACEMENT: Creating new sort at position ({:.1}, {:.1}), existing sorts: {}", 
          snapped_position.x, snapped_position.y, existing_sorts_count);
    
    // CRITICAL FIX: Deactivate all existing sorts before creating new active sort
    // This prevents multiple active sorts from existing simultaneously
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get_mut(i) {
            if sort.is_active {
                info!("🔻 SORT PLACEMENT: Deactivating existing sort - glyph '{}'", sort.kind.glyph_name());
                sort.is_active = false;
            }
        }
    }
    
    // Create a new independent sort (not part of text buffer)
    info!("🖱️ SORT PLACEMENT: About to call create_independent_sort_with_fontir");
    create_independent_sort_with_fontir(
        &mut text_editor_state,
        snapped_position,
        current_placement_mode.0.to_sort_layout_mode(),
        fontir_app_state.as_deref(),
    );
    
    // CRITICAL: Mark the text editor state as changed to trigger entity spawning
    text_editor_state.set_changed();
    
    info!("🖱️ SORT PLACEMENT: create_independent_sort_with_fontir completed");

    info!("🖱️ SORT PLACEMENT: Created new sort, total sorts now: {}", 
          text_editor_state.get_text_sorts().len());
}

/// Create an independent sort that can coexist with other sorts
/// This is different from the text buffer system which creates connected text
fn create_independent_sort_with_fontir(
    text_editor_state: &mut crate::core::state::TextEditorState,
    world_position: bevy::math::Vec2,
    layout_mode: crate::core::state::text_editor::SortLayoutMode,
    fontir_app_state: Option<&crate::core::state::FontIRAppState>,
) {
    use crate::core::state::text_editor::{SortEntry, SortKind, SortLayoutMode};
    
    info!("🖱️ INSIDE create_independent_sort_with_fontir: Starting function");

    // Choose appropriate default glyph based on layout mode
    let (placeholder_glyph, placeholder_codepoint) = 
        crate::core::state::text_editor::editor::get_default_glyph_for_direction(&layout_mode);
    
    let advance_width = if let Some(fontir_state) = fontir_app_state {
        fontir_state.get_glyph_advance_width(&placeholder_glyph)
    } else {
        // Fallback to reasonable default if FontIR not available
        500.0
    };

    // CRITICAL FIX: Check if there's already a buffer root with the same layout mode
    // If so, add to that text flow instead of creating a new buffer root
    let mut existing_root_index = None;
    for (i, existing_sort) in text_editor_state.buffer.iter().enumerate() {
        if existing_sort.is_buffer_root && existing_sort.layout_mode == layout_mode {
            existing_root_index = Some(i);
            break;
        }
    }

    let (is_buffer_root, cursor_position) = if let Some(_root_index) = existing_root_index {
        // There's already a buffer root with this layout mode
        // Add this sort to that existing text flow instead of creating a new root
        info!("🖱️ Found existing {} buffer root, adding to existing text flow instead of creating new root", 
              match layout_mode {
                  SortLayoutMode::RTLText => "RTL",
                  SortLayoutMode::LTRText => "LTR", 
                  SortLayoutMode::Freeform => "Freeform",
              });
        (false, None) // Not a buffer root, no cursor position
    } else {
        // No existing buffer root with this layout mode, create a new one
        info!("🖱️ No existing {} buffer root found, creating new buffer root", 
              match layout_mode {
                  SortLayoutMode::RTLText => "RTL",
                  SortLayoutMode::LTRText => "LTR",
                  SortLayoutMode::Freeform => "Freeform", 
              });
        (true, Some(0)) // Is a buffer root with cursor at start
    };

    let independent_sort = SortEntry {
        kind: SortKind::Glyph {
            codepoint: Some(placeholder_codepoint), // Use appropriate codepoint for layout mode
            glyph_name: placeholder_glyph,
            advance_width, // Get from FontIR runtime data
        },
        is_active: true, // Automatically activate the new sort
        layout_mode, // Use the actual layout mode (RTL, LTR, etc.) not hardcoded Freeform
        root_position: world_position,
        is_buffer_root, // Only first sort in each layout mode becomes buffer root
        buffer_cursor_position: cursor_position, // Only buffer roots have cursor position
    };

    // Insert at the end of the buffer (this creates a new independent sort)  
    let insert_index = text_editor_state.buffer.len();
    info!("🖱️ Inserting independent sort at index {} into buffer with {} existing entries (is_buffer_root: {})", 
          insert_index, text_editor_state.buffer.len(), is_buffer_root);
    text_editor_state.buffer.insert(insert_index, independent_sort);
    info!("🖱️ Successfully inserted sort, buffer now has {} entries", text_editor_state.buffer.len());

    info!("🖱️ Created independent sort at world position ({:.1}, {:.1}), index: {}, is_buffer_root: {}", 
          world_position.x, world_position.y, insert_index, is_buffer_root);
}
