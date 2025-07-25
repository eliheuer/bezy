//! Unicode input handling for text editor sorts
//!
//! This module provides Unicode character input support for the text editor,
//! enabling input of any Unicode character including Latin, Arabic, Hebrew,
//! Chinese, Japanese, Korean, and other global scripts.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::{AppState, TextEditorState};
use crate::systems::text_editor_sorts::input_utilities::unicode_to_glyph_name;
use crate::ui::toolbars::edit_mode_toolbar::text::{
    CurrentTextPlacementMode, TextPlacementMode,
};
use crate::ui::toolbars::edit_mode_toolbar::CurrentTool;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

/// Handle Unicode character input using Bevy 0.16 keyboard events
/// This system provides comprehensive Unicode support for global scripts
pub fn handle_unicode_text_input(
    mut key_evr: EventReader<KeyboardInput>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    current_tool: Res<CurrentTool>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
) {
    // Only handle input when text tool is active
    if current_tool.get_current() != Some("text") {
        return;
    }

    // Handle keyboard input events
    for ev in key_evr.read() {
        // Only process pressed keys
        if !ev.state.is_pressed() {
            continue;
        }

        match &ev.logical_key {
            // Handle Unicode character input
            Key::Character(character_string) => {
                // Process each character in the string (usually just one)
                for character in character_string.chars() {
                    // Skip control characters (except newline)
                    if character.is_control() && character != '\n' {
                        continue;
                    }

                    // Handle space character
                    if character == ' ' {
                        handle_space_character(
                            &mut text_editor_state,
                            &app_state,
                            &fontir_app_state,
                        );
                        continue;
                    }

                    // Handle newline (Enter key)
                    if character == '\n' {
                        handle_newline_character(
                            &mut text_editor_state,
                            &current_placement_mode,
                        );
                        continue;
                    }

                    // Handle regular Unicode character
                    handle_unicode_character(
                        character,
                        &mut text_editor_state,
                        &app_state,
                        &fontir_app_state,
                        &current_placement_mode,
                    );
                }
            }
            // Handle special keys
            Key::Backspace => {
                handle_backspace(
                    &mut text_editor_state,
                    &current_placement_mode,
                );
            }
            Key::Delete => {
                handle_delete(&mut text_editor_state, &current_placement_mode);
            }
            Key::Enter => {
                handle_newline_character(
                    &mut text_editor_state,
                    &current_placement_mode,
                );
            }
            Key::Space => {
                handle_space_character(
                    &mut text_editor_state,
                    &app_state,
                    &fontir_app_state,
                );
            }
            _ => {
                // Ignore other special keys
            }
        }
    }
}

/// Handle a single Unicode character input
fn handle_unicode_character(
    character: char,
    text_editor_state: &mut TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    // Find glyph name for this Unicode character
    let glyph_name = if let Some(app_state) = app_state.as_ref() {
        unicode_to_glyph_name(character, app_state)
    } else {
        // For FontIR, use character as glyph name or try common mappings
        unicode_to_glyph_name_fontir(character, fontir_app_state)
    };

    if let Some(glyph_name) = glyph_name {
        // Get advance width
        let advance_width =
            get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // Check if we need to create a text root
        let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
        if !has_text_sorts {
            let center_position = Vec2::new(500.0, 0.0);
            text_editor_state.create_text_root(center_position);
        }

        // Insert the character
        match current_placement_mode.0 {
            TextPlacementMode::Insert => {
                text_editor_state
                    .insert_sort_at_cursor(glyph_name.clone(), advance_width);
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Insert mode", 
                      character, character as u32, glyph_name);
            }
            TextPlacementMode::Text => {
                text_editor_state
                    .insert_sort_at_cursor(glyph_name.clone(), advance_width);
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Text mode", 
                      character, character as u32, glyph_name);
            }
            TextPlacementMode::Freeform => {
                // In freeform mode, characters are placed freely - for now use same logic
                text_editor_state
                    .insert_sort_at_cursor(glyph_name.clone(), advance_width);
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Freeform mode", 
                      character, character as u32, glyph_name);
            }
        }
    } else {
        warn!(
            "Unicode input: No glyph found for character '{}' (U+{:04X})",
            character, character as u32
        );
    }
}

/// Handle space character input
fn handle_space_character(
    text_editor_state: &mut TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) {
    let glyph_name = "space".to_string();

    // Check if space glyph exists
    let glyph_exists = if let Some(app_state) = app_state.as_ref() {
        app_state.workspace.font.glyphs.contains_key(&glyph_name)
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        fontir_state.get_glyph(&glyph_name).is_some()
    } else {
        false
    };

    if glyph_exists {
        let advance_width =
            get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // Check if we need to create a text root
        let has_text_sorts = !text_editor_state.get_text_sorts().is_empty();
        if !has_text_sorts {
            let center_position = Vec2::new(500.0, 0.0);
            text_editor_state.create_text_root(center_position);
        }

        text_editor_state.insert_sort_at_cursor(glyph_name, advance_width);
        info!("Unicode input: Inserted space character");
    } else {
        // Fallback: insert a space-width advance without glyph
        let space_width = 250.0; // Default space width
        text_editor_state
            .insert_sort_at_cursor("space".to_string(), space_width);
        info!("Unicode input: Inserted space character (fallback)");
    }
}

/// Handle newline character input
fn handle_newline_character(
    text_editor_state: &mut TextEditorState,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert => {
            text_editor_state.insert_line_break_at_cursor();
            info!("Unicode input: Inserted line break in Insert mode");
        }
        TextPlacementMode::Text => {
            // In Text mode, newlines might move to next line in grid
            text_editor_state.insert_line_break_at_cursor();
            info!("Unicode input: Inserted line break in Text mode");
        }
        TextPlacementMode::Freeform => {
            // In Freeform mode, newlines might not be meaningful
            info!("Unicode input: Newline ignored in Freeform mode");
        }
    }
}

/// Handle backspace key
fn handle_backspace(
    text_editor_state: &mut TextEditorState,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert => {
            text_editor_state.delete_sort_at_cursor();
            info!("Unicode input: Backspace in Insert mode");
        }
        TextPlacementMode::Text => {
            if text_editor_state.cursor_position > 0 {
                text_editor_state.move_cursor_left();
                text_editor_state.delete_sort_at_cursor();
            }
            info!("Unicode input: Backspace in Text mode");
        }
        TextPlacementMode::Freeform => {
            if text_editor_state.cursor_position > 0 {
                text_editor_state.move_cursor_left();
                text_editor_state.delete_sort_at_cursor();
            }
            info!("Unicode input: Backspace in Freeform mode");
        }
    }
}

/// Handle delete key
fn handle_delete(
    text_editor_state: &mut TextEditorState,
    _current_placement_mode: &CurrentTextPlacementMode,
) {
    text_editor_state.delete_sort_at_cursor();
    info!("Unicode input: Delete key pressed");
}

/// Convert Unicode character to glyph name for FontIR
fn unicode_to_glyph_name_fontir(
    character: char,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) -> Option<String> {
    if let Some(fontir_state) = fontir_app_state.as_ref() {
        // Try direct character mapping first
        let char_name = character.to_string();
        if fontir_state.get_glyph(&char_name).is_some() {
            return Some(char_name);
        }

        // Try common Unicode to glyph name mappings
        let glyph_name = match character {
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
            '!' => "exclam",
            '?' => "question",
            '.' => "period",
            ',' => "comma",
            ':' => "colon",
            ';' => "semicolon",
            '(' => "parenleft",
            ')' => "parenright",
            '-' => "hyphen",
            '_' => "underscore",
            '+' => "plus",
            '=' => "equal",
            '@' => "at",
            '#' => "numbersign",
            '$' => "dollar",
            '%' => "percent",
            '&' => "ampersand",
            '*' => "asterisk",
            '/' => "slash",
            '\\' => "backslash",
            '|' => "bar",
            '<' => "less",
            '>' => "greater",
            '[' => "bracketleft",
            ']' => "bracketright",
            '{' => "braceleft",
            '}' => "braceright",
            '"' => "quotedbl",
            '\'' => "quotesingle",
            '`' => "grave",
            '~' => "asciitilde",
            '^' => "asciicircum",
            // For letters, try lowercase first
            c if c.is_ascii_alphabetic() => {
                let lower_name = c.to_lowercase().to_string();
                if fontir_state.get_glyph(&lower_name).is_some() {
                    return Some(lower_name);
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        if fontir_state.get_glyph(glyph_name).is_some() {
            Some(glyph_name.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Get advance width for a glyph from either AppState or FontIR
fn get_glyph_advance_width(
    glyph_name: &str,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) -> f32 {
    if let Some(app_state) = app_state.as_ref() {
        if let Some(glyph_data) =
            app_state.workspace.font.glyphs.get(glyph_name)
        {
            return glyph_data.advance_width as f32;
        }
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        return fontir_state.get_glyph_advance_width(glyph_name);
    }

    // Fallback default width
    500.0
}

/// Legacy function - replaced by handle_unicode_text_input
pub fn handle_unicode_input() {
    // This function is kept for compatibility but does nothing
    // Use handle_unicode_text_input instead
}
