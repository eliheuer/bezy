//! Unicode utilities for font processing
//!
//! This module provides Unicode utility functions that can be used 
//! across different font formats (UFO, TTF, OTF, etc.).

/// Parse a hex codepoint string to a Unicode character
pub fn parse_codepoint(codepoint_hex: &str) -> Option<char> {
    let hex_str = codepoint_hex.trim_start_matches("0x");
    
    match u32::from_str_radix(hex_str, 16) {
        Ok(code_value) => char::from_u32(code_value),
        Err(_) => None,
    }
}

/// Generate common glyph name variants for a Unicode character
pub fn generate_glyph_name_variants(character: char) -> Vec<String> {
    let mut variants = Vec::new();
    let unicode_value = character as u32;
    
    // Add the character itself as a name (for basic ASCII)
    if character.is_ascii_alphabetic() {
        variants.push(character.to_string());
    }
    
    // Add standard UFO naming conventions
    variants.push(format!("uni{:04X}", unicode_value));
    variants.push(format!("u{:04X}", unicode_value));
    
    // Add special cases for common characters
    match character {
        ' ' => variants.push("space".to_string()),
        '.' => variants.push("period".to_string()),
        ',' => variants.push("comma".to_string()),
        '\'' => variants.push("quotesingle".to_string()),
        _ => {}
    }
    
    variants
}

/// Remove duplicates and sort a list of Unicode codepoints
pub fn sort_and_deduplicate_codepoints(codepoints: &mut Vec<String>) {
    codepoints.sort();
    codepoints.dedup();
} 