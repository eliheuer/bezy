# Bezy Font Editor llms.txt File

> Bezy is a free and open-source (GPL licensed) cross-platform font editor built with the Bevy game engine, Rust, and Linebender crates. It started as a port of Runebender but has evolved into a unique approach to font editing. Unlike traditional font editors that separate glyph overview grids and text layout, Bezy provides a unified workspace where everything happens on one large grid, allowing visual proofing and glyphs managment to occur directly in one view. It also will be able to operate as a CLI tool in headless mode. It has AI-agent chat features and accepts micro payments with stable coins using the Base Ethereum L2. It has a text editor with a vim mode and a DrawBot scriptable visual layout and image/animation tool. It's basicly a minimalist swiss army knife of a font editor build in the tradion of American Graphic Modernism. It supports font editiong for both LRT scripts like Latin and RTL sscripts like Arabic.

## Core Philosophy & Distinguishing Features

**Sorts System**: Bezy's key innovation is the "sorts" system - named after metal typesetting pieces. Sorts are movable type entities that can be arranged freely or in buffer layouts, acting like Lego-like building blocks for text composition. Each sort represents a glyph instance with its own position and state.

**Unified Grid Workspace**: Everything happens on one continuous design space rather than switching between overview and edit modes.

**Dual State Architecture**: The app maintains two representations of font data:

- **Runtime State**: Thread-safe, ECS-optimized structures for real-time editing

- **UFO Persistence**: Norad library used only for loading/saving, not runtime storage

## Input System Architecture

Bezy uses a centralized input system that provides consistent, predictable input handling across all tools and modes. This system eliminates the scattered input handling that was causing conflicts and coordinate system inconsistencies.

### Input System Components

**InputPlugin**: The main input system plugin that manages all input state and events.

**InputState**: Centralized resource containing the complete state of all input devices:
- Mouse state (position, buttons, wheel, motion)
- Keyboard state (keys, modifiers, text buffer)
- Gamepad state (future expansion)
- Current input mode
- UI consumption state

**InputEvent**: Event-driven input system that provides typed events for all input actions:
- MouseClick, MouseRelease, MouseDrag, MouseMove, MouseWheel
- KeyPress, KeyRelease, TextInput
- GamepadButtonPress, GamepadButtonRelease, GamepadAnalog

**InputConsumer**: Trait-based system for routing input events to appropriate handlers based on priority and mode.

### Input Priority System

The input system uses a clear priority hierarchy to determine which system handles input:

1. **High Priority**: UI elements, modals, text editor
2. **Mode-Specific**: Current active tool (select, pen, knife, etc.)
3. **Low Priority**: Camera control, default actions

### Input Modes

The system tracks the current input mode to route events appropriately:
- **Normal**: Default editing mode
- **Select**: Selection tool active
- **Pen**: Pen tool active
- **Knife**: Knife tool active
- **Shape**: Shape tool active
- **Hyper**: Hyper tool active
- **Text**: Text editing mode
- **Temporary**: Temporary tool mode

### Coordinate Systems

Bezy uses a three-tiered coordinate system, and understanding the transformation flow is critical. The new architecture centralizes this transformation to eliminate bugs.

- **Screen Space**: The 2D pixel coordinates of the application window, where the origin (0,0) is the top-left corner. This is where raw mouse events originate.

- **World Space**: Bevy's intermediate 2D coordinate system. The camera transforms `Screen Space` coordinates into `World Space`. This space is used by the engine for rendering, but for application logic, you should almost always use `Design Space`.

- **Design Space**: The canonical coordinate system for all font data. The origin (0,0) is at the intersection of the baseline and a glyph's left sidebearing. Ascenders are in positive Y, and descenders are in negative Y. All font geometry (points, contours, metrics) is defined in this space.

- **`CursorInfo` Resource (Single Source of Truth)**: To ensure consistency, all coordinate conversions are handled by a single system that updates the `CursorInfo` resource once per frame. This resource holds the cursor's up-to-date position in both `Screen Space` and `Design Space`. **This is the ONLY place systems should get cursor coordinates from.**

- **ViewPort**: This resource is now deprecated for coordinate transforms and should not be used. It may be removed in the future.

### Critical Coordinate System Gotchas

**NEVER Perform Manual Cursor Coordinate Transformations**: This is the most critical rule. Any system that needs the cursor's position in `Design Space` **must** get it by accessing the `Res<CursorInfo>` resource (e.g., `cursor_info.design`). Do not use `camera.viewport_to_world_2d()` or any other method yourself. The conversion is done once, centrally, and correctly.

**Camera Positioning**: The camera must be positioned to view the font design space, not just (0,0). Since font glyphs typically span from positive Y (ascenders) to negative Y (descenders), the camera should be centered around the font's typical glyph bounding box area.

**Checkerboard Zoom Logic (CRITICAL)**: The checkerboard grid scaling must follow the correct zoom-to-grid-size relationship:
- **ZOOMED OUT** (large projection scale, seeing more world space) → **LARGE grid squares** (better performance, fewer squares rendered)
- **ZOOMED IN** (small projection scale, seeing less world space) → **SMALL grid squares** (more detail for precise editing)

In Bevy's OrthographicProjection, `scale` represents how much world space is visible:
- LARGER scale = more world space visible = more ZOOMED OUT = need LARGER grid squares
- SMALLER scale = less world space visible = more ZOOMED IN = need SMALLER grid squares

**Zoom Threshold Logic**: When implementing zoom thresholds for grid size changes, ensure the array is ordered from highest zoom (most zoomed out) to lowest zoom (most zoomed in), with corresponding grid size multipliers that increase as zoom increases.

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

### Input System
- [Input system](src/core/input.rs): Centralized input state management, event generation, and coordinate handling

- [Input consumers](src/systems/input_consumer.rs): Priority-based input routing and tool-specific input handlers

- [Cursor Position](src/core/cursor.rs): Defines the `CursorInfo` resource and the plugin that centrally manages screen-to-design-space coordinate conversions.

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

## Input System Patterns

**Centralized Input State**: All input is managed through the `InputState` resource, which provides consistent access to mouse, keyboard, and gamepad state.

**Event-Driven Architecture**: Input events are generated once per frame and routed to appropriate consumers based on priority and mode.

**Priority-Based Routing**: Input events are handled by the highest priority consumer that can handle them, preventing conflicts between systems.

**Mode-Aware Processing**: The input system automatically routes events to the appropriate tool based on the current input mode.

**Coordinate Consistency**: All input events include both screen and design space coordinates, ensuring consistent coordinate handling across all tools.

**UI Consumption**: The system automatically detects when UI elements are consuming input and routes events accordingly.

## Input System Best Practices

**Use InputState for State Queries**: Always query the `InputState` resource for current input state rather than directly accessing Bevy's input resources.

**Handle Events in Consumers**: Implement the `InputConsumer` trait for new tools rather than creating separate input handling systems.

**Check Input Mode**: Always verify the current input mode before processing input events to avoid conflicts.

**Use Helper Functions**: Use the helper functions in `input::helpers` for common input checks rather than implementing them manually.

**Coordinate Consistency**: Always use the coordinates provided in input events rather than performing manual coordinate transformations.

**Priority Order**: Respect the input priority system - high priority consumers (UI, text editor) should handle input before mode-specific consumers.

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

## Migrating to the New Input System

When updating existing input handling code to use the new centralized input system:

### 1. Replace Direct Input Access

**Before (Old System)**:
```rust
fn handle_mouse_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let cursor_pos = window.cursor_position().unwrap();
        let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos).unwrap();
        // Handle click...
    }
}
```

**After (New System)**:
```rust
fn handle_mouse_input(
    mut input_events: EventReader<InputEvent>,
    input_state: Res<InputState>,
) {
    for event in input_events.read() {
        if let InputEvent::MouseClick { button, position, modifiers } = event {
            if *button == MouseButton::Left {
                // Handle click with position already in design space...
            }
        }
    }
}
```

### 2. Implement InputConsumer for New Tools

```rust
#[derive(Resource)]
pub struct MyToolInputConsumer;

impl InputConsumer for MyToolInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Check if this tool should handle the event
        matches!(event, InputEvent::MouseClick { .. }) && 
        helpers::is_input_mode(input_state, InputMode::MyTool)
    }

    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                // Handle the click
            }
            _ => {}
        }
    }
}
```

### 3. Use InputState for State Queries

**Before**:
```rust
let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || 
                   keyboard_input.pressed(KeyCode::ShiftRight);
```

**After**:
```rust
let shift_pressed = helpers::is_shift_pressed(&input_state);
```

### 4. Coordinate Handling

**Before**:
```rust
let cursor_pos = window.cursor_position().unwrap();
let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos).unwrap();
```

**After**:
```rust
if let Some(position) = helpers::get_mouse_design_position(&input_state) {
    // Use position directly - already in design space
}
```

### 5. Update System Registration

Remove old input handling systems and ensure the new input system plugins are registered:

```rust
// In app.rs
app.add_plugins((
    InputPlugin,
    InputConsumerPlugin,
    // ... other plugins
));
``` 