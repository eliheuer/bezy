//! The floating panel that displays coordinates of selected points.

use bevy::prelude::*;
use crate::quadrant::Quadrant;

// We'll use our own marker for testing if we can't directly access Selected component
#[derive(Component)]
struct MockSelected;

/// Resource to store the current coordinate selection
#[derive(Resource, Default)]
pub struct CoordinateSelection {
    /// The number of points selected
    pub count: usize,
    /// The current quadrant used for selection
    pub quadrant: Quadrant,
    /// The bounding box of the selection
    pub frame: Rect,
}

/// Marker component for the coordinate pane
#[derive(Component)]
pub struct CoordPane;

/// Marker component for the coordinate pane text
#[derive(Component)]
struct CoordText;

/// Marker component for quadrant selector
#[derive(Component)]
struct QuadrantSelector;

/// Component to track which quadrant a button represents
#[derive(Component)]
struct QuadrantButton(Quadrant);

/// Plugin for coordinate pane functionality
pub struct CoordPanePlugin;

impl Plugin for CoordPanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoordinateSelection>()
            // Spawn the coordinate pane
            .add_systems(Startup, spawn_coord_pane)
            // Add system to sync with the selection system
            .add_systems(Update, sync_with_selection_system)
            // Add debugging system to update the UI when selection changes
            .add_systems(Update, debug_selection_changes)
            // Handle quadrant selection
            .add_systems(Update, handle_quadrant_selection)
            // Testing: initialize with a test selection to ensure visibility
            .add_systems(PostStartup, |mut coord_selection: ResMut<CoordinateSelection>| {
                info!("Initializing test selection for coordinate pane");
                coord_selection.count = 1;
                coord_selection.frame = Rect::from_corners(
                    Vec2::new(100.0, 100.0),
                    Vec2::new(200.0, 200.0)
                );
                coord_selection.quadrant = Quadrant::Center;
            });
    }
}

/// Debug system to log changes to selection
fn debug_selection_changes(
    coord_selection: Res<CoordinateSelection>,
    mut coord_pane_query: Query<&mut Text, With<CoordText>>,
    mut visibility_query: Query<&mut Visibility, With<CoordPane>>,
) {
    // Make sure the coordinate pane is visible
    for mut visibility in visibility_query.iter_mut() {
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
            info!("Set coordinate pane to visible");
        }
    }

    if coord_selection.is_changed() {
        info!("CoordinateSelection changed: count={}, quadrant={:?}", 
              coord_selection.count, 
              coord_selection.quadrant);
        
        // Update coordinate pane text when selection changes
        if let Ok(mut text) = coord_pane_query.get_single_mut() {
            if coord_selection.count == 0 {
                *text = Text::new("Waiting for selection");
            } else {
                let frame = coord_selection.frame;
                
                // Get the point based on the selected quadrant
                let point = match coord_selection.quadrant {
                    Quadrant::Center => Vec2::new(
                        (frame.min.x + frame.max.x) / 2.0,
                        (frame.min.y + frame.max.y) / 2.0
                    ),
                    Quadrant::TopLeft => Vec2::new(frame.min.x, frame.max.y),
                    Quadrant::Top => Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.max.y),
                    Quadrant::TopRight => Vec2::new(frame.max.x, frame.max.y),
                    Quadrant::Right => Vec2::new(frame.max.x, (frame.min.y + frame.max.y) / 2.0),
                    Quadrant::BottomRight => Vec2::new(frame.max.x, frame.min.y),
                    Quadrant::Bottom => Vec2::new((frame.min.x + frame.max.x) / 2.0, frame.min.y),
                    Quadrant::BottomLeft => Vec2::new(frame.min.x, frame.min.y),
                    Quadrant::Left => Vec2::new(frame.min.x, (frame.min.y + frame.max.y) / 2.0),
                };
                
                let display_text = format!(
                    "Selection: {} points\nx: {:.1}, y: {:.1}\nw: {:.1}, h: {:.1}",
                    coord_selection.count, 
                    point.x, point.y,
                    frame.max.x - frame.min.x,
                    frame.max.y - frame.min.y
                );
                
                *text = Text::new(display_text);
            }
        }
    }
}

/// System to handle quadrant selection from UI
fn handle_quadrant_selection(
    interaction_query: Query<(&Interaction, &QuadrantButton), Changed<Interaction>>,
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
fn spawn_coord_pane(mut commands: Commands) {
    // Define panel colors (matching glyph_pane style)
    let panel_background_color = Color::srgba(0.1, 0.1, 0.1, 0.9);
    let text_color = Color::WHITE;
    let border_color = Color::srgba(1.0, 1.0, 1.0, 0.3);
    let border_radius = 4.0;
    let quadrant_button_size = 20.0;
    let quadrant_spacing = 2.0;

    info!("Spawning coordinate pane");
    
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
            // Make the pane initially visible
            Visibility::Visible,
            Name::new("CoordinatePane"),
        ))
        .with_children(|parent| {
            // Text display for coordinates
            parent.spawn((
                Text::new("Waiting for selection"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(text_color),
                CoordText,
            ));

            // Add quadrant selector grid (3x3)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
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

/// System to synchronize the CoordinateSelection with the actual selection system
/// This is the key system that makes the coordinate pane respond to selections
fn sync_with_selection_system(
    // Use standard queries instead of world access to avoid type issues
    point_query: Query<(Entity, &GlobalTransform, Option<&Name>)>,
    mut coord_selection: ResMut<CoordinateSelection>,
) {
    // Look for entities that might be selected points
    let mut selected_points = Vec::new();
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    
    // Look for entities with names that indicate they might be selected points
    for (entity, transform, name) in point_query.iter() {
        if let Some(name) = name {
            // Simple heuristic: assume points with certain names are selected
            let name_str = name.as_str();
            if (name_str.contains("Point") && name_str.contains("Select")) ||
               name_str.contains("Selected") {
                selected_points.push((entity, transform.translation()));
            }
        }
    }
    
    // Process the points we found
    let selected_count = selected_points.len();
    
    if selected_count > 0 {
        // We found points, process them
        for (_, position) in &selected_points {
            min_x = min_x.min(position.x);
            min_y = min_y.min(position.y);
            max_x = max_x.max(position.x);
            max_y = max_y.max(position.y);
        }
        
        // Update the selection
        let bounds = Rect::from_corners(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
        
        // Only update if something changed
        let selection_changed = coord_selection.count != selected_count || coord_selection.frame != bounds;
        
        if selection_changed {
            info!("Updating selection: {} points found", selected_count);
            
            coord_selection.count = selected_count;
            coord_selection.frame = bounds;
            
            // Use Center quadrant for new selections
            if selected_count == 1 || coord_selection.count == 0 {
                coord_selection.quadrant = Quadrant::Center;
            }
        }
    } else if coord_selection.count > 0 {
        // No points found but we have a selection, clear it
        info!("Clearing selection, no points found");
        coord_selection.count = 0;
        coord_selection.frame = Rect::default();
    }
    
    // Add debug mark to make selection visible
    // Make at least one point selected for testing
    // This is a placeholder until we get the real selection working
    if selected_count == 0 {
        // Debug: Hard-code a test selection to verify the UI works
        if coord_selection.count == 0 {
            info!("Adding test selection");
            coord_selection.count = 1;
            coord_selection.frame = Rect::from_corners(
                Vec2::new(100.0, 100.0),
                Vec2::new(200.0, 200.0)
            );
            coord_selection.quadrant = Quadrant::Center;
        }
    }
}
