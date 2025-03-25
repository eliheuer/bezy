//! Coordinate Pane Module
//! 
//! This module implements a floating panel that displays coordinates and dimensions of selected elements.
//! The pane consists of two main components:
//! 
//! 1. Coordinate Display:
//!    - Shows X, Y coordinates of the selected reference point
//!    - Shows Width (W) and Height (H) of the selection
//!    - Values update in real-time as selection changes
//!    - Values are hidden when nothing is selected
//! 
//! 2. Quadrant Selector:
//!    - A 3x3 grid of circular buttons
//!    - Allows choosing which point of the selection to use as reference
//!    - Examples: top-left, center, bottom-right, etc.
//!    - Always visible, even when nothing is selected
//!    - Selected quadrant highlighted in orange
//!
//! Visual Layout:
//! ```text
//! ┌─────────────────┐
//! │ X: 520         │  ← Coordinate display (shows when items selected)
//! │ Y: 8           │
//! │ W: 16          │
//! │ H: 16          │
//! │ ○ ○ ○         │  ← Quadrant selector (always visible)
//! │ ○ ● ○         │    (● = selected quadrant)
//! │ ○ ○ ○         │
//! └─────────────────┘
//! ```
//!
//! The pane automatically positions itself in the bottom-right corner of the window
//! and can be toggled with Ctrl+P.

use crate::quadrant::Quadrant;
use crate::theme::*;
use crate::selection::SelectionState;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::ui::UiRect;

// ===============================================================================
// CONSTANTS
// ===============================================================================

/// Size of the quadrant selector grid (width and height)
/// This determines the overall size of the 3x3 grid of circular buttons
const QUADRANT_GRID_SIZE: f32 = 128.0;

/// Radius of the individual circular buttons in the quadrant selector
/// Each button is a circle with this radius, spaced evenly in the grid
const QUADRANT_CIRCLE_RADIUS: f32 = 16.0;

/// Thickness of the border around the quadrant selector outline
/// Creates a square border that contains all nine quadrant buttons
const QUADRANT_OUTLINE_THICKNESS: f32 = 2.0;

/// Color for the currently selected quadrant button
/// Bright orange with high opacity for clear visibility
const QUADRANT_SELECTED_COLOR: Color = Color::srgba(1.0, 0.6, 0.1, 1.0);

/// Color for unselected quadrant buttons
/// Dark gray that's visible but not distracting
const QUADRANT_UNSELECTED_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 1.0);

/// Border color for the selected quadrant button
/// Lighter orange than the fill color for a subtle glow effect
const QUADRANT_SELECTED_OUTLINE_COLOR: Color = Color::srgba(1.0, 0.8, 0.5, 1.0);

/// Border color for unselected quadrant buttons
/// Light gray that provides subtle definition
const QUADRANT_UNSELECTED_OUTLINE_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 1.0);

/// Text color used when coordinate values are disabled
/// (i.e., when no elements are selected)
const TEXT_COLOR_DISABLED: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);

// ===============================================================================
// COMPONENTS & RESOURCES
// ===============================================================================

/// Resource that tracks the current state of coordinate selection and display
/// This is updated whenever the selection changes and drives the UI updates
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CoordinateSelection {
    /// Number of elements currently selected
    /// Used to determine if coordinates should be shown (count > 0)
    pub count: usize,
    
    /// Currently active quadrant that determines which reference point to use
    /// This affects which point of the selection bounds is used for X/Y coordinates
    pub quadrant: Quadrant,
    
    /// Bounding rectangle that encompasses all selected elements
    /// Used to calculate coordinates and dimensions
    pub frame: Rect,
}

/// Marker component for the main coordinate pane container
/// This is the root node that contains both the coordinate display and quadrant selector
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordPane;

/// Marker for the container that holds all coordinate value displays
/// This container is collapsed when nothing is selected
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordValuesContainer;

/// Marker components for the different coordinate value text elements
/// These are used to update the specific text values when coordinates change
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct XValue;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct YValue;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WidthValue;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HeightValue;

/// Marker for the quadrant selector widget container
/// This contains the 3x3 grid of circular buttons
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct QuadrantSelector;

/// Component that associates a quadrant button with its position in the grid
/// The Quadrant enum value determines which reference point this button represents
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuadrantButton(pub Quadrant);

impl Default for QuadrantButton {
    fn default() -> Self {
        Self(Quadrant::Center) // Center is the default reference point
    }
}

/// Marker components for the coordinate display rows
/// Used to control visibility of individual coordinate rows
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct XCoordinateRow;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct YCoordinateRow;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WidthCoordinateRow;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HeightCoordinateRow;

/// Component that stores the type of coordinate value
/// Used to determine which value to display in text elements
#[derive(Component)]
struct CoordinateValue(String);

// ===============================================================================
// PLUGIN IMPLEMENTATION
// ===============================================================================

/// Plugin that adds coordinate pane functionality to the application
/// 
/// This plugin handles:
/// 1. Component and resource registration
/// 2. Initial UI setup
/// 3. Coordinate calculation and display systems
/// 4. User interaction systems
pub struct CoordinatePanePlugin;

impl Plugin for CoordinatePanePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all component types with Bevy's reflection system
            // This enables debugging and serialization of these types
            .register_type::<CoordPane>()
            .register_type::<CoordinateSelection>()
            .register_type::<XValue>()
            .register_type::<YValue>()
            .register_type::<WidthValue>()
            .register_type::<HeightValue>()
            .register_type::<QuadrantSelector>()
            .register_type::<QuadrantButton>()
            .register_type::<XCoordinateRow>()
            .register_type::<YCoordinateRow>()
            .register_type::<WidthCoordinateRow>()
            .register_type::<HeightCoordinateRow>()
            .register_type::<CoordValuesContainer>()
            
            // Register the Quadrant enum for reflection
            .register_type::<Quadrant>()
            
            // Initialize the coordinate selection resource with default values
            .init_resource::<CoordinateSelection>()
            
            // Add systems to the application
            .add_systems(Startup, spawn_coord_pane)  // Creates initial UI
            .add_systems(
                Update,
                (
                    // Core functionality
                    display_selected_coordinates,     // Calculates coordinates from selection
                    
                    // UI update systems
                    update_coord_pane_ui,            // Updates visibility and text values
                    handle_quadrant_selection,        // Handles quadrant button clicks
                    update_coord_pane_layout,         // Adjusts layout based on selection
                    update_coordinate_display,        // Updates coordinate text display
                    
                    // User interaction
                    toggle_coord_pane_visibility,     // Handles Ctrl+P shortcut
                ),
            );
    }
}

// ===============================================================================
// UI CONSTRUCTION
// ===============================================================================

/// Creates the coordinate pane and adds it to the UI
/// 
/// This function sets up the entire coordinate pane hierarchy:
/// ```text
/// CoordPane (root)
/// ├── CoordValuesContainer
/// │   ├── XCoordinateRow
/// │   │   ├── Label ("X: ")
/// │   │   └── Value
/// │   ├── YCoordinateRow
/// │   │   ├── Label ("Y: ")
/// │   │   └── Value
/// │   ├── WidthCoordinateRow
/// │   │   ├── Label ("W: ")
/// │   │   └── Value
/// │   └── HeightCoordinateRow
/// │       ├── Label ("H: ")
/// │       └── Value
/// └── QuadrantSelector
///     └── Grid of 9 circular buttons
/// ```
fn spawn_coord_pane(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning coordinate pane");

    // Position the coordinate pane in the bottom-right corner
    // Using Auto for top/left prevents the pane from stretching
    let position = UiRect {
        right: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto,    // Prevents stretching
        left: Val::Auto,   // Prevents stretching
    };

    // Spawn the main coordinate pane container
    commands
        .spawn(create_widget_style(
            &asset_server,
            PositionType::Absolute,  // Makes the pane float over other UI
            position,
            CoordPane,
            "CoordinatePane",
        ))
        .with_children(|parent| {
            // Create the coordinate value displays (X, Y, W, H)
            spawn_coordinate_values(parent, &asset_server);
            
            // Create the quadrant selector grid
            spawn_quadrant_selector_widget(parent);
        });
}

/// Creates the container for coordinate value displays (X, Y, W, H)
/// 
/// This container holds four rows, one for each coordinate value.
/// Each row contains a label and a value text element.
/// The entire container is hidden when no elements are selected.
fn spawn_coordinate_values(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,    // Stack rows vertically
            align_items: AlignItems::Stretch,        // Stretch rows to container width
            width: Val::Percent(100.0),              // Take full width of parent
            margin: UiRect::bottom(Val::Px(4.0)),    // Space before quadrant selector
            ..default()
        },
        CoordValuesContainer,
        Name::new("CoordValuesContainer"),
    ))
    .with_children(|container| {
        // Create all coordinate rows in order
        spawn_coordinate_row(container, "X", XCoordinateRow, asset_server);
        spawn_coordinate_row(container, "Y", YCoordinateRow, asset_server);
        spawn_coordinate_row(container, "W", WidthCoordinateRow, asset_server);
        spawn_coordinate_row(container, "H", HeightCoordinateRow, asset_server);
    });
}

/// Creates a single coordinate row with label and value
/// 
/// Each row is structured as:
/// ```text
/// ┌─ CoordinateRow ──────────┐
/// │ Label ("X: ") │ Value    │
/// └──────────────────────────┘
/// ```
/// 
/// Parameters:
/// - label: The text to show ("X", "Y", "W", or "H")
/// - marker: Component to identify this row type
/// - asset_server: For loading fonts
fn spawn_coordinate_row<T: Component + Default>(
    parent: &mut ChildBuilder,
    label: &str,
    marker: T,
    asset_server: &Res<AssetServer>,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,      // Label and value side by side
            align_items: AlignItems::Center,         // Vertically center content
            margin: UiRect::bottom(Val::Px(4.0)),    // Space between rows
            width: Val::Percent(100.0),              // Take full width
            ..default()
        },
        Name::new(format!("{}CoordinateRow", label)),
        marker,
        Visibility::Hidden,                          // Initially hidden
    ))
    .with_children(|row| {
        // Create the label (e.g., "X: ")
        row.spawn((
            Node {
                margin: UiRect::right(Val::Px(4.0)), // Space between label and value
                ..default()
            },
            Text::new(format!("{}: ", label)),
            TextFont {
                font: asset_server.load(MONO_FONT_PATH),
                font_size: WIDGET_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),  // Dimmed label color
        ));

        // Create the value text with appropriate marker component
        match label {
            "X" => spawn_value_text(row, asset_server, XValue, "XValue"),
            "Y" => spawn_value_text(row, asset_server, YValue, "YValue"),
            "W" => spawn_value_text(row, asset_server, WidthValue, "WidthValue"),
            "H" => spawn_value_text(row, asset_server, HeightValue, "HeightValue"),
            _ => spawn_value_text(row, asset_server, XValue, "DefaultValue"),
        }
    });
}

/// Helper function to spawn a value text element
/// This reduces code duplication in spawn_coordinate_row
fn spawn_value_text<T: Component>(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    marker: T,
    name: &str,
) {
    parent.spawn((
        Text::new("0"),
        TextFont {
            font: asset_server.load(MONO_FONT_PATH),
            font_size: WIDGET_TEXT_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        marker,
        Name::new(name.to_string()),  // Convert to owned String
    ));
}

/// Creates the quadrant selector widget
/// 
/// This widget is a 3x3 grid of circular buttons that let the user
/// choose which point of the selection to use as the reference point.
/// 
/// Layout:
/// ```text
/// ┌─ QuadrantSelector ─┐
/// │ ○ ○ ○             │
/// │ ○ ● ○  ← Selected │
/// │ ○ ○ ○             │
/// └──────────────────┘
/// ```
fn spawn_quadrant_selector_widget(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(0.0)),   // Parent handles spacing
                padding: UiRect::all(Val::Px(4.0)),  // Internal padding
                width: Val::Px(QUADRANT_GRID_SIZE),  // Fixed size
                height: Val::Px(QUADRANT_GRID_SIZE),
                ..default()
            },
            BorderColor(WIDGET_BORDER_COLOR),
            BorderRadius::all(Val::Px(WIDGET_BORDER_RADIUS / 2.0)),
            QuadrantSelector,
            Name::new("QuadrantSelector"),
            Visibility::Visible,                      // Always visible
        ))
        .with_children(|parent| {
            spawn_quadrant_selector_grid(parent);
        });
}

/// Creates the grid container for the quadrant selector
/// 
/// This creates a square outline that contains the 3x3 grid of buttons.
/// The grid is centered within the outline with proper spacing.
fn spawn_quadrant_selector_grid(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(QUADRANT_GRID_SIZE),
            height: Val::Px(QUADRANT_GRID_SIZE),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Name::new("QuadrantSelectorContainer"),
    ))
    .with_children(|container| {
        // Create the square outline container
        container.spawn((
            Node {
                width: Val::Px(QUADRANT_GRID_SIZE - 50.0),  // Account for padding
                height: Val::Px(QUADRANT_GRID_SIZE - 50.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(QUADRANT_OUTLINE_THICKNESS)),
                ..default()
            },
            Name::new("QuadrantGrid"),
            BorderColor(QUADRANT_UNSELECTED_OUTLINE_COLOR),
        ))
        .with_children(|grid| {
            // Create the 3x3 grid of buttons
            for row_idx in 0..3 {
                spawn_quadrant_grid_row(grid, row_idx);
            }
        });
    });
}

/// Creates a single row in the quadrant selector grid
/// 
/// Each row contains three circular buttons, evenly spaced.
/// The buttons are mapped to specific quadrants based on their position.
fn spawn_quadrant_grid_row(parent: &mut ChildBuilder, row_idx: usize) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            height: Val::Px((QUADRANT_GRID_SIZE - 16.0) / 3.0),
            width: Val::Px(QUADRANT_GRID_SIZE - 16.0),
            ..default()
        },
        Name::new(format!("QuadrantRow_{}", row_idx)),
    ))
    .with_children(|row| {
        // Create three buttons for this row
        for col_idx in 0..3 {
            // Map grid position to quadrant type
            let quadrant = match (row_idx, col_idx) {
                (0, 0) => Quadrant::TopLeft,
                (0, 1) => Quadrant::Top,
                (0, 2) => Quadrant::TopRight,
                (1, 0) => Quadrant::Left,
                (1, 1) => Quadrant::Center,
                (1, 2) => Quadrant::Right,
                (2, 0) => Quadrant::BottomLeft,
                (2, 1) => Quadrant::Bottom,
                (2, 2) => Quadrant::BottomRight,
                _ => unreachable!(),
            };

            spawn_quadrant_button(row, quadrant);
        }
    });
}

/// Creates a single circular button in the quadrant selector
/// 
/// Each button is a circle with:
/// - Background color (orange when selected, gray when not)
/// - Border (lighter color than background)
/// - Interaction component for click handling
/// - QuadrantButton component to identify its position
fn spawn_quadrant_button(parent: &mut ChildBuilder, quadrant: Quadrant) {
    // Center is selected by default
    let is_selected = quadrant == Quadrant::Center;
    
    // Choose colors based on selection state
    let color = if is_selected {
        QUADRANT_SELECTED_COLOR
    } else {
        QUADRANT_UNSELECTED_COLOR
    };
    
    let outline_color = if is_selected {
        QUADRANT_SELECTED_OUTLINE_COLOR
    } else {
        QUADRANT_UNSELECTED_OUTLINE_COLOR
    };

    // Create a container for proper button spacing
    parent.spawn((
        Node {
            width: Val::Px((QUADRANT_GRID_SIZE - 16.0) / 3.0),
            height: Val::Px((QUADRANT_GRID_SIZE - 16.0) / 3.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Name::new(format!("QuadrantCell_{:?}", quadrant)),
    ))
    .with_children(|cell| {
        // Create the circular button
        cell.spawn((
            Node {
                width: Val::Px(QUADRANT_CIRCLE_RADIUS * 2.0),
                height: Val::Px(QUADRANT_CIRCLE_RADIUS * 2.0),
                border: UiRect::all(Val::Px(QUADRANT_OUTLINE_THICKNESS)),
                ..default()
            },
            BackgroundColor(color),
            BorderColor(outline_color),
            BorderRadius::all(Val::Px(QUADRANT_CIRCLE_RADIUS)),
            Interaction::default(),
            QuadrantButton(quadrant),
            Name::new(format!("QuadrantButton_{:?}", quadrant)),
        ));
    });
}

// ===============================================================================
// COORDINATE CALCULATION & DISPLAY SYSTEMS
// ===============================================================================

/// Calculates and updates coordinates for selected entities
/// 
/// This system:
/// 1. Checks if any entities are selected
/// 2. Gets the global transform (position) of each selected entity
/// 3. Calculates a bounding box that contains all selected entities
/// 4. Updates the CoordinateSelection resource with the new information
/// 
/// The bounding box is used to:
/// - Calculate X/Y coordinates based on the selected quadrant
/// - Determine the width and height of the selection
pub fn display_selected_coordinates(
    mut coord_selection: ResMut<CoordinateSelection>,
    selection_state: Option<Res<crate::selection::components::SelectionState>>,
    transforms: Query<&GlobalTransform>,
) {
    // Log selection state for debugging
    if let Some(state) = &selection_state {
        info!(
            "CoordPane: SelectionState is available with {} selected entities",
            state.selected.len()
        );

        if !state.selected.is_empty() {
            let entities: Vec<_> = state.selected.iter().collect();
            info!("CoordPane: Selected entities in SelectionState: {:?}", entities);
        }
    } else {
        info!("CoordPane: SelectionState resource is NOT available");
    }

    // Get number of selected entities
    let selected_count = selection_state
        .as_ref()
        .map_or(0, |state| state.selected.len());

    info!("CoordPane: Selection system running with {selected_count} selected entities");

    if selected_count > 0 {
        // Process selection if we have selected entities
        process_selected_entities(&selection_state, &transforms, &mut coord_selection);
    } else {
        // Clear selection data if nothing is selected
        clear_coordinate_selection(&mut coord_selection);
    }
}

/// Processes selected entities and updates the coordinate selection
/// 
/// This helper function:
/// 1. Collects positions of all selected entities
/// 2. Calculates a bounding rectangle that contains all positions
/// 3. Updates the coordinate selection resource with new data
fn process_selected_entities(
    selection_state: &Option<Res<crate::selection::components::SelectionState>>,
    transforms: &Query<&GlobalTransform>,
    coord_selection: &mut CoordinateSelection,
) {
    // Collect positions of all selected entities
    let mut positions = Vec::new();

    if let Some(state) = selection_state {
        for &entity in &state.selected {
            if let Ok(transform) = transforms.get(entity) {
                let pos = transform.translation().truncate();
                info!("CoordPane: Found position for entity {entity:?}: {:?}", pos);
                positions.push(pos);
            } else {
                info!("CoordPane: Selected entity {entity:?} has no transform");
            }
        }
    }

    if !positions.is_empty() {
        // Calculate bounding rectangle from positions
        let frame = calculate_bounding_rect(&positions);
        
        // Update selection resource
        coord_selection.count = selection_state.as_ref().map_or(0, |state| state.selected.len());
        coord_selection.frame = frame;

        info!(
            "Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}",
            coord_selection.count, coord_selection.quadrant, frame
        );
    } else {
        clear_coordinate_selection(coord_selection);
    }
}

/// Calculates a bounding rectangle containing all provided positions
/// 
/// This function finds the minimum and maximum X/Y coordinates
/// of all positions to create a rectangle that encompasses them all.
fn calculate_bounding_rect(positions: &[Vec2]) -> Rect {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    // Find the extremes of all positions
    for position in positions {
        min_x = min_x.min(position.x);
        min_y = min_y.min(position.y);
        max_x = max_x.max(position.x);
        max_y = max_y.max(position.y);
    }

    // Create a rectangle from the min/max points
    Rect::from_corners(
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, max_y),
    )
}

/// Clears the coordinate selection state
/// 
/// Called when no entities are selected to reset the display
fn clear_coordinate_selection(coord_selection: &mut CoordinateSelection) {
    info!("CoordPane: No selection - clearing coordinate display");
    coord_selection.count = 0;
    coord_selection.frame = Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
    info!(
        "Updating coordinate pane UI: count=0, quadrant={:?}, frame={:?}",
        coord_selection.quadrant, coord_selection.frame
    );
}

/// Updates the UI elements in the coordinate pane
/// 
/// This system:
/// 1. Updates visibility of coordinate rows based on selection
/// 2. Keeps the quadrant selector always visible
/// 3. Updates text values with current coordinates
fn update_coord_pane_ui(
    coord_selection: Res<CoordinateSelection>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
    mut visibility_query: Query<(&mut Visibility, Option<&XCoordinateRow>, Option<&YCoordinateRow>, Option<&WidthCoordinateRow>, Option<&HeightCoordinateRow>, Option<&QuadrantSelector>)>,
) {
    info!(
        "Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}",
        coord_selection.count, coord_selection.quadrant, coord_selection.frame
    );

    // Set visibility based on selection state
    let coord_visibility = if coord_selection.count == 0 {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    // Update visibility for all elements in a single pass
    for (mut visibility, is_x, is_y, is_w, is_h, is_quadrant) in visibility_query.iter_mut() {
        if is_quadrant.is_some() {
            // Quadrant selector is always visible
            *visibility = Visibility::Visible;
        } else if is_x.is_some() || is_y.is_some() || is_w.is_some() || is_h.is_some() {
            // Coordinate rows follow selection state
            *visibility = coord_visibility;
        }
    }

    // Update coordinate values if we have a selection
    if coord_selection.count > 0 {
        update_coordinate_values(&coord_selection, &mut text_queries);
    }
}

/// Updates the coordinate text values based on selection
/// 
/// This function:
/// 1. Gets the reference point based on selected quadrant
/// 2. Updates X/Y coordinates to show that point's position
/// 3. Updates W/H values to show selection dimensions
fn update_coordinate_values(
    coord_selection: &CoordinateSelection,
    text_queries: &mut ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
) {
    let frame = coord_selection.frame;

    // Get the reference point for the selected quadrant
    let point = get_quadrant_point(&frame, coord_selection.quadrant);

    // Update each coordinate value
    if let Ok(mut text) = text_queries.p0().get_single_mut() {
        let formatted = format_coord_value(point.x);
        info!("Setting X value to: {}", formatted);
        *text = Text::new(formatted);
    }

    if let Ok(mut text) = text_queries.p1().get_single_mut() {
        let formatted = format_coord_value(point.y);
        info!("Setting Y value to: {}", formatted);
        *text = Text::new(formatted);
    }

    if let Ok(mut text) = text_queries.p2().get_single_mut() {
        let formatted = format_coord_value(frame.width());
        info!("Setting Width value to: {}", formatted);
        *text = Text::new(formatted);
    }

    if let Ok(mut text) = text_queries.p3().get_single_mut() {
        let formatted = format_coord_value(frame.height());
        info!("Setting Height value to: {}", formatted);
        *text = Text::new(formatted);
    }
}

/// Calculates a point on the bounding rectangle based on quadrant
/// 
/// Given a rectangle and a quadrant, this returns the corresponding point:
/// - Center: Middle of the rectangle
/// - TopLeft: Upper-left corner
/// - Top: Middle of top edge
/// - etc.
fn get_quadrant_point(frame: &Rect, quadrant: Quadrant) -> Vec2 {
    match quadrant {
        Quadrant::Center => Vec2::new(
            (frame.min.x + frame.max.x) / 2.0,  // Center X
            (frame.min.y + frame.max.y) / 2.0,  // Center Y
        ),
        Quadrant::TopLeft => Vec2::new(frame.min.x, frame.max.y),
        Quadrant::Top => Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.max.y),
        Quadrant::TopRight => Vec2::new(frame.max.x, frame.max.y),
        Quadrant::Right => Vec2::new(frame.max.x, (frame.min.y + frame.max.y) / 2.0),
        Quadrant::BottomRight => Vec2::new(frame.max.x, frame.min.y),
        Quadrant::Bottom => Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.min.y),
        Quadrant::BottomLeft => Vec2::new(frame.min.x, frame.min.y),
        Quadrant::Left => Vec2::new(frame.min.x, (frame.min.y + frame.max.y) / 2.0),
    }
}

/// Updates coordinate text display based on selection state
///
/// This system updates text values and colors in the coordinate display
/// based on the current selection of entities.
fn update_coordinate_display(
    mut text_query: Query<(&mut Text, &mut TextColor, &CoordinateValue)>,
    selection_state: Res<SelectionState>,
    position_query: Query<&Transform, With<crate::selection::Selected>>,
) {
    let is_selected = !selection_state.selected.is_empty();
    
    // Calculate coordinates based on selection
    let (x, y, width, height) = if is_selected {
        calculate_selection_coordinates(&position_query)
    } else {
        (0.0, 0.0, 0.0, 0.0) // Default values when nothing is selected
    };

    // Update all coordinate text elements
    for (mut text, mut text_color, value_type) in text_query.iter_mut() {
        // Select the right value based on the coordinate type
        let value = match value_type.0.as_str() {
            "X" => x,
            "Y" => y,
            "W" => width,
            "H" => height,
            _ => 0.0,
        };

        // Update text and color
        *text = Text::new(format_coord_value(value));
        text_color.0 = if is_selected { TEXT_COLOR } else { TEXT_COLOR_DISABLED };
    }
}

/// Calculates coordinates for the current selection
fn calculate_selection_coordinates(
    position_query: &Query<&Transform, With<crate::selection::Selected>>
) -> (f32, f32, f32, f32) {
    // Get positions of all selected entities
    let positions: Vec<Vec2> = position_query
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();

    if !positions.is_empty() {
        // Find min/max bounds
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for pos in &positions {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
        }

        (min_x, min_y, max_x - min_x, max_y - min_y)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

// ===============================================================================
// INTERACTION & LAYOUT SYSTEMS
// ===============================================================================

/// Handles user interaction with the quadrant selector buttons
/// 
/// This system:
/// 1. Detects when a quadrant button is clicked
/// 2. Updates the selected quadrant in CoordinateSelection
/// 3. Updates the visual appearance of all buttons
/// 
/// When a button is clicked:
/// - It becomes highlighted in orange
/// - Its border gets a lighter orange glow
/// - Other buttons return to their unselected gray state
fn handle_quadrant_selection(
    interaction_query: Query<
        (&Interaction, &QuadrantButton),
        Changed<Interaction>,
    >,
    mut coord_selection: ResMut<CoordinateSelection>,
    mut quadrant_buttons: Query<(&mut BackgroundColor, &mut BorderColor, &QuadrantButton)>,
) {
    // Check for new button interactions
    for (interaction, quadrant_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            coord_selection.quadrant = quadrant_button.0;
        }
    }

    // Update button appearances when selection changes
    if coord_selection.is_changed() {
        for (mut background, mut border_color, quadrant_button) in quadrant_buttons.iter_mut() {
            if quadrant_button.0 == coord_selection.quadrant {
                // Selected button gets bright colors
                *background = BackgroundColor(QUADRANT_SELECTED_COLOR);
                *border_color = BorderColor(QUADRANT_SELECTED_OUTLINE_COLOR);
            } else {
                // Unselected buttons get muted colors
                *background = BackgroundColor(QUADRANT_UNSELECTED_COLOR);
                *border_color = BorderColor(QUADRANT_UNSELECTED_OUTLINE_COLOR);
            }
        }
    }
}

/// Adjusts the layout of the coordinate pane based on selection
/// 
/// This system manages the pane's layout in two states:
/// 
/// 1. When nothing is selected:
///    - Collapses the coordinate values section
///    - Makes the pane more compact
///    - Shows only the quadrant selector
/// 
/// 2. When elements are selected:
///    - Expands to show coordinate values
///    - Adjusts width to fit content
///    - Maintains consistent padding
fn update_coord_pane_layout(
    coord_selection: Res<CoordinateSelection>,
    mut coord_values_query: Query<&mut Node, With<CoordValuesContainer>>,
    mut coord_pane_query: Query<&mut Node, (With<CoordPane>, Without<CoordValuesContainer>)>,
) {
    // Only update when selection changes
    if !coord_selection.is_changed() {
        return;
    }

    // Update the coordinate values container layout
    if let Ok(mut node) = coord_values_query.get_single_mut() {
        if coord_selection.count == 0 {
            // Collapse when nothing is selected
            node.height = Val::Px(0.0);
            node.margin = UiRect::all(Val::Px(0.0));
            node.padding = UiRect::all(Val::Px(0.0));
        } else {
            // Expand when something is selected
            node.height = Val::Auto;
            node.margin = UiRect::bottom(Val::Px(0.0));
            node.padding = UiRect::all(Val::Px(0.0));
        }
    }

    // Update the main pane layout
    if let Ok(mut node) = coord_pane_query.get_single_mut() {
        if coord_selection.count == 0 {
            // Compact layout when just showing quadrant selector
            node.width = Val::Px(QUADRANT_GRID_SIZE + (WIDGET_PADDING * 2.0));
            node.padding = UiRect::all(Val::Px(WIDGET_PADDING));
        } else {
            // Expanded layout when showing coordinates
            node.width = Val::Auto;
            node.min_width = Val::Px(QUADRANT_GRID_SIZE + (WIDGET_PADDING * 2.0));
            node.padding = UiRect::all(Val::Px(WIDGET_PADDING));
        }
    }
}

/// Toggles the coordinate pane visibility with Ctrl+P
/// 
/// This system:
/// 1. Checks for the Ctrl+P key combination
/// 2. Toggles the pane's visibility when pressed
/// 3. Logs the visibility change for debugging
pub fn toggle_coord_pane_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut coord_pane_query: Query<&mut Visibility, With<CoordPane>>,
) {
    // Check for Ctrl+P combination
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyP) {
        for mut visibility in coord_pane_query.iter_mut() {
            // Toggle between visible and hidden
            *visibility = match *visibility {
                Visibility::Visible => {
                    info!("Hiding coordinate pane");
                    Visibility::Hidden
                }
                _ => {
                    info!("Showing coordinate pane");
                    Visibility::Visible
                }
            };
        }
    }
}

// ===============================================================================
// UTILITY FUNCTIONS
// ===============================================================================

/// Formats a coordinate value for display
/// 
/// This function formats numbers in two ways:
/// 1. Integers: Shown without decimal point (e.g., "42")
/// 2. Decimals: Shown with one decimal place (e.g., "42.5")
fn format_coord_value(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i32)
    } else {
        format!("{:.1}", value)
    }
}
