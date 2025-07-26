//! Sort placement handling for text editor
//!
//! This module handles mouse-based sort placement when using text tools.

use bevy::prelude::*;
use crate::rendering::checkerboard::calculate_dynamic_grid_size;

/// Handle sort placement input (mouse clicks in text modes)
pub fn handle_sort_placement_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut current_placement_mode: ResMut<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
    mut text_editor_state: ResMut<crate::core::state::TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    use crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode;

    // Only handle input when text tool is active
    if current_tool.get_current() != Some("text") {
        return;
    }

    // Only handle text placement modes, not insert mode
    match current_placement_mode.0 {
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // Continue with placement
        }
        TextPlacementMode::Insert | TextPlacementMode::Freeform => {
            // These modes don't place sorts on click
            return;
        }
    }

    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Get camera, transform, and projection
    let Ok((camera, camera_transform, projection)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(raw_world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
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

    // Check if we already have text sorts - if not, create a text root
    let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
    if !has_text_sorts {
        info!("Creating text root at snapped position ({:.1}, {:.1})", snapped_position.x, snapped_position.y);
        text_editor_state.create_text_root_with_fontir(
            snapped_position, 
            current_placement_mode.0.to_sort_layout_mode(),
            fontir_app_state.as_deref()
        );
        
        // Automatically switch to Insert mode after placing a text sort
        current_placement_mode.0 = TextPlacementMode::Insert;
        info!("Automatically switched to Insert mode for typing");
    }
}