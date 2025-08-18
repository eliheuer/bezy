# HarfBuzz RTL Text Shaping Implementation Notes

This document describes the current state of HarfBuzz RTL text shaping implementation in Bezy, including all hacks and areas that need improvement.

## Current Status

✅ **WORKING**: HarfBuzz RTL text shaping is functional for the Arabic word "اشهد" (Ashadu) as a proof of concept.

The system successfully:
- Detects Arabic text in the text buffer
- Calls HarfBuzz for professional text shaping
- Maps glyph IDs to proper contextual forms
- Updates text buffer with shaped results

## Architecture Overview

### Core Files

1. **`src/systems/harfbuzz_shaping.rs`** - Main HarfBuzz integration system
2. **`src/core/app.rs`** - Plugin registration (HarfBuzzShapingPlugin enabled)
3. **`test_harfbuzz_arabic.rs`** - Standalone test for determining glyph IDs

### System Integration

- **System Set**: `FontEditorSets::TextBuffer` - Ensures execution order
- **Resource**: `HarfBuzzShapingCache` - Manages font compilation and caching
- **Dependencies**: `harfrust` crate for HarfBuzz bindings

## Current Implementation Details

### Font Loading (HACK #1)

**Current Implementation:**
```rust
pub fn compile_font_for_shaping(...) -> Result<Vec<u8>, String> {
    // HACK: Use existing TTF file directly
    let font_bytes = std::fs::read("assets/fonts/BezyGrotesk-Regular.ttf")
        .map_err(|e| format!("Failed to load BezyGrotesk-Regular.ttf: {}", e))?;
    Ok(font_bytes)
}
```

**TODO:** Should compile from FontIR using fontc, but fontc has issues with Arabic composite glyphs.

**Problem:** fontc panics with "Unable to make progress on composite bbox" when compiling Arabic glyphs.

### Glyph ID Mapping (HACK #2)

**Current Implementation:**
```rust
fn get_glyph_name_from_id(glyph_id: u32, _fontir_state: &FontIRAppState) -> String {
    match glyph_id {
        // Confirmed glyph IDs from HarfBuzz shaping test for "اشهد"
        54 => "alef-ar".to_string(),        // Alef isolated
        93 => "dal-ar.fina".to_string(),    // Dal final form
        107 => "sheen-ar.init".to_string(), // Sheen initial form
        170 => "heh-ar.medi".to_string(),   // Heh medial form
        
        _ => format!("gid{}", glyph_id)
    }
}
```

**TODO:** This should parse the actual font tables to get glyph names dynamically.

**How to Determine Glyph IDs:**
1. Run `cargo run --bin test_harfbuzz_arabic` 
2. Copy the output glyph IDs into the mapping function
3. Test with the full word in the main app

### System Detection (FIXED)

**Previous Issue:** HarfBuzz system wasn't running because it wasn't registered in the correct system set.

**Solution:** Added `.in_set(FontEditorSets::TextBuffer)` to ensure proper execution order.

### Arabic Text Detection

**Current Implementation:**
```rust
// Only run HarfBuzz for Arabic text
if (0x0600..=0x06FF).contains(&code) {
    has_arabic = true;
    // Process with HarfBuzz...
}
```

**Status:** ✅ Working correctly

## Test Results

### HarfBuzz Output for "اشهد"

From `test_harfbuzz_arabic.rs`:

```
Input: اشهد
Characters: ا (U+0627), ش (U+0634), ه (U+0647), د (U+062F)

Shaped into 4 glyphs:
- Glyph[0]: ID 93 = dal-ar.fina (rightmost in RTL)
- Glyph[1]: ID 170 = heh-ar.medi  
- Glyph[2]: ID 107 = sheen-ar.init
- Glyph[3]: ID 54 = alef-ar (leftmost in RTL)
```

This shows correct contextual shaping:
- **Dal (د)**: Final form (connects to preceding letter only)
- **Heh (ه)**: Medial form (connects on both sides)  
- **Sheen (ش)**: Initial form (connects to following letter only)
- **Alef (ا)**: Isolated form (doesn't connect)

## Current Limitations

### 1. Font Compilation Issues

**Problem:** fontc fails to compile UFO with Arabic composite glyphs.

**Error:** "Unable to make progress on composite bbox, stuck at [Glyph { name: theh-ar.fina, ..."

**Workaround:** Using pre-compiled TTF file directly.

**Impact:** Changes to UFO fonts won't be reflected in HarfBuzz shaping until fontc issues are resolved.

### 2. Limited Glyph Coverage

**Problem:** Only 4 glyph IDs are mapped (for "اشهد" only).

**Impact:** Other Arabic text will show as "gid###" instead of proper glyph names.

**Solution:** Need systematic approach to map all Arabic glyphs.

### 3. No Dynamic Glyph Name Resolution

**Problem:** Glyph names are hardcoded rather than read from font tables.

**Impact:** System is brittle and requires manual updates for each font.

**Solution:** Implement proper font table parsing (probably via harfrust's glyph name APIs).

### 4. No Fallback for Non-Arabic Text

**Problem:** System only processes Arabic text, skips everything else.

**Impact:** Latin text with Arabic insertions might not be handled properly.

**Solution:** Implement proper text run detection and processing.

## Future Improvements

### High Priority

1. **Fix fontc Arabic Issues**: Work with fontc maintainers to resolve composite glyph compilation
2. **Dynamic Glyph Mapping**: Implement proper font table parsing for glyph names
3. **Comprehensive Arabic Support**: Extend mapping to cover all Arabic Unicode ranges

### Medium Priority

1. **Text Run Processing**: Proper handling of mixed-script text
2. **Caching Improvements**: Better font compilation caching and invalidation
3. **Error Handling**: More robust error handling and fallbacks

### Low Priority

1. **Performance**: Profile and optimize HarfBuzz integration
2. **Other Scripts**: Support for other complex scripts (Hebrew, Indic, etc.)
3. **Feature Support**: Advanced OpenType features and language-specific shaping

## Testing Procedure

### To Test Arabic Shaping:

1. **Start Bezy**: `RUST_LOG=info cargo run --bin bezy`
2. **Switch to Text Tool**: Press 't' or click text tool
3. **Type Arabic**: Enter "اشهد" 
4. **Verify Output**: Check that glyphs show as contextual forms, not isolated
5. **Check Logs**: Look for "🔤 HarfBuzz:" log messages

### To Add New Glyph Mappings:

1. **Run Test**: `cargo run --bin test_harfbuzz_arabic` with your text
2. **Copy IDs**: Note the glyph IDs from output
3. **Update Mapping**: Add to `get_glyph_name_from_id()` function
4. **Test**: Verify in main app

## Integration Notes

### For Upstream Issues

When working with HarfRust repository:

1. **Document Issues**: Create detailed issue reports with full error output
2. **Provide Minimal Examples**: Include simple test cases that reproduce problems
3. **Test Thoroughly**: Try multiple approaches before reporting as bugs

### For FontIR Integration

The system integrates with FontIR through:

- **Text Buffer**: Updates glyph names and advance widths in text editor state
- **Font Loading**: Retrieves font data from FontIR state
- **Unicode Detection**: Uses codepoint information from text buffer

## Debug Information

### Key Log Messages

- `🔤 HarfBuzz: System called N times` - System is running
- `🔤 HarfBuzz: Found Arabic character` - Arabic text detected
- `🔤 HarfBuzz: Successfully shaped` - Shaping completed
- `🔤 HarfBuzz: Updated glyph ... from '...' to '...'` - Buffer updated

### Debug Commands

```bash
# Run with verbose logging
RUST_LOG=debug cargo run --bin bezy

# Test specific Arabic text
cargo run --bin test_harfbuzz_arabic

# Check for fontc issues
fontc assets/fonts/bezy-grotesk.designspace -o test.ttf
```

## Conclusion

The HarfBuzz RTL text shaping implementation is functional as a proof of concept for Arabic text. While it has several limitations and hacks, it successfully demonstrates professional text shaping integration with Bezy's font editor architecture.

The main blocker for full implementation is the fontc Arabic composite glyph issue, which requires either:
1. Fixing fontc's Arabic glyph compilation, or
2. Implementing a different approach to font compilation for HarfBuzz

All hacks are well-documented and can be systematically replaced with proper implementations as the underlying issues are resolved.