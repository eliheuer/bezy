# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bezy is a font editor built with Rust and the Bevy game engine. It's designed as a modern, user-empowerable font editing tool that follows modernist design principles. The codebase is intentionally simple and readable for education purposes.

## Key Technologies

- **Bevy**: ECS-based game engine for the UI framework
- **Norad**: UFO font format parsing and manipulation
- **Kurbo**: 2D curve mathematics
- **Rust**: Core programming language

## Development Commands

### Basic Development
```bash
# Run the application (default: Bezy Grotesk font, glyph 'a')
cargo run

# Run with specific UFO font
cargo run -- --load-ufo <path_to_ufo>

# Run with specific Unicode character
cargo run -- --test-unicode <hex_codepoint>

# Debug mode with verbose logging
cargo run -- --debug --log-level debug
```

### Code Quality
```bash
# Format code (max width: 80 characters per rustfmt.toml)
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

### Build Variants
```bash
# Development build with dynamic linking (faster compilation)
cargo run --features dev

# WASM build for web deployment
./build-wasm.sh
# or manually:
cargo run --target wasm32-unknown-unknown

# Release build
cargo build --release
```

## Architecture Overview

### Module Structure
- **`core/`**: Application initialization, CLI, settings, state management
- **`data/`**: UFO font data handling, Unicode utilities, workspace management
- **`editing/`**: Edit operations, selection system, undo/redo, text editing
- **`geometry/`**: Geometric primitives, paths, points, design space coordinates
- **`rendering/`**: Drawing systems, glyph outlines, visual feedback
- **`systems/`**: Bevy systems for input handling, UI interaction, commands
- **`ui/`**: User interface components, toolbars, panes, themes

### Key Architectural Patterns

#### ECS-Based Design
The application uses Bevy's Entity-Component-System pattern. Major systems include:
- **Selection System**: Manages point/component selection state
- **Edit System**: Handles glyph modifications and undo operations
- **Input System**: Processes keyboard/mouse events
- **Rendering System**: Draws glyphs, UI elements, and visual feedback

#### State Management
- **AppState**: Main application state resource
- **GlyphNavigation**: Current glyph and font navigation
- **BezySettings**: Configuration and preferences

#### Plugin Architecture
The application is composed of plugins:
- **Core plugins**: Input, pointer management, settings
- **UI plugins**: HUD, panes, toolbars
- **Editing plugins**: Selection, undo, text editing
- **Rendering plugins**: Cameras, drawing systems

### Design Philosophy

#### Visual Theming
All visual styling constants MUST be declared in `src/ui/theme.rs`. No visual constants should exist outside this file to enable complete theme swapping.

#### Code Style
- Max line width: 80 characters (enforced by rustfmt.toml)
- Simple, readable code suitable for educational purposes
- Modernist "less is more" approach
- Game-like feel with fast, engaging interactions

### Font Data Model

The application uses FontIR as the primary runtime data structure:
- **FontIR**: The single source of truth for font data that gets modified during editing
- **Data Flow**: Load font sources (UFO, TTF, OTF, etc.) → FontIR runtime structure → Edit FontIR data → Save back to disk
- **Transform Components**: Only for visual positioning in UI, NOT the source of truth
- **Critical**: When editing points, ALWAYS update the underlying FontIR glyph data, not just Transform positions

Legacy UFO support:
- **Font**: Container for all glyphs and metadata
- **Glyph**: Individual character containing contours and components
- **Contour**: Closed or open paths made of points
- **Point**: Curve or line points with coordinates and control handles

## Testing and Debugging

### Built-in Test Font
The repository includes "Bezy Grotesk" test font at `assets/fonts/bezy-grotesk-regular.ufo` with Latin and Arabic characters for testing.

### Development Features
- Debug overlays for geometry visualization
- Performance monitoring systems
- Coordinate display panes
- Real-time glyph editing feedback

## Build Scripts

- **`build-wasm.sh`**: Creates WASM build for web deployment
- **`build-github-pages.sh`**: Builds and deploys to GitHub Pages
- **`assets/fonts/build-*.sh`**: Font asset processing scripts

## Common Workflows

### Adding New Tools
The toolbar system uses a plugin-based architecture. New editing tools can be added by:
1. Creating a tool struct implementing `EditTool` trait
2. Adding plugin registration in the toolbar module
3. Tools automatically appear in UI with proper ordering

### Font Loading
UFO fonts are loaded through the `data::ufo` module which uses the Norad library for parsing and the workspace system for file management.

### Selection System
Point selection uses coordinate-based hit testing with visual feedback. The selection system supports:
- Single point selection
- Multi-select with Shift+click
- Box selection with click+drag
- Keyboard-based nudging with configurable amounts (2, 8, 32 units)

# critical-behavior-guidelines
NEVER make changes the user didn't explicitly ask for, even if they seem helpful.
NEVER change default behaviors, settings, or configurations unless specifically requested.
NEVER add "improvements" or "enhancements" that weren't requested.
ALWAYS ask for clarification if the request is unclear rather than making assumptions.
FOCUS entirely on the specific issue described by the user.
DO NOT be ambitious or make sweeping changes - be precise and minimal.
RESPECT the existing default tool (select) and user workflows.

# camera-zoom-scaling-direction
CRITICAL: Camera-responsive scaling relationship in src/rendering/camera_responsive.rs:
⚠️  I KEEP GETTING THIS WRONG - READ CAREFULLY ⚠️

ACTUAL PROVEN RELATIONSHIP (after multiple corrections):
- When USER ZOOMS OUT: camera_scale < 1.0, visual elements need to be BIGGER  
- When USER ZOOMS IN: camera_scale > 1.0, visual elements need to be SMALLER
- Normal zoom: camera_scale = 1.0

WORKING CODE PATTERN:
```rust
if camera_scale < 1.0 {
    // ZOOMED OUT: make elements BIGGER  
    base_responsive * 2.0
} else {
    // ZOOMED IN: make elements SMALLER
    base_responsive * 0.5  
}
```

DO NOT CHANGE THIS AGAIN - I have repeatedly gotten the camera_scale relationship backwards!