//! Keyboard input handling for text editor sorts

use bevy::prelude::*;

/// Handle text editor keyboard input
pub fn handle_text_editor_keyboard_input() {
    // TODO: Implement keyboard input handling
}

/// Handle Arabic text input
pub fn handle_arabic_text_input() {
    // TODO: Implement Arabic text input handling
}

/// Legacy placeholder - unicode text input is now handled in unicode_input.rs
pub fn handle_unicode_text_input_legacy() {
    // This function is kept for compatibility but replaced by unicode_input.rs
}

/// Handle sort placement input (mouse clicks in text modes)
pub fn handle_sort_placement_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>,
    mut text_editor_state: ResMut<crate::core::state::TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    // Only handle input when text tool is active
    if current_tool.get_current() != Some("text") {
        return;
    }

    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Only handle LTR and RTL text modes (not Insert or Freeform)
    if !matches!(current_placement_mode.0, 
        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::LTRText | 
        crate::ui::toolbars::edit_mode_toolbar::text::TextPlacementMode::RTLText) {
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor position in world coordinates
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // Check if we already have text sorts - if not, create a text root
    let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
    if !has_text_sorts {
        info!("Creating text root at position ({:.1}, {:.1})", world_position.x, world_position.y);
        text_editor_state.create_text_root_with_fontir(
            world_position, 
            current_placement_mode.0.to_sort_layout_mode(),
            fontir_app_state.as_deref()
        );
    }
}
