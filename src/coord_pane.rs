//! The floating panel that displays coordinates of selected points.

use crate::quadrant::Quadrant;
use crate::theme::*; // Import all theme items
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::ui::{AlignItems, FlexDirection, PositionType};

/// Resource to store the current coordinate selection
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CoordinateSelection {
    /// The number of points selected
    pub count: usize,
    /// The current quadrant used for selection
    pub quadrant: Quadrant,
    /// The bounding box of the selection
    pub frame: Rect,
}

/// Marker component for the coordinate pane
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordPane;

/// Marker for X value text
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct XValue;

/// Marker for Y value text
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct YValue;

/// Marker for Width value text
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WidthValue;

/// Marker for Height value text
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HeightValue;

/// Marker component for quadrant selector
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct QuadrantSelector;

/// Marker component for quadrant selector buttons
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuadrantButton(pub Quadrant);

impl Default for QuadrantButton {
    fn default() -> Self {
        Self(Quadrant::Center)
    }
}

/// Plugin for coordinate pane functionality
pub struct CoordinatePanePlugin;

impl Plugin for CoordinatePanePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register the component type with the Reflect system
            .register_type::<CoordPane>()
            .register_type::<CoordinateSelection>()
            .register_type::<XValue>()
            .register_type::<YValue>()
            .register_type::<WidthValue>()
            .register_type::<HeightValue>()
            .register_type::<QuadrantSelector>()
            .register_type::<QuadrantButton>()
            // Register enums
            .register_type::<Quadrant>()
            // Initialize the coordinate selection resource
            .init_resource::<CoordinateSelection>()
            // Add systems to system sets
            .add_systems(Startup, spawn_coord_pane)
            .add_systems(
                Update,
                (
                    display_selected_coordinates,
                    update_coord_pane_ui,
                    handle_quadrant_selection,
                    toggle_coord_pane_visibility, // Allow toggling the pane with Ctrl+P
                ),
            );
    }
}

/// Debug system to log changes to selection and update UI with formatted values
fn update_coord_pane_ui(
    coord_selection: Res<CoordinateSelection>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
) {
    // Log the selection state
    info!(
        "Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}",
        coord_selection.count, coord_selection.quadrant, coord_selection.frame
    );

    // Update UI based on the selection state
    if coord_selection.count == 0 {
        // No selection - show zeros
        if let Ok(mut text) = text_queries.p0().get_single_mut() {
            *text = Text::new("0.0");
        }
        if let Ok(mut text) = text_queries.p1().get_single_mut() {
            *text = Text::new("0.0");
        }
        if let Ok(mut text) = text_queries.p2().get_single_mut() {
            *text = Text::new("0.0");
        }
        if let Ok(mut text) = text_queries.p3().get_single_mut() {
            *text = Text::new("0.0");
        }
    } else {
        let frame = coord_selection.frame;

        // Get the point based on the selected quadrant
        let point = match coord_selection.quadrant {
            Quadrant::Center => Vec2::new(
                (frame.min.x + frame.max.x) / 2.0,
                (frame.min.y + frame.max.y) / 2.0,
            ),
            Quadrant::TopLeft => Vec2::new(frame.min.x, frame.max.y),
            Quadrant::Top => {
                Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.max.y)
            }
            Quadrant::TopRight => Vec2::new(frame.max.x, frame.max.y),
            Quadrant::Right => {
                Vec2::new(frame.max.x, (frame.min.y + frame.max.y) / 2.0)
            }
            Quadrant::BottomRight => Vec2::new(frame.max.x, frame.min.y),
            Quadrant::Bottom => {
                Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.min.y)
            }
            Quadrant::BottomLeft => Vec2::new(frame.min.x, frame.min.y),
            Quadrant::Left => {
                Vec2::new(frame.min.x, (frame.min.y + frame.max.y) / 2.0)
            }
        };

        // Update UI values with precision and log the values being set
        if let Ok(mut text) = text_queries.p0().get_single_mut() {
            let formatted = format!("{:.1}", point.x);
            info!("Setting X value to: {}", formatted);
            *text = Text::new(formatted);
        }
        if let Ok(mut text) = text_queries.p1().get_single_mut() {
            let formatted = format!("{:.1}", point.y);
            info!("Setting Y value to: {}", formatted);
            *text = Text::new(formatted);
        }
        if let Ok(mut text) = text_queries.p2().get_single_mut() {
            let formatted = format!("{:.1}", frame.max.x - frame.min.x);
            info!("Setting Width value to: {}", formatted);
            *text = Text::new(formatted);
        }
        if let Ok(mut text) = text_queries.p3().get_single_mut() {
            let formatted = format!("{:.1}", frame.max.y - frame.min.y);
            info!("Setting Height value to: {}", formatted);
            *text = Text::new(formatted);
        }
    }
}

/// System to handle quadrant selection from UI
fn handle_quadrant_selection(
    interaction_query: Query<
        (&Interaction, &QuadrantButton),
        Changed<Interaction>,
    >,
    mut coord_selection: ResMut<CoordinateSelection>,
    mut quadrant_buttons: Query<(&mut BackgroundColor, &mut BorderColor, &QuadrantButton)>,
) {
    // First, handle any interactions
    for (interaction, quadrant_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Update the selected quadrant
            coord_selection.quadrant = quadrant_button.0;
        }
    }

    // Then, update the visual state of all buttons
    if coord_selection.is_changed() {
        for (mut background, mut border_color, quadrant_button) in quadrant_buttons.iter_mut() {
            // Set the color based on whether this is the selected quadrant
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

// Constants for quadrant selector styling
const QUADRANT_CIRCLE_RADIUS: f32 = 12.0;
const QUADRANT_GRID_SIZE: f32 = 100.0;
const QUADRANT_OUTLINE_THICKNESS: f32 = 2.0;

// Colors for quadrant selector (matching edit mode buttons)
const QUADRANT_SELECTED_COLOR: Color = Color::srgba(1.0, 0.6, 0.1, 0.9);  // Bright orange for selected
const QUADRANT_UNSELECTED_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.7); // Dark gray for unselected
const QUADRANT_SELECTED_OUTLINE_COLOR: Color = Color::srgba(1.0, 0.8, 0.5, 0.8); // Bright outline for selected
const QUADRANT_UNSELECTED_OUTLINE_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.3); // Subtle outline for unselected

/// Spawns the coordinate pane in the lower right corner
fn spawn_coord_pane(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning coordinate pane");

    // Create the position properties for the coord pane (bottom right)
    let position_props = UiRect {
        right: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto,  // Explicitly set top to Auto to prevent stretching
        left: Val::Auto, // Explicitly set left to Auto for correct sizing
        ..default()
    };

    commands
        .spawn(create_widget_style(
            &asset_server,
            PositionType::Absolute,
            position_props,
            CoordPane,
            "CoordinatePane",
        ))
        .with_children(|parent| {
            // Coordinate Editor Section
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        width: Val::Auto,
                        height: Val::Auto,
                        ..default()
                    },
                    Name::new("CoordinateEditor"),
                ))
                .with_children(|row| {
                    // X Label and Value
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            margin: UiRect::right(Val::Px(12.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Name::new("XCoordinate"),
                    ))
                    .with_children(|x_row| {
                        // X Label with value - using the label-value pair helper function
                        let label = "x";
                        let value = "0.0";
                        
                        // X Label
                        x_row.spawn((
                            Node {
                                margin: UiRect::right(Val::Px(4.0)),
                                width: Val::Auto,
                                ..default()
                            },
                            Text::new(label),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));

                        // X Value
                        x_row.spawn((
                            Text::new(value),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            XValue,
                            Name::new("XValue"),
                        ));
                    });

                    // Y Label and Value
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Name::new("YCoordinate"),
                    ))
                    .with_children(|y_row| {
                        // Y Label
                        y_row.spawn((
                            Node {
                                margin: UiRect::right(Val::Px(4.0)),
                                ..default()
                            },
                            Text::new("y"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));

                        // Y Value
                        y_row.spawn((
                            Text::new("0.0"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            YValue,
                            Name::new("YValue"),
                        ));
                    });
                });

            // Add size information for multi-selection
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Start,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                    Name::new("SizeInfo"),
                ))
                .with_children(|row| {
                    // Width Label and Value
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            margin: UiRect::right(Val::Px(12.0)),
                            ..default()
                        },
                        Name::new("WidthInfo"),
                    ))
                    .with_children(|w_row| {
                        // W Label
                        w_row.spawn((
                            Node {
                                margin: UiRect::right(Val::Px(4.0)),
                                ..default()
                            },
                            Text::new("w"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));

                        // Width Value
                        w_row.spawn((
                            Text::new("0.0"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            WidthValue,
                            Name::new("WidthValue"),
                        ));
                    });

                    // Height Label and Value
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Name::new("HeightInfo"),
                    ))
                    .with_children(|h_row| {
                        // H Label
                        h_row.spawn((
                            Node {
                                margin: UiRect::right(Val::Px(4.0)),
                                ..default()
                            },
                            Text::new("h"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                        ));

                        // Height Value
                        h_row.spawn((
                            Text::new("0.0"),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: WIDGET_TEXT_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            HeightValue,
                            Name::new("HeightValue"),
                        ));
                    });
                });

            // Add a Runebender-style quadrant selector 
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        width: Val::Px(QUADRANT_GRID_SIZE + 8.0), // Reduced padding
                        height: Val::Px(QUADRANT_GRID_SIZE + 8.0), // Reduced padding
                        ..default()
                    },
                    BorderColor(WIDGET_BORDER_COLOR),
                    BorderRadius::all(Val::Px(WIDGET_BORDER_RADIUS / 2.0)),
                    QuadrantSelector,
                    Name::new("QuadrantSelector"),
                ))
                .with_children(|parent| {
                    // Spawn the visual quadrant selector
                    spawn_quadrant_selector(parent);
                });
        });
}

/// Spawns a Runebender-style quadrant selector with circles at key points
/// This replaces the previous grid of button approach with a more visual representation
fn spawn_quadrant_selector(parent: &mut ChildBuilder) {
    // Base container for the selector
    parent.spawn((
        Node {
            width: Val::Px(QUADRANT_GRID_SIZE),
            height: Val::Px(QUADRANT_GRID_SIZE),
            ..default()
        },
        Name::new("QuadrantSelectorGrid"),
    ))
    .with_children(|grid| {
        // Spawn the outline rectangle first (as a background element)
        grid.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(QUADRANT_GRID_SIZE),
                height: Val::Px(QUADRANT_GRID_SIZE),
                border: UiRect::all(Val::Px(QUADRANT_OUTLINE_THICKNESS)),
                ..default()
            },
            BorderColor(QUADRANT_UNSELECTED_OUTLINE_COLOR),
            Name::new("QuadrantOutline"),
        ));

        // Define quadrant positions as a percentage of the grid
        let positions = [
            (Quadrant::TopLeft, 0.0, 0.0),     // Top-Left corner
            (Quadrant::Top, 0.5, 0.0),         // Top center
            (Quadrant::TopRight, 1.0, 0.0),    // Top-Right corner
            (Quadrant::Left, 0.0, 0.5),        // Left center
            (Quadrant::Center, 0.5, 0.5),      // Center
            (Quadrant::Right, 1.0, 0.5),       // Right center
            (Quadrant::BottomLeft, 0.0, 1.0),  // Bottom-Left corner
            (Quadrant::Bottom, 0.5, 1.0),      // Bottom center
            (Quadrant::BottomRight, 1.0, 1.0), // Bottom-Right corner
        ];

        // Spawn a circle for each quadrant position
        for (quadrant, x_pct, y_pct) in positions {
            // Calculate absolute position
            let x_pos = x_pct * QUADRANT_GRID_SIZE;
            let y_pos = y_pct * QUADRANT_GRID_SIZE;

            // Determine if this is the default selected quadrant (Center)
            let is_selected = quadrant == Quadrant::Center;
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

            // Spawn the quadrant circle button - using components directly
            grid.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(x_pos - QUADRANT_CIRCLE_RADIUS),
                    top: Val::Px(y_pos - QUADRANT_CIRCLE_RADIUS),
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
        }
    });
}

/// System to display coordinates for selected entities
/// Uses SelectionState resource directly to get the most accurate selection information
pub fn display_selected_coordinates(
    mut coord_selection: ResMut<CoordinateSelection>,
    selection_state: Option<Res<crate::selection::components::SelectionState>>,
    transforms: Query<&GlobalTransform>,
) {
    // Check if the SelectionState resource is available
    if let Some(state) = &selection_state {
        info!(
            "CoordPane: SelectionState is available with {} selected entities",
            state.selected.len()
        );

        // Log entities in the selection for debugging
        if !state.selected.is_empty() {
            let entities: Vec<_> = state.selected.iter().collect();
            info!(
                "CoordPane: Selected entities in SelectionState: {:?}",
                entities
            );
        }
    } else {
        info!("CoordPane: SelectionState resource is NOT available");
    }

    // Default to zero if SelectionState is not available
    let selected_count = selection_state
        .as_ref()
        .map_or(0, |state| state.selected.len());

    // Log the selection count for debugging
    info!("CoordPane: Selection system running with {selected_count} selected entities");

    if selected_count > 0 {
        // Collect positions of selected entities directly from SelectionState
        let mut positions = Vec::new();

        // Use the SelectionState directly instead of relying on Selected components
        if let Some(state) = &selection_state {
            for &entity in &state.selected {
                if let Ok(transform) = transforms.get(entity) {
                    let pos = transform.translation().truncate();
                    info!(
                        "CoordPane: Found position for entity {entity:?}: {:?}",
                        pos
                    );
                    positions.push(pos);
                } else {
                    // Log warning if entity has no transform
                    info!("CoordPane: Selected entity {entity:?} has no transform");
                }
            }
        }

        if !positions.is_empty() {
            // Create a bounding rect from all positions
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;

            for position in &positions {
                min_x = min_x.min(position.x);
                min_y = min_y.min(position.y);
                max_x = max_x.max(position.x);
                max_y = max_y.max(position.y);
            }

            let frame = Rect::from_corners(
                Vec2::new(min_x, min_y),
                Vec2::new(max_x, max_y),
            );
            coord_selection.count = selected_count;
            coord_selection.frame = frame;

            info!("Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}", 
                  selected_count, coord_selection.quadrant, frame);
        } else {
            // No valid positions found
            info!("CoordPane: No valid positions found for selected entities");
            coord_selection.count = 0;
            coord_selection.frame = Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
            info!("Updating coordinate pane UI: count=0, quadrant={:?}, frame={:?}", 
                  coord_selection.quadrant, coord_selection.frame);
        }
    } else {
        // No selection
        info!("CoordPane: No selection - clearing coordinate display");
        coord_selection.count = 0;
        coord_selection.frame = Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
        info!(
            "Updating coordinate pane UI: count=0, quadrant={:?}, frame={:?}",
            coord_selection.quadrant, coord_selection.frame
        );
    }
}

/// System to toggle the coordinate pane visibility with keyboard shortcut (Ctrl+P)
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
            // Toggle visibility between Visible and Hidden
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
