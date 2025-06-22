//! Coordinate Pane Module
//!
//! This module implements a floating panel that displays coordinates and dimensions of selected elements.

use crate::editing::selection::SelectionState;
use crate::geometry::quadrant::Quadrant;
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::ui::UiRect;

// ===============================================================================
// COMPONENTS & RESOURCES
// ===============================================================================

/// Resource that tracks the current state of coordinate selection and display
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CoordinateSelection {
    /// Number of elements currently selected
    pub count: usize,
    /// Currently active quadrant that determines which reference point to use
    pub quadrant: Quadrant,
    /// Bounding rectangle that encompasses all selected elements
    pub frame: Rect,
}

/// Component marker for the coordinate pane
#[derive(Component, Default)]
pub struct CoordPane;

/// Component marker for coordinate value text elements
#[derive(Component, Default)]
pub struct XValue;

#[derive(Component, Default)]
pub struct YValue;

#[derive(Component, Default)]
pub struct WidthValue;

#[derive(Component, Default)]
pub struct HeightValue;

/// Component for quadrant buttons
#[derive(Component)]
pub struct QuadrantButton(pub Quadrant);

impl Default for QuadrantButton {
    fn default() -> Self {
        Self(Quadrant::Center)
    }
}

// ===============================================================================
// PLUGIN IMPLEMENTATION
// ===============================================================================

/// Plugin that adds coordinate pane functionality to the application
pub struct CoordinatePanePlugin;

impl Plugin for CoordinatePanePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<CoordinateSelection>()
            .register_type::<Quadrant>()
            .init_resource::<CoordinateSelection>()
            .add_systems(Startup, spawn_coord_pane)
            .add_systems(Update, (
                update_coordinate_selection,
                update_coordinate_display,
                handle_quadrant_buttons,
            ));
    }
}

/// Spawns the coordinate pane in the bottom-right corner
pub fn spawn_coord_pane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Create the position properties for the coordinate pane (bottom right)
    let position_props = UiRect {
        right: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto,
        left: Val::Auto,
        ..default()
    };

    commands
        .spawn(create_widget_style(
            &asset_server,
            PositionType::Absolute,
            position_props,
            CoordPane,
            "CoordPane",
        ))
        .with_children(|parent| {
            // X coordinate row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(8.0)),
                            ..default()
                        },
                        Text::new("X:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                    
                    // Value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        XValue,
                    ));
                });

            // Y coordinate row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(8.0)),
                            ..default()
                        },
                        Text::new("Y:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                    
                    // Value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        YValue,
                    ));
                });

            // Width row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(8.0)),
                            ..default()
                        },
                        Text::new("W:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                    
                    // Value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        WidthValue,
                    ));
                });

            // Height row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(8.0)),
                            ..default()
                        },
                        Text::new("H:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                    
                    // Value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        HeightValue,
                    ));
                });

            // Quadrant selector (3x3 grid of buttons)
            parent
                .spawn((Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        RepeatedGridTrack::fr(1, 1.0),
                        RepeatedGridTrack::fr(1, 1.0),
                        RepeatedGridTrack::fr(1, 1.0),
                    ],
                    grid_template_rows: vec![
                        RepeatedGridTrack::fr(1, 1.0),
                        RepeatedGridTrack::fr(1, 1.0),
                        RepeatedGridTrack::fr(1, 1.0),
                    ],
                    width: Val::Px(96.0),
                    height: Val::Px(96.0),
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                },))
                .with_children(|grid| {
                    // Create 3x3 grid of quadrant buttons
                    let quadrants = [
                        [Quadrant::TopLeft, Quadrant::Top, Quadrant::TopRight],
                        [Quadrant::Left, Quadrant::Center, Quadrant::Right],
                        [Quadrant::BottomLeft, Quadrant::Bottom, Quadrant::BottomRight],
                    ];

                    for row in quadrants.iter() {
                        for &quadrant in row.iter() {
                            let is_selected = quadrant == Quadrant::Center;
                            grid.spawn((
                                Button,
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_selected {
                                    PRESSED_BUTTON
                                } else {
                                    NORMAL_BUTTON
                                }),
                                BorderColor(if is_selected {
                                    PRESSED_BUTTON_OUTLINE_COLOR
                                } else {
                                    NORMAL_BUTTON_OUTLINE_COLOR
                                }),
                                BorderRadius::all(Val::Px(12.0)),
                                QuadrantButton(quadrant),
                            ));
                        }
                    }
                });
        });
}

/// System to update coordinate selection based on current selection state
fn update_coordinate_selection(
    mut coord_selection: ResMut<CoordinateSelection>,
    // TODO: Connect with actual selection system when available
) {
    // Placeholder implementation - will be connected to actual selection system
    coord_selection.count = 0;
    coord_selection.frame = Rect::default();
}

/// System to update the coordinate display text
fn update_coordinate_display(
    coord_selection: Res<CoordinateSelection>,
    mut x_query: Query<&mut Text, (With<XValue>, Without<YValue>, Without<WidthValue>, Without<HeightValue>)>,
    mut y_query: Query<&mut Text, (With<YValue>, Without<XValue>, Without<WidthValue>, Without<HeightValue>)>,
    mut w_query: Query<&mut Text, (With<WidthValue>, Without<XValue>, Without<YValue>, Without<HeightValue>)>,
    mut h_query: Query<&mut Text, (With<HeightValue>, Without<XValue>, Without<YValue>, Without<WidthValue>)>,
) {
    if coord_selection.count > 0 {
        let reference_point = get_quadrant_point(&coord_selection.frame, coord_selection.quadrant);
        
        // Update coordinate values
        if let Ok(mut text) = x_query.get_single_mut() {
            *text = Text::new(format!("{}", reference_point.x as i32));
        }
        if let Ok(mut text) = y_query.get_single_mut() {
            *text = Text::new(format!("{}", reference_point.y as i32));
        }
        if let Ok(mut text) = w_query.get_single_mut() {
            *text = Text::new(format!("{}", coord_selection.frame.width() as i32));
        }
        if let Ok(mut text) = h_query.get_single_mut() {
            *text = Text::new(format!("{}", coord_selection.frame.height() as i32));
        }
    } else {
        // Show dashes when nothing is selected
        if let Ok(mut text) = x_query.get_single_mut() {
            *text = Text::new("--");
        }
        if let Ok(mut text) = y_query.get_single_mut() {
            *text = Text::new("--");
        }
        if let Ok(mut text) = w_query.get_single_mut() {
            *text = Text::new("--");
        }
        if let Ok(mut text) = h_query.get_single_mut() {
            *text = Text::new("--");
        }
    }
}

/// System to handle quadrant button clicks
fn handle_quadrant_buttons(
    mut interaction_query: Query<
        (&Interaction, &QuadrantButton),
        Changed<Interaction>,
    >,
    mut coord_selection: ResMut<CoordinateSelection>,
    mut all_buttons: Query<(&QuadrantButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (interaction, button) in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                // Update the selected quadrant
                coord_selection.quadrant = button.0;
                
                // Update all button appearances
                for (other_button, mut other_bg, mut other_border) in all_buttons.iter_mut() {
                    if other_button.0 == button.0 {
                        // This is the selected button
                        *other_bg = BackgroundColor(PRESSED_BUTTON);
                        *other_border = BorderColor(PRESSED_BUTTON_OUTLINE_COLOR);
                    } else {
                        // This is an unselected button
                        *other_bg = BackgroundColor(NORMAL_BUTTON);
                        *other_border = BorderColor(NORMAL_BUTTON_OUTLINE_COLOR);
                    }
                }
            }
            Interaction::Hovered => {
                if coord_selection.quadrant != button.0 {
                    // Update this specific button's appearance
                    for (other_button, mut other_bg, mut other_border) in all_buttons.iter_mut() {
                        if other_button.0 == button.0 {
                            *other_bg = BackgroundColor(HOVERED_BUTTON);
                            *other_border = BorderColor(HOVERED_BUTTON_OUTLINE_COLOR);
                            break;
                        }
                    }
                }
            }
            Interaction::None => {
                if coord_selection.quadrant != button.0 {
                    // Update this specific button's appearance
                    for (other_button, mut other_bg, mut other_border) in all_buttons.iter_mut() {
                        if other_button.0 == button.0 {
                            *other_bg = BackgroundColor(NORMAL_BUTTON);
                            *other_border = BorderColor(NORMAL_BUTTON_OUTLINE_COLOR);
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Gets the reference point for a quadrant
fn get_quadrant_point(frame: &Rect, quadrant: Quadrant) -> Vec2 {
    match quadrant {
        Quadrant::TopLeft => Vec2::new(frame.min.x, frame.max.y),
        Quadrant::Top => Vec2::new(frame.center().x, frame.max.y),
        Quadrant::TopRight => Vec2::new(frame.max.x, frame.max.y),
        Quadrant::Left => Vec2::new(frame.min.x, frame.center().y),
        Quadrant::Center => frame.center(),
        Quadrant::Right => Vec2::new(frame.max.x, frame.center().y),
        Quadrant::BottomLeft => Vec2::new(frame.min.x, frame.min.y),
        Quadrant::Bottom => Vec2::new(frame.center().x, frame.min.y),
        Quadrant::BottomRight => Vec2::new(frame.max.x, frame.min.y),
    }
}
