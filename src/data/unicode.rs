//! Unicode range constants and utilities
//!
//! This module provides Unicode range definitions and utility functions 
//! that can be used across different font formats (UFO, TTF, OTF, etc.).

/// Unicode ranges for common character sets
pub struct UnicodeRanges;

impl UnicodeRanges {
    // Basic Latin (ASCII)
    pub const BASIC_LATIN_START: u32 = 0x0020;
    pub const BASIC_LATIN_END: u32 = 0x007F;
    
    // Latin-1 Supplement
    pub const LATIN_1_SUPPLEMENT_START: u32 = 0x00A0;
    pub const LATIN_1_SUPPLEMENT_END: u32 = 0x00FF;
    
    // Latin Extended-A
    pub const LATIN_EXTENDED_A_START: u32 = 0x0100;
    pub const LATIN_EXTENDED_A_END: u32 = 0x017F;
    
    // Latin Extended-B
    pub const LATIN_EXTENDED_B_START: u32 = 0x0180;
    pub const LATIN_EXTENDED_B_END: u32 = 0x024F;
    
    // Cyrillic
    pub const CYRILLIC_START: u32 = 0x0400;
    pub const CYRILLIC_END: u32 = 0x04FF;
    
    // Arabic
    pub const ARABIC_START: u32 = 0x0600;
    pub const ARABIC_END: u32 = 0x06FF;
}

/// Get commonly scanned Unicode ranges for font analysis
pub fn get_common_unicode_ranges() -> Vec<(u32, u32)> {
    vec![
        (UnicodeRanges::BASIC_LATIN_START, UnicodeRanges::BASIC_LATIN_END),
        (UnicodeRanges::LATIN_1_SUPPLEMENT_START, UnicodeRanges::LATIN_1_SUPPLEMENT_END),
        (UnicodeRanges::LATIN_EXTENDED_A_START, UnicodeRanges::LATIN_EXTENDED_A_END),
        (UnicodeRanges::LATIN_EXTENDED_B_START, UnicodeRanges::LATIN_EXTENDED_B_END),
        (UnicodeRanges::CYRILLIC_START, UnicodeRanges::CYRILLIC_END),
        (UnicodeRanges::ARABIC_START, UnicodeRanges::ARABIC_END),
    ]
}

/// Parse a hex codepoint string to a Unicode character
pub fn parse_codepoint(codepoint_hex: &str) -> Option<char> {
    let hex_str = codepoint_hex.trim_start_matches("0x");
    
    match u32::from_str_radix(hex_str, 16) {
        Ok(code_value) => char::from_u32(code_value),
        Err(_) => None,
    }
}

/// Generate common glyph name variants for a Unicode character
pub fn generate_glyph_name_variants(target_char: char) -> [String; 3] {
    let code = target_char as u32;
    [
        target_char.to_string(),
        format!("uni{:04X}", code),
        format!("u{:04X}", code),
    ]
}

/// Sort codepoints numerically and remove duplicates
pub fn sort_and_deduplicate_codepoints(codepoints: &mut Vec<String>) {
    codepoints.sort_by(|a, b| {
        let a_val = u32::from_str_radix(a, 16).unwrap_or(0);
        let b_val = u32::from_str_radix(b, 16).unwrap_or(0);
        a_val.cmp(&b_val)
    });
    
    codepoints.dedup();
} 