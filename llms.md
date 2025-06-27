# Bezy Font Editor llms.txt File

> Bezy is a free and open-source (GPL licensed) cross-platform font editor built with the Bevy game engine, Rust, and Linebender crates. It started as a port of Runebender but has evolved into a unique approach to font editing. Unlike traditional font editors that separate glyph overview grids and text layout, Bezy provides a unified workspace where everything happens on one large grid, allowing visual proofing and glyphs managment to occur directly in one view. It also will be able to operate as a CLI tool in headless mode. It has AI-agent chat features and accepts micro payments with stable coins using the Base Ethereum L2. It has a text editor with a vim mode and a DrawBot scriptable visual layout and image/animation tool. It's basicly a minimalist swiss army knife of a font editor build in the tradion of American Graphic Modernism. It supports font editiong for both LRT scripts like Latin and RTL sscripts like Arabic.

## Core Philosophy & Distinguishing Features

**Sorts System**: Bezy's key innovation is the "sorts" system - named after metal typesetting pieces. Sorts are movable type entities that can be arranged freely or in buffer layouts, acting like Lego-like building blocks for text composition. Each sort represents a glyph instance with its own position and state.

**Unified Grid Workspace**: Everything happens on one continuous design space rather than switching between overview and edit modes.

**Dual State Architecture**: The app maintains two representations of font data:

- **Runtime State**: Thread-safe, ECS-optimized structures for real-time editing

- **UFO Persistence**: Norad library used only for loading/saving, not runtime storage

## Coordinate Systems

Bezy uses a dual coordinate system fundamental to all operations:

- **Design Space**: Fixed coordinate space where glyphs and entities are described. Origin (0,0) is at the intersection of baseline and left sidebearing. This is the canonical coordinate system for font data. Font metrics like descender lines are at negative Y values (e.g., Y=-800).

- **Screen Space**: View coordinate space for rendering and user interaction, accounting for zoom, pan, and screen geometry.

- **ViewPort**: Handles transformations between spaces, including zoom level, pan offset, and Y-axis flipping (Y is always flipped in screen space).

### Critical Coordinate System Gotchas

**Camera Positioning**: The camera must be positioned to view the font design space, not just (0,0). Since font glyphs typically span from positive Y (ascenders) to negative Y (descenders), the camera should be centered around the font's typical glyph bounding box area.

**Click Detection vs Rendering**: Always ensure that click detection and visual rendering use the same coordinate transformations. A common bug pattern is when:
- Rendering uses: `world_pos + Vec2::new(0.0, descender)`
- Click detection uses: `sort_pos + Vec2::new(0.0, descender - offset)` 
This mismatch can cause clickable areas to be hundreds of units away from visual elements.

**Tolerance Values**: Click tolerances must account for coordinate system gaps. If font elements are rendered in design space but clicks are detected in screen space with incomplete transformations, tolerances may need to be much larger (1000+ units) than expected (30 units) to bridge coordinate system mismatches.

## Architecture Overview

**Bevy ECS Foundation**: Built on Bevy's Entity Component System with plugins for modular functionality. Key patterns:
- Resources for global state (AppState, ViewPort, SelectionState)
- Components for entity data (EditPoint, Sort, Selected)
- Systems for behavior and rendering
- Events for communication between systems

**Thread-Safe Design**: All runtime data structures are optimized for multi-threading and real-time editing operations.

## Code Style

- Rust files should be max 80 characters wide

- Readability and ease of modification for junior programmers prioritized over clever tricks

- Extensive use of type safety and validation

## Key Data Structures

**AppState**: Main resource containing the entire font workspace

**FontData**: Thread-safe representation of glyph data, contours, and points

**Sorts**: Movable type entities with two layout modes:
  - Buffer mode: Grid-based layout following text flow
  - Freeform mode: Free positioning in design space

**SortBuffer**: Gap buffer implementation for efficient text editing operations on both LTR scripts like Latin and RTL scripts like Arabic.

## Core Architecture Files

### Application Core
- [Main app setup](src/core/app.rs): Plugin registration, startup systems, resource initialization

- [Application state](src/core/state.rs): Thread-safe font data structures, UFO conversion, workspace management

- [CLI interface](src/core/cli.rs): Command line argument parsing and configuration

- [Settings management](src/core/settings.rs): User preferences and editor configuration

### Coordinate Systems & Geometry
- [Design space](src/ui/panes/design_space.rs): Core coordinate definitions, ViewPort implementation, DPoint/DVec2 types

- [Point management](src/geometry/point.rs): EditPoint, EntityId system for glyph components

- [Path handling](src/geometry/path.rs): Geometric path operations and curve manipulation

- [Point collections](src/geometry/point_list.rs): Efficient point collection management

### Data Management
- [UFO operations](src/data/ufo.rs): UFO file I/O, Unicode codepoint mapping, norad integration

- [Unicode utilities](src/data/unicode.rs): Character range handling and sorting

- [Workspace management](src/data/workspace.rs): Project-level data organization

### Editing System
- [Selection system](src/editing/selection/mod.rs): Multi-select, drag operations, selection state

- [Point editing](src/editing/selection/components.rs): Selectable, Selected, Hovered components

- [Point nudging](src/editing/selection/nudge.rs): Keyboard-based point movement with configurable increments

- [Undo/redo](src/editing/undo_plugin.rs): Command pattern for undoable operations

- [Sort management](src/editing/sort.rs): Sort entity lifecycle, active/inactive states

### Rendering System

- [Camera control](src/rendering/cameras.rs): 2D camera with pan/zoom using bevy_pancam

- [Glyph rendering](src/rendering/glyph_outline.rs): Vector outline drawing and visualization

- [Drawing primitives](src/rendering/draw.rs): Low-level drawing utilities

- [Font metrics](src/rendering/metrics.rs): Baseline, x-height, ascender/descender visualization

### User Interface

- [Design space pane](src/ui/panes/design_space.rs): Main editing viewport, coordinate handling

- [Glyph navigation](src/ui/panes/glyph_pane.rs): Glyph selection and browsing interface

- [Coordinate display](src/ui/panes/coord_pane.rs): Real-time coordinate information

- [Edit mode toolbar](src/ui/toolbars/edit_mode_toolbar/mod.rs): Dynamic tool system

- [Theme system](src/ui/theme.rs): Color schemes and visual styling

### System Organization
- [Plugin management](src/systems/plugins.rs): Plugin configuration and system organization
- [Command handling](src/systems/commands.rs): Keyboard shortcuts and command dispatch
- [UI interaction](src/systems/ui_interaction.rs): Mouse/keyboard input handling

## Plugin Architecture

Bezy follows a modular plugin system:
- **Core Plugins**: SelectionPlugin, UndoPlugin, CommandsPlugin
- **Rendering Plugins**: CameraPlugin, CheckerboardPlugin
- **UI Plugins**: DesignSpacePlugin, GlyphPanePlugin, EditModeToolbarPlugin
- **System Plugins**: BezySystems (main plugin bundler)

## Key Concepts for LLMs

**Entity Identification**: Points, guides, and components use EntityId with parent/index/kind structure
**State Synchronization**: Changes flow from user input → ECS components → AppState → UFO format
**Multi-Select Operations**: Selection system supports complex multi-entity operations
**Coordinate Transformation**: Always be aware of design space vs screen space conversions
**Sort Lifecycle**: Sorts can be active (editable) or inactive (rendered), only one active at a time
**Gap Buffer**: SortBuffer uses gap buffer for efficient text editing operations

## Common Patterns

- Resources for shared state, Components for entity data
- Systems communicate via Events, not direct calls
- Validation functions ensure data integrity
- Thread-safe wrappers around norad types
- Extensive use of Option<> for nullable values
- Builder patterns for complex initialization

## Debugging Guide for Coordinate Issues

When encountering click detection problems or visual/interaction mismatches:

1. **Add Debug Logging**: Use `info!` level logging to see:
   - Actual click world positions: `world_position = ({:.1}, {:.1})`
   - Element render positions: `element_position = ({:.1}, {:.1})`
   - Calculated distances: `distance = {:.1}, tolerance = {:.1}`

2. **Check Camera Setup**: Verify camera is positioned to view the design space:
   - Font elements typically range from Y=+800 (ascenders) to Y=-800 (descenders)
   - Camera should be centered around Y=0 or the font's center point

3. **Verify Coordinate Consistency**: Ensure rendering and interaction use identical calculations:
   - If rendering uses `pos + Vec2::new(0.0, descender)`, click detection must use the same
   - Avoid hardcoded offsets that aren't applied to both systems

4. **Tolerance Reality Check**: If distances are 1000+ units but tolerance is 30, either:
   - Fix the coordinate transformation to reduce distances
   - Increase tolerance to bridge legitimate coordinate gaps (as temporary measure)
   - Prefer fixing the root coordinate issue over inflating tolerance

5. **System Order**: Check if multiple click handling systems are interfering:
   - Use `grep_search` to find all systems handling mouse input
   - Verify only appropriate systems are loaded in app.rs 