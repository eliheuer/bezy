# FontIR and Kerning Groups Architecture

This document explains why kerning groups are not directly accessible through FontIR and why we implemented a workaround in Bezy.

## Background: FontIR vs UFO Format

### FontIR's Purpose
FontIR is designed as a **compilation target** for font generation, not a complete UFO data container. It serves as:

- **Runtime font generation engine**: Focuses on data needed to compile fonts (outlines, metrics, kerning pairs)
- **Cross-format compatibility layer**: Abstracts format-specific details to work uniformly with UFO, Glyphs, FontLab files
- **Performance-optimized structure**: Only loads and processes data essential for font compilation

### Kerning Groups vs Kerning Data

There's an important distinction between these two concepts:

- **Kerning groups** (`public.kern1.a`, `public.kern2.a`) are UFO-specific organizational structures
- **Kerning pairs** are the actual spacing values FontIR needs for font compilation
- FontIR processes groups into concrete kerning pairs and discards the group structure

## The Data Transformation Process

```
UFO groups.plist → FontIR → Compiled kerning pairs
    ↑                           ↓
    Groups lost here        Only pairs remain
```

When FontIR loads a UFO file:

1. **Reads** `groups.plist` to understand group relationships
2. **Reads** `kerning.plist` and expands group-based rules into concrete glyph pairs
3. **Stores** the expanded kerning pairs for font compilation  
4. **Discards** the original group structure as it's no longer needed for compilation

## Why This Design Makes Sense

This is actually good architecture for FontIR's intended use case:

- **Font compilation focus**: You don't need group names in the final font, just the kerning values
- **Cross-format support**: Not all font formats have group concepts like UFO does
- **Memory efficiency**: Storing expanded pairs is more efficient than maintaining group hierarchies
- **Compilation performance**: Direct pair lookup is faster than group resolution

## The Challenge for Font Editors

However, **font editing applications** like Bezy need that organizational metadata for the user interface. Users want to see:

- Which kerning group a glyph belongs to
- Group-based organization of spacing relationships
- The ability to edit groups, not just individual pairs

## Our Workaround Implementation

Since FontIR doesn't retain the original group structure, we implemented a parallel system:

### 1. Direct UFO Access
```rust
// Load groups directly from UFO files with norad
match norad::Font::load(&ufo_path) {
    Ok(font) => {
        for (group_name, glyph_names) in font.groups.iter() {
            // Store in our parallel structure
            self.kerning_groups.insert(group_name.clone(), names);
        }
    }
}
```

### 2. Designspace Support
```rust
// Handle designspace files by loading groups from all source UFOs
match DesignSpaceDocument::load(&source_path) {
    Ok(designspace) => {
        for source in &designspace.sources {
            let ufo_path = designspace_dir.join(&source.filename);
            self.load_groups_from_ufo(&ufo_path)?;
        }
    }
}
```

### 3. Parallel Data Structure
```rust
/// Kerning groups data loaded from UFO groups.plist
/// Maps group name (e.g. "public.kern1.a") to list of glyph names
pub kerning_groups: HashMap<String, Vec<String>>,
```

### 4. Manual Synchronization
- Load groups separately during FontIR initialization
- Maintain groups alongside FontIR's compiled kerning data
- Provide UI access through `get_glyph_kerning_groups()`

## The Architectural Trade-off

This represents a common pattern when using specialized libraries:

**FontIR's Strengths:**
- Excellent for font compilation and cross-format support
- High performance and memory efficiency
- Handles complex variable font scenarios

**FontIR's Limitations for Editing:**
- Discards source-level organizational metadata
- Optimized for compilation, not editing workflows
- Format-specific features get abstracted away

**Our Solution:**
- Use FontIR for what it's good at (glyph data, metrics, compilation)
- Maintain parallel structures for editing-specific needs (groups, UI metadata)
- Accept the complexity of manual synchronization

## Lessons Learned

1. **Library Purpose Matters**: FontIR is a compilation library, not an editing library
2. **Abstraction Has Costs**: Higher-level abstractions can hide details you need
3. **Hybrid Approaches Work**: Combining specialized libraries with custom code is often necessary
4. **Data Lifecycle Awareness**: Understanding when data gets transformed/discarded is crucial

This pattern appears in many domains - you often need to maintain additional data structures alongside specialized libraries to support your specific use case.

## Future Considerations

If this becomes a common need across font editors, potential solutions could include:

1. **FontIR Extension**: Add optional metadata preservation to FontIR
2. **Separate Metadata Library**: Create a companion library for UFO metadata
3. **Editor-Specific Fork**: Maintain a fork of FontIR with editing features
4. **Standard Protocol**: Define interfaces for metadata preservation across compilation libraries

For now, our workaround provides the functionality Bezy needs while leveraging FontIR's strengths.