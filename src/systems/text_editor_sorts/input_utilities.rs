//! Input utility functions for keyboard and character handling
//!
//! This module provides utilities for:
//! - Tool shortcut key detection
//! - Key code to character conversion with shift state handling
//! - Unicode character to glyph name mapping

use crate::core::state::AppState;
use bevy::prelude::*;

/// Check if a key is used as a tool shortcut
#[allow(dead_code)]
pub fn is_tool_shortcut_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::KeyT |  // Text tool
        KeyCode::KeyP |  // Pen tool  
        KeyCode::KeyV |  // Select tool
        KeyCode::KeyK |  // Knife tool
        KeyCode::KeyH |  // Hyper tool
        KeyCode::KeyR |  // Shapes tool
        KeyCode::KeyM // Measure/Metaballs tool
    )
}

/// Convert key code to character, considering shift state
pub fn key_code_to_char(
    key: KeyCode,
    keyboard_input: &ButtonInput<KeyCode>,
) -> Option<char> {
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);

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
pub fn unicode_to_glyph_name(
    unicode_char: char,
    app_state: &AppState,
) -> Option<String> {
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

    if app_state
        .workspace
        .font
        .glyphs
        .contains_key(fallback_mapping)
    {
        Some(fallback_mapping.to_string())
    } else {
        None
    }
}
