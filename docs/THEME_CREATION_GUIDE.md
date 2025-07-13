# 🎨 Bezy Theme Creation Guide

Creating custom themes for Bezy is incredibly easy! This guide shows you exactly how to add your own themes.

## ✨ **Super Simple Theme Creation**

### **Step 1: Create Your Theme File**

Create a new `.rs` file in `src/ui/themes/` with your theme name:

```bash
# Example: Create ocean theme
touch src/ui/themes/ocean.rs
```

### **Step 2: Add Your Theme Module**

Add one line to `src/ui/themes/mod.rs`:

```rust
pub mod ocean;  // Add this line
```

### **Step 3: Register Your Theme**

Add one line to the `ThemeRegistry::new()` function in `src/ui/themes/mod.rs`:

```rust
themes.insert("ocean".to_string(), Box::new(|| Box::new(ocean::OceanTheme)));
```

### **Step 4: Copy Theme Template**

Use this template for your theme file (`ocean.rs`):

```rust
//! Ocean Theme for Bezy
//!
//! A deep, calming theme inspired by ocean depths.

use bevy::prelude::*;
use super::BezyTheme;

/// Ocean theme implementation
pub struct OceanTheme;

impl Default for OceanTheme {
    fn default() -> Self {
        Self
    }
}

impl BezyTheme for OceanTheme {
    fn name(&self) -> &'static str {
        "Ocean"
    }

    // =================================================================
    // CUSTOMIZE THESE COLORS! 🎨
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.02, 0.1, 0.15) // Deep ocean blue
    }

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.85, 0.95, 1.0) // Light cyan
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.6, 0.8, 0.9) // Muted cyan
    }

    // ... customize all other colors (see full example in ocean.rs)
}
```

### **Step 5: Use Your Theme!**

```bash
cargo run -- --theme ocean
```

That's it! Your theme is now available! 🎉

## 🎨 **Color Customization Guide**

### **Essential Colors to Customize:**
- `background_color()` - Main app background
- `normal_text_color()` - Primary text
- `secondary_text_color()` - Dimmed text
- `widget_background_color()` - Panel backgrounds
- `on_curve_point_color()` - Glyph outline points
- `off_curve_point_color()` - Bezier control points
- `selected_point_color()` - Selected elements

### **Color Palette Tips:**
- **Background**: Start with your main background color
- **Contrast**: Ensure text has good contrast with backgrounds
- **Accents**: Use 2-3 accent colors for highlights and selection
- **Tools**: Each editing tool can have its own distinct color

### **Bevy Color Format:**
```rust
Color::srgb(red, green, blue)        // Values from 0.0 to 1.0
Color::srgba(red, green, blue, alpha) // With transparency
```

## 🚀 **Advanced Features**

### **Theme Inheritance**
Start with an existing theme and modify only what you need:

```rust
impl BezyTheme for MyTheme {
    // Override just the colors you want to change
    fn background_color(&self) -> Color {
        Color::srgb(0.1, 0.0, 0.2) // Purple background
    }
    
    // All other colors will use the default trait implementations
}
```

### **Theme Sharing**
Share your theme by creating a GitHub Gist with your `.rs` file:

1. Create a Gist with your theme file
2. Users copy the file to their `src/ui/themes/` directory
3. Add the module and registration lines
4. Theme is instantly available!

## 🎯 **Example Themes**

The codebase includes these example themes:
- **darkmode** - Classic dark theme (default)
- **lightmode** - Clean bright theme
- **strawberry** - Warm pink/red theme
- **campfire** - Cozy orange/golden theme
- **ocean** - Deep blue/teal theme

Look at these files for inspiration and copy techniques you like!

## 🛠️ **Development Tips**

### **Testing Your Theme:**
```bash
# Test quickly during development
cargo check  # Make sure it compiles
cargo run -- --theme mytheme
```

### **Common Patterns:**
```rust
// Consistent color families
let base_hue = 0.6; // Blue hue
Color::srgb(base_hue * 0.3, base_hue * 0.8, base_hue) // Dark variant
Color::srgb(base_hue * 0.8, base_hue * 0.9, base_hue) // Light variant

// Accessible contrast
let background = Color::srgb(0.1, 0.1, 0.1); // Dark
let text = Color::srgb(0.9, 0.9, 0.9);       // Light - good contrast!
```

### **Theme Organization:**
Group your colors logically:
1. **Backgrounds** - Base colors for surfaces
2. **Text** - Readable colors for different text types  
3. **Accents** - Highlight colors for interactive elements
4. **Tools** - Distinct colors for each editing tool
5. **States** - Selection, hover, and focus colors

Happy theming! 🎨✨