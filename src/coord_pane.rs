//! Coordinate Pane Module
//! 
//! This module implements a floating panel that displays coordinates of selected points
//! and provides a quadrant selector to choose reference points.
//!
//! The coordinate pane consists of two main parts:
//! 1. A display showing X, Y, Width, and Height values of selected points
//! 2. A quadrant selector grid allowing the user to choose which reference point to use
//!
//! When points are selected, the coordinate values update to show their position and dimensions.
//! The quadrant selector determines which point of the selection box is used as reference.

use crate::quadrant::Quadrant;
use crate::theme::*;
use crate::selection::SelectionState;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::ui::UiRect;

// ===============================================================================
// CONSTANTS
// ===============================================================================

/// Width for the coordinate pane when fully expanded
const COORD_PANE_WIDTH: f32 = 256.0;

/// Size of the quadrant grid for the selector
const QUADRANT_GRID_SIZE: f32 = 128.0;

/// Radius of the circles in the quadrant selector
const QUADRANT_CIRCLE_RADIUS: f32 = 16.0;

/// Border thickness for quadrant selector outline
const QUADRANT_OUTLINE_THICKNESS: f32 = 2.0;

/// Color for selected quadrant button (bright orange for visibility)
const QUADRANT_SELECTED_COLOR: Color = Color::srgba(1.0, 0.6, 0.1, 0.9);

/// Color for unselected quadrant buttons (dark gray, less prominent)
const QUADRANT_UNSELECTED_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.7);

/// Border color for selected quadrant button (bright outline for contrast)
const QUADRANT_SELECTED_OUTLINE_COLOR: Color = Color::srgba(1.0, 0.8, 0.5, 0.8);

/// Border color for unselected quadrant buttons (subtle outline)
const QUADRANT_UNSELECTED_OUTLINE_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.3);

/// Text color for disabled state (when no selection exists)
const TEXT_COLOR_DISABLED: Color = Color::srgba(0.6, 0.6, 0.6, 0.8);

// ===============================================================================
// COMPONENTS & RESOURCES
// ===============================================================================

/// Resource that stores information about the currently selected points
/// and how their coordinates should be displayed
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CoordinateSelection {
    /// Number of points currently selected
    pub count: usize,
    
    /// Which quadrant/reference point is currently active
    pub quadrant: Quadrant,
    
    /// Bounding rectangle of the current selection
    pub frame: Rect,
}

/// Marker component for the main coordinate pane container
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordPane;

/// Marker component for the container holding the coordinate value displays
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordValuesContainer;

/// Marker for the X coordinate value text element
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct XValue;

/// Marker for the Y coordinate value text element
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct YValue;

/// Marker for the Width value text element
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WidthValue;

/// Marker for the Height value text element
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HeightValue;

/// Marker component for the quadrant selector widget
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct QuadrantSelector;

/// Marker component with data for quadrant selector buttons
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuadrantButton(pub Quadrant);

impl Default for QuadrantButton {
    fn default() -> Self {
        Self(Quadrant::Center) // Default to center quadrant
    }
}

/// Marker for the X coordinate row in the display
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct XCoordinateRow;

/// Marker for the Y coordinate row in the display
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct YCoordinateRow;

/// Marker for the Width coordinate row in the display
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WidthCoordinateRow;

/// Marker for the Height coordinate row in the display
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HeightCoordinateRow;

/// Component to store coordinate value type for text elements
#[derive(Component)]
struct CoordinateValue(String);

// ===============================================================================
// PLUGIN IMPLEMENTATION
// ===============================================================================

/// Plugin that adds coordinate pane functionality to the application
pub struct CoordinatePanePlugin;

impl Plugin for CoordinatePanePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all component types with reflection system
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
            
            // Register enum types
            .register_type::<Quadrant>()
            
            // Initialize resources
            .init_resource::<CoordinateSelection>()
            
            // Add systems
            .add_systems(Startup, spawn_coord_pane)
            .add_systems(
                Update,
                (
                    // Selection handling and coordinate calculation
                    display_selected_coordinates,
                    
                    // UI update systems
                    update_coord_pane_ui,
                    handle_quadrant_selection,
                    update_coord_pane_layout,
                    update_coordinate_display,
                    
                    // User interaction
                    toggle_coord_pane_visibility,
                ),
            );
    }
}

// ===============================================================================
// UI CONSTRUCTION
// ===============================================================================

/// Creates the coordinate pane and adds it to the UI
/// 
/// This function spawns the main coordinate pane container in the bottom-right
/// corner of the screen, along with all of its child elements.
fn spawn_coord_pane(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning coordinate pane");

    // Position the coordinate pane in the bottom-right corner
    let position = UiRect {
        right: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto,    // Prevents stretching
        left: Val::Auto,   // Ensures correct sizing
        ..default()
    };

    // Spawn the main coordinate pane container
    commands
        .spawn(create_widget_style(
            &asset_server,
            PositionType::Absolute,
            position,
            CoordPane,
            "CoordinatePane",
        ))
        .with_children(|parent| {
            // Add the coordinate value display
            spawn_coordinate_values(parent, &asset_server);
            
            // Add the quadrant selector
            spawn_quadrant_selector_widget(parent);
        });
}

/// Creates the container for coordinate value displays (X, Y, W, H)
fn spawn_coordinate_values(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            width: Val::Percent(100.0),
            ..default()
        },
        CoordValuesContainer,
        Name::new("CoordValuesContainer"),
    ))
    .with_children(|container| {
        // Create all coordinate rows
        spawn_coordinate_row(container, "X", XCoordinateRow, asset_server);
        spawn_coordinate_row(container, "Y", YCoordinateRow, asset_server);
        spawn_coordinate_row(container, "W", WidthCoordinateRow, asset_server);
        spawn_coordinate_row(container, "H", HeightCoordinateRow, asset_server);
    });
}

/// Creates a single coordinate row with label and value
/// 
/// Each row consists of a label (e.g., "X: ") and a value text element.
/// The value element is tagged with the appropriate marker component.
fn spawn_coordinate_row<T: Component + Default>(
    parent: &mut ChildBuilder,
    label: &str,
    marker: T,
    asset_server: &Res<AssetServer>,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(4.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        Name::new(format!("{}CoordinateRow", label)),
        marker,
        Visibility::Hidden, // Initially hidden until selection
    ))
    .with_children(|row| {
        // Label component (e.g., "X: ")
        row.spawn((
            Node {
                margin: UiRect::right(Val::Px(4.0)),
                ..default()
            },
            Text::new(format!("{}: ", label)),
            TextFont {
                font: asset_server.load(MONO_FONT_PATH),
                font_size: WIDGET_TEXT_FONT_SIZE,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ));

        // Value component with appropriate marker
        // Spawn the appropriate value component based on the label
        match label {
            "X" => {
                row.spawn((
                    Text::new("0"),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: WIDGET_TEXT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    XValue,
                    Name::new("XValue"),
                ));
            },
            "Y" => {
                row.spawn((
                    Text::new("0"),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: WIDGET_TEXT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    YValue,
                    Name::new("YValue"),
                ));
            },
            "W" => {
                row.spawn((
                    Text::new("0"),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: WIDGET_TEXT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    WidthValue,
                    Name::new("WidthValue"),
                ));
            },
            "H" => {
                row.spawn((
                    Text::new("0"),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: WIDGET_TEXT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    HeightValue,
                    Name::new("HeightValue"),
                ));
            },
            _ => {
                row.spawn((
                    Text::new("0"),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: WIDGET_TEXT_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    XValue, // Default to XValue
                    Name::new("DefaultValue"),
                ));
            }
        }
    });
}

/// Creates the quadrant selector widget in the coordinate pane
fn spawn_quadrant_selector_widget(parent: &mut ChildBuilder) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(8.0)), // Equal margins on all sides
                padding: UiRect::all(Val::Px(4.0)),
                width: Val::Px(QUADRANT_GRID_SIZE + 16.0), // Adjusted for margins
                height: Val::Px(QUADRANT_GRID_SIZE + 16.0), // Adjusted for margins
                ..default()
            },
            BorderColor(WIDGET_BORDER_COLOR),
            BorderRadius::all(Val::Px(WIDGET_BORDER_RADIUS / 2.0)),
            QuadrantSelector,
            Name::new("QuadrantSelector"),
            Visibility::Visible, // Always visible
        ))
        .with_children(|parent| {
            // Create the grid of quadrant selectors
            spawn_quadrant_selector_grid(parent);
        });
}

/// Creates the grid container for the quadrant selector
fn spawn_quadrant_selector_grid(parent: &mut ChildBuilder) {
    // Outer container with fixed dimensions
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
        // Inner container with border outline
        container.spawn((
            Node {
                width: Val::Px(QUADRANT_GRID_SIZE - 16.0), // Account for padding
                height: Val::Px(QUADRANT_GRID_SIZE - 16.0), // Account for padding
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
            // Create the 3x3 grid of quadrant buttons
            for row_idx in 0..3 {
                spawn_quadrant_grid_row(grid, row_idx);
            }
        });
    });
}

/// Creates a single row in the quadrant selector grid
fn spawn_quadrant_grid_row(parent: &mut ChildBuilder, row_idx: usize) {
    // Set up a row with fixed height and centered content
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
        // Create 3 quadrant buttons in this row
        for col_idx in 0..3 {
            // Map the grid coordinates to a quadrant
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

            // Create the button for this quadrant
            spawn_quadrant_button(row, quadrant);
        }
    });
}

/// Creates a single button in the quadrant selector grid
fn spawn_quadrant_button(parent: &mut ChildBuilder, quadrant: Quadrant) {
    // Determine if this is the default selected quadrant (Center)
    let is_selected = quadrant == Quadrant::Center;
    
    // Apply appropriate styling based on selection state
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

    // Create a container to ensure proper spacing
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
        // Create the circular quadrant button
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
            Interaction::default(), // Make it interactive
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
/// 1. Gets all selected entities from the SelectionState resource
/// 2. Calculates the bounding box containing all selected entities
/// 3. Updates the CoordinateSelection resource with the new information
pub fn display_selected_coordinates(
    mut coord_selection: ResMut<CoordinateSelection>,
    selection_state: Option<Res<crate::selection::components::SelectionState>>,
    transforms: Query<&GlobalTransform>,
) {
    // Check if selection state is available
    if let Some(state) = &selection_state {
        info!(
            "CoordPane: SelectionState is available with {} selected entities",
            state.selected.len()
        );

        // Log selected entities for debugging
        if !state.selected.is_empty() {
            let entities: Vec<_> = state.selected.iter().collect();
            info!("CoordPane: Selected entities in SelectionState: {:?}", entities);
        }
    } else {
        info!("CoordPane: SelectionState resource is NOT available");
    }

    // Get the number of selected entities
    let selected_count = selection_state
        .as_ref()
        .map_or(0, |state| state.selected.len());

    info!("CoordPane: Selection system running with {selected_count} selected entities");

    if selected_count > 0 {
        // Process selection only if we have selected entities
        process_selected_entities(&selection_state, &transforms, &mut coord_selection);
    } else {
        // Clear selection data if nothing is selected
        clear_coordinate_selection(&mut coord_selection);
    }
}

/// Processes selected entities and updates the coordinate selection
/// 
/// Helper function that handles the logic of calculating the bounding box
/// of selected entities and updating the coordinate selection resource.
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
        // Create a bounding rect from all positions
        let frame = calculate_bounding_rect(&positions);
        
        // Update the selection resource
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
fn calculate_bounding_rect(positions: &[Vec2]) -> Rect {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for position in positions {
        min_x = min_x.min(position.x);
        min_y = min_y.min(position.y);
        max_x = max_x.max(position.x);
        max_y = max_y.max(position.y);
    }

    Rect::from_corners(
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, max_y),
    )
}

/// Clears the coordinate selection state (for when nothing is selected)
fn clear_coordinate_selection(coord_selection: &mut CoordinateSelection) {
    info!("CoordPane: No selection - clearing coordinate display");
    coord_selection.count = 0;
    coord_selection.frame = Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
    info!(
        "Updating coordinate pane UI: count=0, quadrant={:?}, frame={:?}",
        coord_selection.quadrant, coord_selection.frame
    );
}

/// Updates the UI elements in the coordinate pane based on selection state
/// 
/// This system:
/// 1. Updates visibility of coordinate rows based on selection state
/// 2. Always keeps the quadrant selector visible
/// 3. Updates text values with calculated coordinates
fn update_coord_pane_ui(
    coord_selection: Res<CoordinateSelection>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
    mut row_queries: ParamSet<(
        Query<&mut Visibility, With<XCoordinateRow>>,
        Query<&mut Visibility, With<YCoordinateRow>>,
        Query<&mut Visibility, With<WidthCoordinateRow>>,
        Query<&mut Visibility, With<HeightCoordinateRow>>,
        Query<&mut Visibility, With<QuadrantSelector>>,
    )>,
) {
    // Log the current selection state
    info!(
        "Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}",
        coord_selection.count, coord_selection.quadrant, coord_selection.frame
    );

    // Set visibility based on whether anything is selected
    let coord_visibility = if coord_selection.count == 0 {
        Visibility::Hidden // Hide coordinate rows when nothing is selected
    } else {
        Visibility::Visible // Show when something is selected
    };

    // Update visibility for all coordinate rows directly
    if let Ok(mut visibility) = row_queries.p0().get_single_mut() {
        *visibility = coord_visibility;
    }
    if let Ok(mut visibility) = row_queries.p1().get_single_mut() {
        *visibility = coord_visibility;
    }
    if let Ok(mut visibility) = row_queries.p2().get_single_mut() {
        *visibility = coord_visibility;
    }
    if let Ok(mut visibility) = row_queries.p3().get_single_mut() {
        *visibility = coord_visibility;
    }
    
    // Always keep the quadrant selector visible
    if let Ok(mut visibility) = row_queries.p4().get_single_mut() {
        *visibility = Visibility::Visible;
    }

    // Only update coordinate values if we have a selection
    if coord_selection.count > 0 {
        update_coordinate_values(&coord_selection, &mut text_queries);
    }
}

/// Updates the coordinate text values based on the current selection
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

    // Calculate the reference point based on selected quadrant
    let point = get_quadrant_point(&frame, coord_selection.quadrant);

    // Update X coordinate text
    if let Ok(mut text) = text_queries.p0().get_single_mut() {
        let formatted = format_coord_value(point.x);
        info!("Setting X value to: {}", formatted);
        *text = Text::new(formatted);
    }

    // Update Y coordinate text
    if let Ok(mut text) = text_queries.p1().get_single_mut() {
        let formatted = format_coord_value(point.y);
        info!("Setting Y value to: {}", formatted);
        *text = Text::new(formatted);
    }

    // Update Width text
    if let Ok(mut text) = text_queries.p2().get_single_mut() {
        let formatted = format_coord_value(frame.width());
        info!("Setting Width value to: {}", formatted);
        *text = Text::new(formatted);
    }

    // Update Height text
    if let Ok(mut text) = text_queries.p3().get_single_mut() {
        let formatted = format_coord_value(frame.height());
        info!("Setting Height value to: {}", formatted);
        *text = Text::new(formatted);
    }
}

/// Calculates a point on the bounding rectangle based on the selected quadrant
fn get_quadrant_point(frame: &Rect, quadrant: Quadrant) -> Vec2 {
    match quadrant {
        Quadrant::Center => Vec2::new(
            (frame.min.x + frame.max.x) / 2.0,
            (frame.min.y + frame.max.y) / 2.0,
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

// ===============================================================================
// INTERACTION & LAYOUT SYSTEMS
// ===============================================================================

/// Handles user interaction with the quadrant selector buttons
/// 
/// This system:
/// 1. Detects when a quadrant button is clicked
/// 2. Updates the selected quadrant in the CoordinateSelection resource
/// 3. Updates the visual appearance of all quadrant buttons
fn handle_quadrant_selection(
    interaction_query: Query<
        (&Interaction, &QuadrantButton),
        Changed<Interaction>,
    >,
    mut coord_selection: ResMut<CoordinateSelection>,
    mut quadrant_buttons: Query<(&mut BackgroundColor, &mut BorderColor, &QuadrantButton)>,
) {
    // First, check for new interactions
    for (interaction, quadrant_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Update the selected quadrant when a button is clicked
            coord_selection.quadrant = quadrant_button.0;
        }
    }

    // Then update the visual state of all buttons
    if coord_selection.is_changed() {
        for (mut background, mut border_color, quadrant_button) in quadrant_buttons.iter_mut() {
            if quadrant_button.0 == coord_selection.quadrant {
                // Selected - use bright color
                *background = BackgroundColor(QUADRANT_SELECTED_COLOR);
                *border_color = BorderColor(QUADRANT_SELECTED_OUTLINE_COLOR);
            } else {
                // Not selected - use darker color
                *background = BackgroundColor(QUADRANT_UNSELECTED_COLOR);
                *border_color = BorderColor(QUADRANT_UNSELECTED_OUTLINE_COLOR);
            }
        }
    }
}

/// Adjusts the layout of the coordinate pane based on selection state
/// 
/// When no points are selected, this system collapses the coordinate value
/// section and makes the pane more compact, showing only the quadrant selector.
fn update_coord_pane_layout(
    coord_selection: Res<CoordinateSelection>,
    mut coord_values_query: Query<&mut Node, With<CoordValuesContainer>>,
    mut coord_pane_query: Query<&mut Node, (With<CoordPane>, Without<CoordValuesContainer>)>,
) {
    // Only update when the selection state changes
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
            node.margin = UiRect::bottom(Val::Px(8.0));
            node.padding = UiRect::all(Val::Px(4.0));
        }
    }

    // Update the main pane width
    if let Ok(mut node) = coord_pane_query.get_single_mut() {
        if coord_selection.count == 0 {
            // Make more square when just showing quadrant selector
            node.width = Val::Px(QUADRANT_GRID_SIZE + 24.0);
        } else {
            // Use full width when showing coordinates
            node.width = Val::Px(COORD_PANE_WIDTH);
        }
    }
}

/// Toggles the visibility of the coordinate pane with keyboard shortcut (Ctrl+P)
pub fn toggle_coord_pane_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut coord_pane_query: Query<&mut Visibility, With<CoordPane>>,
) {
    // Check for Ctrl+P key combination
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
// UTILITY FUNCTIONS
// ===============================================================================

/// Formats a coordinate value for display
/// 
/// If the value has no fractional part, formats it as an integer.
/// Otherwise, formats it with one decimal place.
fn format_coord_value(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i32)
    } else {
        format!("{:.1}", value)
    }
}
