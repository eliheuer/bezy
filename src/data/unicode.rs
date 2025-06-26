//! Unicode data utilities.

#[allow(dead_code)]
pub fn sort_and_deduplicate_codepoints(codepoints: &mut Vec<String>) {
    codepoints.sort_unstable();
    codepoints.dedup();
} 
