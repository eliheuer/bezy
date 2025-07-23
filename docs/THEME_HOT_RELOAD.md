# Theme Hot Reloading Guide

Bezy now supports hot reloading of theme colors while the app is running! This allows you to see color changes immediately without recompiling.

## How to Use

### Method 1: Keyboard Shortcut (Recommended)

1. Run the app in debug mode: `cargo run`
2. Edit any theme file in `src/ui/themes/` (e.g., `darkmode.rs`)
3. Change any color values in the theme implementation
4. Save the file
5. Press **Ctrl+R** (or **Cmd+R** on Mac) in the app to reload the theme
6. Your color changes will appear immediately!

### Method 2: Automatic File Watching (Alternative)

The hot reload system also includes automatic file watching that checks for changes every 0.5 seconds. However, due to Rust's compilation model, this requires the theme files to be interpreted at runtime, which is more complex.

## Example: Changing Point Colors

Let's say you want to experiment with different point colors in the dark theme:

1. Open `src/ui/themes/darkmode.rs`
2. Find the point color methods:
   ```rust
   fn on_curve_primary_color(&self) -> Color {
       Color::srgb(0.3, 1.0, 0.5)  // Try changing these values!
   }
   ```
3. Change the color values (RGB values are 0.0 to 1.0)
4. Save the file
5. Press **Ctrl+R** in the running app
6. See your new colors instantly!

## Tips for Color Experimentation

- **Primary colors** should be bright/vibrant for visibility
- **Secondary colors** should be darker/muted versions of the primary
- The system uses a three-layer rendering:
  - Layer 1: Full size shape with primary color
  - Layer 2: 70% size shape with secondary color  
  - Layer 3: Small center shape with primary color

## Color Value Reference

Colors use RGB values from 0.0 to 1.0:
- `Color::srgb(1.0, 0.0, 0.0)` = Pure red
- `Color::srgb(0.0, 1.0, 0.0)` = Pure green
- `Color::srgb(0.0, 0.0, 1.0)` = Pure blue
- `Color::srgb(0.5, 0.5, 0.5)` = Medium gray

For colors with transparency:
- `Color::srgba(1.0, 1.0, 1.0, 0.5)` = White with 50% opacity

## Limitations

- Hot reload only works in debug builds (`cargo run` without `--release`)
- Only color values can be hot reloaded - structural changes require recompilation
- The theme file must compile successfully for reload to work

## Future Enhancements

The system could be extended to support:
- External TOML/JSON files for theme configuration
- Live color picker UI within the app
- Theme export/import functionality
- Per-user theme customization

Happy theming! 🎨