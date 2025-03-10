//! The floating panel that displays coordinates of selected points.

use crate::quadrant::Quadrant;
use bevy::prelude::*;

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
            .add_systems(Startup, spawn_coord_pane)
            // Add debugging system to update the UI when selection changes
            .add_systems(Update, debug_selection_changes)
            // Handle quadrant selection
            .add_systems(Update, handle_quadrant_selection);
    }
}

/// Debug system to log changes to selection
fn debug_selection_changes(
    coord_selection: Res<CoordinateSelection>,
    mut coord_pane_query: Query<&mut Text, With<CoordText>>,
) {
    if coord_selection.is_changed() {
        info!(
            "Selection changed: count={}, bounds={:?}, quadrant={:?}",
            coord_selection.count,
            coord_selection.frame,
            coord_selection.quadrant
        );

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
                        (frame.min.y + frame.max.y) / 2.0,
                    ),
                    Quadrant::TopLeft => Vec2::new(frame.min.x, frame.max.y),
                    Quadrant::Top => Vec2::new(
                        (frame.min.x + frame.max.x) / 2.0,
                        frame.max.y,
                    ),
                    Quadrant::TopRight => Vec2::new(frame.max.x, frame.max.y),
                    Quadrant::Right => Vec2::new(
                        frame.max.x,
                        (frame.min.y + frame.max.y) / 2.0,
                    ),
                    Quadrant::BottomRight => {
                        Vec2::new(frame.max.x, frame.min.y)
                    }
                    Quadrant::Bottom => Vec2::new(
                        (frame.min.x + frame.max.x) / 2.0,
                        frame.min.y,
                    ),
                    Quadrant::BottomLeft => Vec2::new(frame.min.x, frame.min.y),
                    Quadrant::Left => Vec2::new(
                        frame.min.x,
                        (frame.min.y + frame.max.y) / 2.0,
                    ),
                };

                let display_text = format!(
                    "Selection: {} points\nX: {:.1}, Y: {:.1}\nW: {:.1}, H: {:.1}",
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
    mut interaction_query: Query<
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
fn spawn_coord_pane(mut commands: Commands) {
    // Define panel colors (matching glyph_pane style)
    let panel_background_color = Color::srgba(0.1, 0.1, 0.1, 0.9);
    let text_color = Color::WHITE;
    let border_color = Color::srgba(1.0, 1.0, 1.0, 0.3);
    let border_radius = 4.0;
    let quadrant_button_size = 20.0;
    let quadrant_spacing = 2.0;

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

/// Public function to update the coordinate selection
/// Called from outside systems when points are selected
pub fn update_selection(
    count: usize,
    frame: Rect,
    mut coord_selection: ResMut<CoordinateSelection>,
) {
    coord_selection.count = count;
    coord_selection.frame = frame;

    // Keep the same quadrant unless this is a new selection
    if coord_selection.count == 0 || coord_selection.count == 1 {
        coord_selection.quadrant = Quadrant::Center;
    }
}
