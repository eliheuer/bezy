# Edit Mode Toolbar - Usage Guide

The new edit mode toolbar system makes it incredibly easy to add new tools. Here's how:

## Adding a New Tool

### 1. Create Your Tool File

Create a new file like `src/ui/toolbars/edit_mode_toolbar/my_tool.rs`:

```rust
use bevy::prelude::*;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};

pub struct MyTool;

impl EditTool for MyTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "my_tool"  // Unique identifier
    }
    
    fn name(&self) -> &'static str {
        "My Tool"  // Display name in UI
    }
    
    fn icon(&self) -> &'static str {
        "\u{E019}"  // Unicode icon character
    }
    
    fn shortcut_key(&self) -> Option<char> {
        Some('m')  // Optional keyboard shortcut
    }
    
    fn default_order(&self) -> i32 {
        50  // Lower numbers appear first
    }
    
    fn description(&self) -> &'static str {
        "My custom tool does amazing things"
    }
    
    fn update(&self, commands: &mut Commands) {
        // Tool behavior while active
        // Handle mouse events, keyboard input, etc.
    }
    
    fn on_enter(&self) {
        info!("Entered My Tool");
        // Setup when tool becomes active
    }
    
    fn on_exit(&self) {
        info!("Exited My Tool");
        // Cleanup when switching away
    }
}

pub struct MyToolPlugin;

impl Plugin for MyToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_my_tool);
    }
}

fn register_my_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(MyTool));
}
```

### 2. Add the Module Declaration

Add one line to `src/ui/toolbars/edit_mode_toolbar/mod.rs`:

```rust
mod my_tool;  // Add this line
pub use my_tool::MyToolPlugin;  // And this one
```

### 3. Register the Plugin

Add your tool's plugin to your app:

```rust
app.add_plugins(MyToolPlugin);
```

**That's it!** Your tool will automatically appear in the toolbar.

## Controlling Tool Order

### Default Ordering

Tools are ordered by their `default_order()` value (lower = first):

```rust
fn default_order(&self) -> i32 {
    10  // Appears before tools with order 20, 30, etc.
}
```

### Custom Ordering

You can override the order for all tools:

```rust
// In your app setup
fn setup_custom_tool_order(mut tool_ordering: ResMut<ToolOrdering>) {
    tool_ordering.set_order(vec![
        "select",
        "my_tool",     // Your tool appears second
        "pen",
        "eraser",
        // Other tools follow their default order
    ]);
}

// Or use predefined orders
tool_ordering.set_design_focused_order();
tool_ordering.set_annotation_focused_order();
```

## Advanced Features

### Temporary Tool Activation

```rust
fn supports_temporary_mode(&self) -> bool {
    true  // Tool can be temporarily activated (e.g., spacebar for pan)
}
```

### Rich Tool Metadata

```rust
fn description(&self) -> &'static str {
    "Detailed description for tooltips and help"
}

fn shortcut_key(&self) -> Option<char> {
    Some('x')  // Keyboard shortcut
}
```

## Examples

Check out these existing tools for reference:
- `select.rs` - Basic selection tool
- `eraser.rs` - Simple eraser tool with custom ordering

## Migration from Old System

The old hardcoded enum system is still supported during the transition. Tools can gradually be converted to the new system. 