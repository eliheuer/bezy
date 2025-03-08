# UFO Data in Bezy

This document explains how Unified Font Object (UFO) font data is loaded from disk and stored in a running Bezy application.

## Overview

Bezy is a font editor that works with UFO (Unified Font Object) files, which are a standardized format for font data. The application uses the [norad](https://github.com/linebender/norad) crate to handle the low-level parsing and manipulation of UFO files. This document explains the flow of data from a UFO file on disk to the in-memory representation used by Bezy.

## Loading Process

### 1. Initial Load

The UFO loading process is initiated in one of two ways:

1. **Command Line Argument**: When Bezy is launched with the `--load-ufo` argument.
2. **User Interface Action**: When a user loads a font via the UI (not shown in the code snippets).

The command line approach is handled during the application startup by the `initialize_font_state` system, which is registered as a startup system in `SetupPlugin`.

### 2. Loading from Disk

The function `load_ufo_from_path` in `src/ufo.rs` handles the actual loading of UFO data from disk:

```rust
pub fn load_ufo_from_path(
    path: &str,
) -> Result<Ufo, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);
    if !font_path.exists() {
        return Err(format!("File not found: {}", font_path.display()).into());
    }

    Ok(Ufo::load(font_path)?)
}
```

This function:
1. Converts the path string to a `PathBuf`
2. Checks if the file exists
3. Uses the `norad` crate's `Ufo::load` method to parse the UFO file
4. Returns the loaded `Ufo` object or an error

### 3. Initializing Application State

After loading the UFO data, it's stored in the application state through the `initialize_font_state` system:

```rust
pub fn initialize_font_state(
    mut commands: Commands,
    cli_args: Res<crate::cli::CliArgs>,
) {
    // Check if a UFO path was provided via CLI
    if let Some(ufo_path) = &cli_args.ufo_path {
        // Load UFO file from the path provided via CLI
        match load_ufo_from_path(ufo_path.to_str().unwrap_or_default()) {
            Ok(ufo) => {
                let mut state = AppState::default();
                state.set_font(ufo, Some(ufo_path.clone()));
                let display_name = state.get_font_display_name();
                commands.insert_resource(state);
                info!("Loaded font: {}", display_name);
            }
            Err(e) => {
                error!("Failed to load UFO file: {}", e);
                commands.init_resource::<AppState>();
            }
        }
    } else {
        // No CLI argument provided, just initialize an empty state
        commands.init_resource::<AppState>();
    }
}
```

This system:
1. Checks if a UFO path was provided via command line arguments
2. If yes, attempts to load the UFO file
3. If successful, creates a new `AppState` and sets the font
4. Inserts the state as a resource into the ECS (Entity Component System)
5. If loading fails or no path was provided, initializes an empty `AppState`

## Data Storage Structure

The UFO data is stored in a hierarchical structure in Bezy's memory:

### 1. AppState

`AppState` is the top-level resource that contains all application state data:

```rust
#[derive(Resource, Default, Clone)]
pub struct AppState {
    pub workspace: Workspace,
}
```

### 2. Workspace

`Workspace` represents a single font (corresponding to a UFO file on disk):

```rust
#[derive(Clone, Default)]
pub struct Workspace {
    pub font: Arc<FontObject>,
    pub selected: Option<GlyphName>,
    pub open_glyphs: Arc<HashMap<GlyphName, Entity>>,
    pub info: SimpleFontInfo,
}
```

The `Workspace` holds:
- `font`: An atomic reference counted `FontObject` that contains the actual UFO data
- `selected`: The currently selected glyph (if any)
- `open_glyphs`: A map of glyphs that are currently open in editors
- `info`: A simplified view of font metadata

### 3. FontObject

`FontObject` encapsulates the raw UFO data and its path:

```rust
#[derive(Clone)]
pub struct FontObject {
    pub path: Option<Arc<Path>>,
    pub ufo: Ufo,
}
```

The `FontObject` contains:
- `path`: An optional path to the UFO file on disk (used for saving)
- `ufo`: The actual `Ufo` object from the norad crate, which contains all the font data

### 4. SimpleFontInfo

`SimpleFontInfo` provides a convenient access to commonly used font metadata:

```rust
#[derive(Clone)]
pub struct SimpleFontInfo {
    pub metrics: FontMetrics,
    pub family_name: String,
    pub style_name: String,
}
```

### 5. FontMetrics

`FontMetrics` stores the metrics information of the font:

```rust
#[derive(Clone)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub descender: Option<f64>,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
    pub ascender: Option<f64>,
    pub italic_angle: Option<f64>,
}
```

## Data Flow Within the Application

1. When a UFO file is loaded, its data is parsed by the `norad` crate into a `Ufo` object.
2. This `Ufo` object is wrapped in a `FontObject` along with its path.
3. The `FontObject` is stored in a `Workspace` along with derived metadata like `SimpleFontInfo`.
4. The `Workspace` is stored in `AppState`, which is inserted as a resource in Bevy's ECS.
5. Various systems can then access and modify this data as needed.

## Accessing UFO Data

Once loaded, systems can access the UFO data through the `AppState` resource:

```rust
fn my_system(app_state: Res<AppState>) {
    // Access the UFO data
    if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        // Do something with the default layer
    }
}
```

## Finding Glyphs by Unicode Codepoint

Bezy includes functionality to find glyphs by their Unicode codepoints:

```rust
pub fn find_glyph_by_unicode(ufo: &Ufo, codepoint_hex: &str) -> Option<String> {
    // Parse the hex value
    let codepoint =
        match u32::from_str_radix(codepoint_hex.trim_start_matches("0x"), 16) {
            Ok(cp) => cp,
            Err(_) => return None,
        };

    // Convert the U32 codepoint to a Rust char
    let target_char = match char::from_u32(codepoint) {
        Some(c) => c,
        None => return None,
    };
    
    // Get the default layer
    if let Some(default_layer) = ufo.get_default_layer() {
        // For Latin lowercase a-z (0061-007A), uppercase A-Z (0041-005A),
        // Try the character name
        if (0x0061..=0x007A).contains(&codepoint) || // a-z
           (0x0041..=0x005A).contains(&codepoint) {  // A-Z
            let glyph_name = norad::GlyphName::from(target_char.to_string());
            if let Some(_glyph) = default_layer.get_glyph(&glyph_name) {
                // Found a match!
                return Some(glyph_name.to_string());
            }
        }
        
        // Try conventional format "uni<CODE>"
        let uni_name = format!("uni{:04X}", codepoint);
        let glyph_name = norad::GlyphName::from(uni_name);
        if let Some(_) = default_layer.get_glyph(&glyph_name) {
            return Some(glyph_name.to_string());
        }
        
        // Special cases for common characters
        let special_cases = [
            (0x0020, "space"),       // Space
            (0x002E, "period"),      // Period
            (0x002C, "comma"),       // Comma
            (0x0027, "quotesingle"), // Single quote
        ];
        
        for (cp, name) in special_cases.iter() {
            if *cp == codepoint {
                let glyph_name = norad::GlyphName::from(*name);
                if let Some(_) = default_layer.get_glyph(&glyph_name) {
                    return Some(glyph_name.to_string());
                }
            }
        }
    }

    // If we got here, we didn't find a matching glyph
    None
}
```

This function demonstrates how Bezy maps between Unicode codepoints and glyph names within the UFO data structure.

## Comprehensive Unicode Support

Bezy includes comprehensive support for working with Unicode codepoints across the entire Unicode spectrum. This allows users to cycle through and view all glyphs defined in a font, regardless of writing system.

### 1. Unicode Codepoint Detection

The application thoroughly scans all Unicode codepoints available in the loaded font using the `get_all_codepoints` function:

```rust
pub fn get_all_codepoints(ufo: &Ufo) -> Vec<String> {
    // Thorough implementation that checks every codepoint in major Unicode blocks
    // ...
}
```

This function examines every codepoint across major Unicode blocks, ensuring complete coverage of:

- Latin, Greek, and Cyrillic scripts
- Middle Eastern scripts (Hebrew, Arabic, Syriac, etc.)
- South and Southeast Asian scripts (Devanagari, Thai, etc.)
- East Asian scripts (CJK, Hiragana, Katakana, Hangul, etc.)
- Symbols, punctuation, and emoji
- Other writing systems and specialized blocks

### 2. Comprehensive Scanning

Bezy uses a thorough approach to ensure no codepoints are missed:

1. Instead of sampling or approximating, it checks EVERY codepoint in each Unicode block
2. This ensures all scripts are properly represented, even if they have sparse codepoint coverage
3. The approach guarantees that cycling will include every supported glyph across all writing systems

### 3. Codepoint Cycling

The application provides seamless cycling through all available codepoints in proper Unicode order:

```rust
pub fn find_next_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    // Implementation that finds the next codepoint in sequence
    // ...
}

pub fn find_previous_codepoint(ufo: &Ufo, current_hex: &str) -> Option<String> {
    // Implementation that finds the previous codepoint in sequence
    // ...
}
```

These functions:
- Get the list of all available codepoints from the font
- Find the currently selected codepoint
- Return the next or previous codepoint in sequence (with wrap-around)
- Ensure cycling moves through all supported codepoints in their proper Unicode order

### 4. Keyboard Navigation

Users can navigate through the font's complete glyph set using keyboard shortcuts:
- **Shift++**: Cycle to the next codepoint
- **Shift+-**: Cycle to the previous codepoint

This navigation system ensures access to every single glyph in the font, regardless of script or Unicode block, in a predictable order following the Unicode standard sequence.

## Saving UFO Data

The `Workspace.save()` method is responsible for saving the UFO data back to disk:

1. It first updates the font information from the simplified view.
2. It saves to a temporary location.
3. It backs up the existing data if needed.
4. It renames the temporary file to the target path.

```rust
pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let font_obj = Arc::make_mut(&mut self.font);
    font_obj.update_info(&self.info);

    if let Some(path) = font_obj.path.as_ref() {
        // Write to a temporary location first
        let temp_path = temp_write_path(path);
        info!("saving to {:?}", temp_path);
        font_obj.ufo.save(&temp_path)?;

        // Backup existing data if needed
        if let Some(backup_path) = backup_ufo_at_path(path)? {
            info!("backing up existing data to {:?}", backup_path);
        }

        std::fs::rename(&temp_path, path)?;
        Ok(())
    } else {
        Err("save called with no path set".into())
    }
}
```

## Conclusion

Bezy uses a well-structured approach to loading, storing, and manipulating UFO font data:

1. The `norad` crate handles the low-level parsing and serialization of UFO files.
2. The parsed data is wrapped in application-specific types (`FontObject`, `Workspace`, etc.).
3. These types are stored in the `AppState` resource, making them accessible to all systems.
4. Helper methods on these types provide convenient access to common operations, such as finding glyphs by Unicode codepoint.

This design separates the concerns of file I/O, data representation, and application logic, resulting in a maintainable and extensible codebase. 