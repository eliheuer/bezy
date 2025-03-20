//! The floating panel that displays coordinates of selected points.

use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::text::TextFont;
use bevy::ui::{AlignItems, Display, FlexDirection, PositionType};
use crate::quadrant::Quadrant;

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

/// Marker for coordinate labels container
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoordinateLabelContainer;

/// Marker for size labels container
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SizeLabelContainer;

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
            .register_type::<CoordinateLabelContainer>()
            .register_type::<SizeLabelContainer>()
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
                    rebuild_coord_pane_on_selection_change, // System to rebuild the UI based on selection
                ),
            );
    }
}

/// System sets for Coordinate Pane systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CoordinatePaneSet {
    SyncSelection,
    UpdateUI,
}

/// System to update the coordinate pane UI values
fn update_coord_pane_ui(
    coord_selection: Res<CoordinateSelection>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
    mut quadrant_query: Query<(&mut BorderColor, &QuadrantButton)>,
) {
    if coord_selection.is_changed() {
        debug!("Updating coordinate pane UI: count={}, quadrant={:?}, frame={:?}", 
               coord_selection.count, coord_selection.quadrant, coord_selection.frame);
        
        // Only update UI values if there's a selection and visible UI elements
        if coord_selection.count > 0 {
            // Update coordinate values
            if let Ok(mut text) = text_queries.p0().get_single_mut() {
                *text = Text::new(format_coordinate(coord_selection.frame.min.x));
            }
            if let Ok(mut text) = text_queries.p1().get_single_mut() {
                *text = Text::new(format_coordinate(coord_selection.frame.min.y));
            }
            
            // Update size values
            if let Ok(mut text) = text_queries.p2().get_single_mut() {
                let width = coord_selection.frame.max.x - coord_selection.frame.min.x;
                *text = Text::new(format_coordinate(width));
            }
            if let Ok(mut text) = text_queries.p3().get_single_mut() {
                let height = coord_selection.frame.max.y - coord_selection.frame.min.y;
                *text = Text::new(format_coordinate(height));
            }
        }
        
        // Update quadrant buttons to highlight the active one
        let active_color = Color::srgba(1.0, 0.9, 0.2, 0.8); // Yellow highlight
        let inactive_color = Color::srgba(0.4, 0.4, 0.4, 0.4); // Gray for inactive
        
        for (mut border_color, button) in quadrant_query.iter_mut() {
            *border_color = if button.0 == coord_selection.quadrant {
                BorderColor(active_color)
            } else {
                BorderColor(inactive_color)
            };
        }
    }
}

/// System to rebuild the coordinate pane UI when selection changes
fn rebuild_coord_pane_on_selection_change(
    coord_selection: Res<CoordinateSelection>,
    current_pane: Query<Entity, With<CoordPane>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if coord_selection.is_changed() {
        // Only rebuild if the selection count changes from 0 to something or vice versa
        // We don't want to rebuild for every selection change, just when we need to show/hide the labels
        if let Ok(pane_entity) = current_pane.get_single() {
            // First, find if we have any children to remove
            commands.entity(pane_entity).despawn_descendants();
            
            // Rebuild the panel internals
            let panel_background_color = Color::srgba(0.1, 0.1, 0.1, 0.9);
            let text_color = Color::WHITE;
            let border_color = Color::srgba(1.0, 1.0, 1.0, 0.3);
            let border_radius = 4.0;
            let quadrant_button_size = 20.0;
            let quadrant_spacing = 2.0;
            
            let has_selection = coord_selection.count > 0;
            
            commands.entity(pane_entity).with_children(|parent| {
                // Only show coordinate editor if there's a selection
                if has_selection {
                    // Coordinate Editor Section
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Start,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        Name::new("CoordinateEditor"),
                        CoordinateLabelContainer,
                    ))
                    .with_children(|row| {
                        // X Label and Value
                        row.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                margin: UiRect::right(Val::Px(12.0)),
                                ..default()
                            },
                        ))
                        .with_children(|x_row| {
                            // X Label
                            x_row.spawn((
                                Text::new("x"),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    margin: UiRect::right(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                            
                            // X Value
                            x_row.spawn((
                                Text::new(format_coordinate(coord_selection.frame.min.x)),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(text_color),
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
                        ))
                        .with_children(|y_row| {
                            // Y Label
                            y_row.spawn((
                                Text::new("y"),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    margin: UiRect::right(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                            
                            // Y Value
                            y_row.spawn((
                                Text::new(format_coordinate(coord_selection.frame.min.y)),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(text_color),
                                YValue,
                                Name::new("YValue"),
                            ));
                        });
                    });

                    // Add size information for multi-selection
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Start,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        Name::new("SizeInfo"),
                        SizeLabelContainer,
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
                        ))
                        .with_children(|w_row| {
                            // W Label
                            w_row.spawn((
                                Text::new("w"),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    margin: UiRect::right(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                            
                            // Width Value
                            let width = coord_selection.frame.max.x - coord_selection.frame.min.x;
                            w_row.spawn((
                                Text::new(format_coordinate(width)),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(text_color),
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
                        ))
                        .with_children(|h_row| {
                            // H Label
                            h_row.spawn((
                                Text::new("h"),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    margin: UiRect::right(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                            
                            // Height Value
                            let height = coord_selection.frame.max.y - coord_selection.frame.min.y;
                            h_row.spawn((
                                Text::new(format_coordinate(height)),
                                TextFont {
                                    font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(text_color),
                                HeightValue,
                                Name::new("HeightValue"),
                            ));
                        });
                    });
                }

                // Add quadrant selector grid (3x3) - always visible
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            // Only add top margin if there's content above it
                            margin: if has_selection { UiRect::top(Val::Px(8.0)) } else { UiRect::all(Val::ZERO) },
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor(border_color),
                        BorderRadius::all(Val::Px(border_radius / 2.0)),
                        QuadrantSelector,
                    ))
                    .with_children(|grid| {
                        // Top row: TopLeft, Top, TopRight
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(quadrant_spacing),
                            margin: UiRect::bottom(Val::Px(quadrant_spacing)),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_quadrant_button(
                                row,
                                Quadrant::TopLeft,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::Top,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::TopRight,
                                quadrant_button_size,
                            );
                        });

                        // Middle row: Left, Center, Right
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(quadrant_spacing),
                            margin: UiRect::vertical(Val::Px(
                                quadrant_spacing / 2.0,
                            )),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_quadrant_button(
                                row,
                                Quadrant::Left,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::Center,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::Right,
                                quadrant_button_size,
                            );
                        });

                        // Bottom row: BottomLeft, Bottom, BottomRight
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(quadrant_spacing),
                            margin: UiRect::top(Val::Px(quadrant_spacing)),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_quadrant_button(
                                row,
                                Quadrant::BottomLeft,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::Bottom,
                                quadrant_button_size,
                            );
                            spawn_quadrant_button(
                                row,
                                Quadrant::BottomRight,
                                quadrant_button_size,
                            );
                        });
                    });
            });
        }
    }
}

/// Helper function to format coordinates
fn format_coordinate(value: f32) -> String {
    // Check if the value is effectively a whole number
    if value.fract().abs() < 0.001 {
        // Display as integer
        format!("{}", value.round() as i32)
    } else {
        // Display with one decimal place
        format!("{:.1}", value)
    }
}

/// System to handle quadrant selection from UI
fn handle_quadrant_selection(
    interaction_query: Query<
        (&Interaction, &QuadrantButton),
        Changed<Interaction>,
    >,
    mut coord_selection: ResMut<CoordinateSelection>,
    mut quadrant_buttons: Query<(&mut BackgroundColor, &QuadrantButton)>,
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
        for (mut background, quadrant_button) in quadrant_buttons.iter_mut() {
            // Set the color based on whether this is the selected quadrant
            if quadrant_button.0 == coord_selection.quadrant {
                // Selected - use bright color
                *background = BackgroundColor(Color::srgba(1.0, 0.6, 0.1, 0.9));
            } else {
                // Not selected - use darker color
                *background = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7));
            }
        }
    }
}

/// Spawns the coordinate pane in the lower right corner
fn spawn_coord_pane(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Define panel colors (matching glyph_pane style)
    let panel_background_color = Color::srgba(0.1, 0.1, 0.1, 0.9);
    let border_color = Color::srgba(1.0, 1.0, 1.0, 0.3);
    let border_radius = 4.0;

    info!("Spawning coordinate pane");

    // Just spawn the container - the contents will be filled by the rebuild system
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(24.0),
                right: Val::Px(24.0),
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect::all(Val::Px(0.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(panel_background_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(border_radius)),
            CoordPane,
            // Make the pane visible by default
            Visibility::Visible,
            Name::new("CoordinatePane"),
        ));
}

/// Helper to spawn a quadrant selection button
fn spawn_quadrant_button(
    parent: &mut ChildBuilder,
    quadrant: Quadrant,
    size: f32,
) {
    // Determine if this is the default selected quadrant (Center)
    let is_selected = quadrant == Quadrant::Center;
    let background_color = if is_selected {
        BackgroundColor(Color::srgba(1.0, 0.6, 0.1, 0.9)) // Bright for selected
    } else {
        BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.7)) // Dark for unselected
    };

    parent.spawn((
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        background_color,
        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        BorderRadius::all(Val::Px(2.0)),
        Interaction::default(),
        QuadrantButton(quadrant),
    ));
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
        info!("CoordPane: SelectionState is available with {} selected entities", state.selected.len());
        
        // Log entities in the selection for debugging
        if !state.selected.is_empty() {
            let entities: Vec<_> = state.selected.iter().collect();
            info!("CoordPane: Selected entities in SelectionState: {:?}", entities);
        }
    } else {
        info!("CoordPane: SelectionState resource is NOT available");
    }
    
    // Default to zero if SelectionState is not available
    let selected_count = selection_state.as_ref().map_or(0, |state| state.selected.len());
    
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
                    info!("CoordPane: Found position for entity {entity:?}: {:?}", pos);
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
            
            let frame = Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
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
        info!("Updating coordinate pane UI: count=0, quadrant={:?}, frame={:?}", 
              coord_selection.quadrant, coord_selection.frame);
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
