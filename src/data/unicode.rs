//! Unicode utilities for font processing
//!
//! This module provides Unicode utility functions that can be used 
//! across different font formats (UFO, TTF, OTF, etc.).

/// Remove duplicates and sort a list of Unicode codepoints
pub fn sort_and_deduplicate_codepoints(codepoints: &mut Vec<String>) {
    codepoints.sort();
    codepoints.dedup();
} 