# Bezy Architecture Guide

Bezy is a modern font editor built with Rust and Bevy. This guide helps developers, designers, and LLMs understand and extend the codebase.

## Quick Start

### For Font Designers
- **Add new themes**: Create a file in `src/ui/themes/` (copy an existing theme)
- **Modify tools**: Look in `src/tools/` for pen, select, knife, etc.
- **Change visual constants**: Edit `src/ui/theme.rs`

### For Developers
- **Add a new tool**: Create `src/tools/your_tool.rs` implementing `EditTool`
- **Add glyph operations**: Add to `src/editing/operations/`
- **Add UI components**: Add to `src/ui/panes/` or `src/ui/`

### For LLMs
- **Module size**: Most files are 200-500 lines (fits in context)
- **Clear hierarchy**: Each module has a specific purpose
- **Entry points**: Start with `main.rs` → `core/app.rs` → specific features

## Architecture Overview

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   main.rs   │────▶│  core/app.rs │────▶│   Plugins    │
└─────────────┘     └──────────────┘     └──────────────┘
                            │
                            ▼
        ┌───────────────────┴───────────────────┐
        │                                       │
        ▼                                       ▼
┌───────────────┐                     ┌─────────────────┐
│  Core Systems │                     │   UI Systems    │
├───────────────┤                     ├─────────────────┤
│ • State       │                     │ • Tools         │
│ • Input/IO    │◀────────────────────│ • Panes         │
│ • Settings    │                     │ • Themes        │
└───────────────┘                     └─────────────────┘
        ▲                                       ▲
        │                                       │
        └───────────────┬───────────────────────┘
                        │
                        ▼
                ┌───────────────┐
                │ Font Systems  │
                ├───────────────┤
                │ • Data/UFO    │
                │ • Editing     │
                │ • Rendering   │
                └───────────────┘
```

## Module Structure

### Core Infrastructure (`src/core/`)
Foundation modules that everything depends on:

- **`app.rs`**: Application initialization and plugin registration
- **`cli.rs`**: Command-line argument parsing
- **`settings.rs`**: User preferences and configuration
- **`state/`**: Application state management
  - `app_state.rs`: Global application state
  - `navigation.rs`: Current glyph/font navigation
  - `font_data.rs`: Font data structures
- **`io/`**: Input/output handling
  - `input.rs`: Centralized keyboard/mouse input
  - `pointer.rs`: Mouse position tracking
  - `gamepad.rs`: Gamepad support

### Data Layer (`src/data/`)
Font format handling:

- **`ufo.rs`**: UFO font format loading/saving
- **`conversions.rs`**: Data type conversions

### Editing Operations (`src/editing/`)
All glyph modification operations:

- **`operations/`**: Individual edit operations
  - `point_operations.rs`: Move, add, delete points
  - `contour_operations.rs`: Contour manipulation
  - `selection_operations.rs`: Selection changes
- **`undo.rs`**: Undo/redo system
- **`selection/`**: Selection state management

### Geometry (`src/geometry/`)
Mathematical primitives:

- **`design_space.rs`**: Coordinate system conversions
- **`point.rs`**: Point representation
- **`quadrant.rs`**: Quadrant calculations

### Rendering (`src/rendering/`)
Visual output:

- **`glyph_outline.rs`**: Glyph drawing
- **`cameras.rs`**: Camera management
- **`checkerboard.rs`**: Background grid
- **`metrics.rs`**: Font metrics visualization

### Tools (`src/tools/`)
User interaction tools - **NEW SIMPLIFIED STRUCTURE**:

- **`select.rs`**: Selection and manipulation tool (V key)
- **`pen.rs`**: Drawing tool for creating contours (P key)
- **`knife.rs`**: Path cutting tool (K key)
- **`text.rs`**: Text/sort placement tool (T key)
- **`pan.rs`**: Navigation tool (Space key)
- **`measure.rs`**: Distance and angle measurement (M key)
- **`shapes.rs`**: Geometric shape creation (S key)
- Each tool implements the `EditTool` trait with consistent interface

### UI Components (`src/ui/`)
User interface:

- **`theme.rs`**: Visual constants (colors, sizes)
- **`themes/`**: Theme definitions
- **`panes/`**: UI panels
  - `glyph_pane.rs`: Glyph selection
  - `coord_pane.rs`: Coordinate display
  - `design_space.rs`: Main editing area

## Key Design Patterns

### 1. Entity-Component-System (ECS)
Bevy's ECS pattern separates data from behavior:

```rust
// Component (data)
#[derive(Component)]
struct Selected;

// System (behavior)
fn selection_system(
    selected: Query<Entity, With<Selected>>,
    // ...
) {
    // Process selected entities
}
```

### 2. Plugin Architecture
Features are organized as plugins:

```rust
pub struct MyFeaturePlugin;

impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system)
           .init_resource::<MyResource>();
    }
}
```

### 3. Event-Driven Input
Input flows through events:

```rust
InputEvent::MouseClick { position, button, modifiers }
    → InputConsumer (prioritizes handlers)
    → Tool/System handles event
    → State updates
    → Visual feedback
```

### 4. Resource Pattern
Shared state uses Bevy Resources:

```rust
#[derive(Resource)]
struct AppState {
    pub workspace: Workspace,
    pub current_glyph: Option<String>,
}
```

## Common Tasks

### Adding a New Tool

1. Create `src/tools/your_tool.rs`:
```rust
use crate::tools::{EditTool, ToolInfo};

pub struct YourTool;

impl EditTool for YourTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "your_tool",
            display_name: "Your Tool",
            icon: "✏️",
            tooltip: "Does something cool",
            shortcut: Some(KeyCode::KeyY),
        }
    }
    
    fn on_activate(&mut self, commands: &mut Commands) {
        // Setup when tool is selected
    }
    
    fn on_deactivate(&mut self, commands: &mut Commands) {
        // Cleanup when switching tools
    }
}
```

2. Register in `src/tools/mod.rs`
3. Tool automatically appears in toolbar

### Adding a Theme

1. Create `src/ui/themes/your_theme.rs`:
```rust
use crate::ui::theme::Theme;

pub fn your_theme() -> Theme {
    Theme {
        name: "Your Theme",
        background_color: Color::srgb(0.1, 0.1, 0.1),
        grid_color: Color::srgba(0.3, 0.3, 0.3, 0.5),
        // ... other colors
    }
}
```

2. Register in `src/ui/theme.rs`
3. Theme appears in settings

### Adding a Glyph Operation

1. Create operation in `src/editing/operations/`:
```rust
pub fn your_operation(
    glyph: &mut Glyph,
    params: YourParams,
) -> Result<(), EditError> {
    // Modify glyph
    Ok(())
}
```

2. Create command in `src/systems/commands.rs`
3. Hook up to tool or keyboard shortcut

## Performance Considerations

- **System Ordering**: Critical systems run in specific order
- **Change Detection**: Use Bevy's change detection to avoid unnecessary work
- **Gizmos**: Debug visualizations use immediate-mode gizmos
- **Batching**: Group similar operations together

## Testing

- Unit tests: In each module (`#[cfg(test)]`)
- Integration tests: In `tests/` directory
- Visual tests: Run with `--debug` flag

## Debugging

- **Debug overlays**: `--debug` flag enables visual debugging
- **Logging**: `RUST_LOG=bezy=debug` for detailed logs
- **Inspector**: Bevy inspector UI for entity inspection

## Best Practices

1. **Keep files under 500 lines** for LLM context windows
2. **Use descriptive names** - clarity over brevity
3. **Document complex algorithms** inline
4. **Follow Bevy patterns** - Resources, Components, Systems
5. **Centralize constants** in theme files
6. **Test edge cases** - empty glyphs, missing data, etc.

## Getting Help

- **Code questions**: Check module documentation
- **Architecture questions**: This file and module structure
- **Bevy questions**: [Bevy Book](https://bevyengine.org/learn/book/)
- **Font questions**: UFO spec and FontForge docs