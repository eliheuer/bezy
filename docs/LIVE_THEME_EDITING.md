# Live Theme Editing Guide

This guide shows you how to edit theme colors in real-time while the app is running using external JSON files.

## How It Works

Instead of editing the compiled Rust source files, you create JSON override files that are watched for changes and applied instantly.

## Quick Start

1. **Run the app in debug mode:**
   ```bash
   cargo run
   ```

2. **Export a theme template:**
   - Press **Cmd+E** (or **Ctrl+E**) in the running app
   - This creates a file like `theme_override_dark_mode.json` in your project root

3. **Edit the JSON file:**
   ```json
   {
     "on_curve_primary": [0.9, 1.0, 0.5],
     "on_curve_secondary": [0.3, 0.4, 0.15],
     "off_curve_primary": [0.6, 0.4, 1.0],
     "off_curve_secondary": [0.2, 0.15, 0.4],
     "selected_primary": [1.0, 1.0, 0.0, 1.0],
     "selected_secondary": [0.4, 0.4, 0.0, 1.0]
   }
   ```

4. **Save the file** - changes appear immediately in the app!

## Color Format

- **RGB colors:** `[red, green, blue]` where each value is 0.0 to 1.0
- **RGBA colors:** `[red, green, blue, alpha]` where alpha is transparency (0.0 = transparent, 1.0 = opaque)

### Examples:
- `[1.0, 0.0, 0.0]` = Pure red
- `[0.0, 1.0, 0.0]` = Pure green  
- `[0.0, 0.0, 1.0]` = Pure blue
- `[0.5, 0.5, 0.5]` = Medium gray
- `[1.0, 1.0, 1.0, 0.5]` = White with 50% transparency

## Available Override Colors

### Point Colors
- `on_curve_primary` - Main color for on-curve points
- `on_curve_secondary` - Secondary color for on-curve points  
- `off_curve_primary` - Main color for off-curve points
- `off_curve_secondary` - Secondary color for off-curve points

### Selection Colors
- `selected_primary` - Main color for selected points
- `selected_secondary` - Secondary color for selected points

### Background Colors
- `background` - Main app background
- `widget_background` - UI widget backgrounds

### Path Colors
- `path_stroke` - Glyph outline color
- `handle_line` - Control handle lines

## Tips

1. **Only specify colors you want to override** - remove any lines for colors you want to keep as default

2. **The file is watched automatically** - changes are applied within 0.5 seconds of saving

3. **Invalid JSON will be ignored** - check the console for error messages if changes don't appear

4. **Per-theme overrides** - each theme gets its own override file (e.g., `theme_override_strawberry.json`)

## Example: Making Bright Pink Points

```json
{
  "on_curve_primary": [1.0, 0.0, 0.5],
  "on_curve_secondary": [0.5, 0.0, 0.25],
  "off_curve_primary": [1.0, 0.5, 0.8],
  "off_curve_secondary": [0.5, 0.25, 0.4]
}
```

## Troubleshooting

- **Changes not appearing?** Check that you saved the JSON file and it has valid syntax
- **File not found?** Make sure you pressed Cmd+E to export the template first
- **Colors look wrong?** Remember RGB values are 0.0-1.0, not 0-255

This system lets you experiment with colors instantly without recompiling - perfect for fine-tuning your theme! 🎨