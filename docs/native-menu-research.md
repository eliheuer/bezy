# Native macOS Menu Bar Implementation Research

This document contains research on implementing native macOS menu bars for the Bezy font editor.

## Current Status

✅ **Working**: Cross-platform keyboard shortcuts (Cmd+S/Ctrl+S)  
❌ **Missing**: Native macOS menu bar integration

## Problem

The initial attempt to use `muda` library failed because:
- Native menu libraries require creation on the main thread
- Bevy's startup systems can run on worker threads
- This creates a fundamental threading conflict

Error encountered:
```
thread 'Compute Task Pool (2)' panicked at `muda::Menu` can only be created on the main thread
```

## Options for Native Menu Implementation

### Option 1: Use `tao` + `muda` with Proper Threading (Most Practical)

**Approach**: Create the menu on the main thread before Bevy starts

```rust
// Create menu before starting Bevy app
fn main() {
    // Initialize menu on main thread first
    let menu = create_native_menu().expect("Failed to create menu");
    
    // Then start Bevy with menu reference
    let mut app = create_bevy_app();
    app.insert_resource(MenuHandle { menu });
    app.run();
}
```

**Pros**: 
- Works with existing code
- Cross-platform support
- Minimal architectural changes

**Cons**: 
- Still requires careful thread coordination
- May have limitations with complex menu interactions

### Option 2: Tauri Integration (Recommended for Desktop Apps)

**Approach**: Use Tauri framework designed specifically for desktop applications

```toml
[dependencies]
tauri = { version = "2.0", features = ["api-all"] }
```

```rust
use tauri::{Menu, MenuItem, Submenu};

fn create_menu() -> Menu {
    let file_submenu = Submenu::new("File", Menu::new()
        .add_item(MenuItem::new("Save").accelerator("CmdOrCtrl+S"))
        .add_item(MenuItem::new("Quit").accelerator("CmdOrCtrl+Q"))
    );
    
    Menu::new().add_submenu(file_submenu)
}
```

**Pros**: 
- Native menus work perfectly on all platforms
- Designed specifically for desktop applications
- Active development and good documentation
- Professional desktop app experience

**Cons**: 
- Major architectural change required
- Need to integrate Bevy as a plugin within Tauri
- Learning curve for Tauri integration

### Option 3: Winit Extension (Medium Complexity)

**Approach**: Use winit's macOS extensions to access NSApplication directly

```rust
#[cfg(target_os = "macos")]
use winit::platform::macos::WindowExtMacOS;

// Access the native window
let ns_window = window.ns_window();
// Create NSMenu using objc/cocoa crate
```

**Pros**: 
- Works with Bevy's existing winit integration
- Direct access to native macOS APIs

**Cons**: 
- macOS-only solution
- Requires unsafe Objective-C bindings
- Complex implementation

### Option 4: Wrapper App Architecture (Complex)

**Approach**: Create a native Swift/Objective-C wrapper that communicates with Bevy app

```
BezyWrapper.app (Swift/AppKit)
├── Native menu bar
├── Window management  
└── Embedded Bevy process (IPC)
```

**Pros**: 
- Full native integration
- Best possible user experience
- Complete control over macOS features

**Cons**: 
- Very complex architecture
- Platform-specific code
- Requires maintaining two separate applications

### Option 5: egui Integration (Alternative UI)

**Approach**: Replace parts of the UI with egui for better desktop integration

```rust
// egui has some menu bar support
egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
    ui.menu_button("File", |ui| {
        if ui.button("Save").clicked() {
            // Handle save
        }
    });
});
```

**Pros**: 
- Better desktop integration than pure Bevy UI
- Cross-platform consistency

**Cons**: 
- Not truly native macOS menus
- UI consistency issues between egui and Bevy UI
- Mixed UI framework approach

## Research Findings

### Bevy Limitations
- Bevy is primarily designed for games, not desktop applications
- Limited native desktop integration features
- No built-in NSMenu support as of 2024

### Winit Limitations
- Winit (Bevy's windowing library) owns NSApplication completely
- No API to access or modify the menu bar
- Community proposals exist but not yet implemented

### Current Workarounds
- Most Bevy desktop apps use keyboard shortcuts only
- Some apps use custom UI elements that look like menus
- Native integration requires external frameworks

## Recommendation

For a professional font editor like Bezy, **Option 2 (Tauri Integration)** is recommended because:

1. **Professional Desktop Experience**: Users expect native menus in font editors
2. **Cross-platform Native Support**: Works properly on macOS, Windows, and Linux
3. **Active Development**: Tauri 2.0 has excellent Rust integration
4. **Industry Standard**: Many professional Rust desktop apps use Tauri

## Current Implementation Status

The current implementation provides:
- ✅ Cross-platform keyboard shortcuts (Cmd+S, Ctrl+S)
- ✅ Save functionality with FontIR integration
- ✅ File pane showing save status
- ✅ Backup creation for safety
- ❌ Native menu bar (requires one of the options above)

## Next Steps

1. **Research Tauri Integration**: Investigate how to integrate Bevy within Tauri
2. **Prototype Menu Creation**: Test Option 1 (threading fix) as a simpler first step
3. **Evaluate User Impact**: Determine if native menus are critical for Bezy users
4. **Consider Timeline**: Balance implementation complexity with development priorities

## Files Modified

- `src/ui/file_menu.rs` - Current keyboard-based implementation
- `src/core/app.rs` - FileMenuPlugin integration
- `Cargo.toml` - Dependencies (muda removed due to threading issues)

## Testing

Current keyboard shortcuts work reliably:
- Cmd+S on macOS triggers save operation
- Ctrl+S on Windows/Linux triggers save operation
- Save creates backup files and updates file pane timestamp
- No crashes or threading issues with current implementation

---

*Last Updated: January 2025*
*Status: Research phase - native menu implementation pending*