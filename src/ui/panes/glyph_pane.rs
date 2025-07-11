//! UI pane to display information about the current glyph
//!
//! Shows glyph name, Unicode codepoint, advance width, side bearings,
//! and side bearings in the lower left corner of the window.

use crate::core::state::AppState;
use crate::ui::theme::*;
use bevy::prelude::*;

/// Resource to store current glyph metrics for display
#[derive(Resource, Default)]
pub struct CurrentGlyphMetrics {
    pub glyph_name: String,
    pub unicode: String,
    pub advance: String,
    pub left_bearing: String,
    pub right_bearing: String,
    pub left_group: String,
    pub right_group: String,
}

/// Component marker for the glyph pane
#[derive(Component, Default)]
pub struct GlyphPane;

/// Component marker for the glyph name text
#[derive(Component)]
pub struct GlyphNameText;

/// Component marker for the glyph unicode text
#[derive(Component)]
pub struct GlyphUnicodeText;

/// Component marker for the glyph advance text
#[derive(Component)]
pub struct GlyphAdvanceText;

/// Component marker for the glyph left bearing text
#[derive(Component)]
pub struct GlyphLeftBearingText;

/// Component marker for the glyph right bearing text
#[derive(Component)]
pub struct GlyphRightBearingText;

/// Component marker for the glyph left kerning group text
#[derive(Component)]
pub struct GlyphLeftGroupText;

/// Component marker for the glyph right kerning group text
#[derive(Component)]
pub struct GlyphRightGroupText;

/// Plugin that adds the glyph pane functionality
pub struct GlyphPanePlugin;

impl Plugin for GlyphPanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGlyphMetrics>()
            .add_systems(Update, (update_glyph_pane, update_glyph_metrics));
    }
}

/// System to update the glyph pane display
///
/// Currently displays values from the CurrentGlyphMetrics resource.
fn update_glyph_pane(world: &mut World) {
    // Get the current metrics resource
    let metrics = world.resource::<CurrentGlyphMetrics>();

    // Format the values for display
    let glyph_name = if metrics.glyph_name.is_empty() {
        "None".to_string()
    } else {
        metrics.glyph_name.clone()
    };

    let unicode = if metrics.unicode.is_empty() {
        "None".to_string()
    } else {
        metrics.unicode.to_uppercase()
    };

    let advance = if metrics.advance.is_empty() {
        "--".to_string()
    } else {
        metrics.advance.clone()
    };

    let lsb = if metrics.left_bearing.is_empty() {
        "--".to_string()
    } else {
        metrics.left_bearing.clone()
    };

    let rsb = if metrics.right_bearing.is_empty() {
        "--".to_string()
    } else {
        metrics.right_bearing.clone()
    };

    let left_group = if metrics.left_group.is_empty() {
        "None".to_string()
    } else {
        metrics.left_group.clone()
    };

    let right_group = if metrics.right_group.is_empty() {
        "None".to_string()
    } else {
        metrics.right_group.clone()
    };

    // Update the texts in the UI
    let mut name_query =
        world.query_filtered::<&mut Text, With<GlyphNameText>>();
    for mut text in name_query.iter_mut(world) {
        *text = Text::new(glyph_name.clone());
    }

    let mut unicode_query =
        world.query_filtered::<&mut Text, With<GlyphUnicodeText>>();
    for mut text in unicode_query.iter_mut(world) {
        *text = Text::new(unicode.clone());
    }

    let mut advance_query =
        world.query_filtered::<&mut Text, With<GlyphAdvanceText>>();
    for mut text in advance_query.iter_mut(world) {
        *text = Text::new(advance.clone());
    }

    let mut lsb_query =
        world.query_filtered::<&mut Text, With<GlyphLeftBearingText>>();
    for mut text in lsb_query.iter_mut(world) {
        *text = Text::new(lsb.clone());
    }

    let mut rsb_query =
        world.query_filtered::<&mut Text, With<GlyphRightBearingText>>();
    for mut text in rsb_query.iter_mut(world) {
        *text = Text::new(rsb.clone());
    }

    let mut left_group_query =
        world.query_filtered::<&mut Text, With<GlyphLeftGroupText>>();
    for mut text in left_group_query.iter_mut(world) {
        *text = Text::new(left_group.clone());
    }

    let mut right_group_query =
        world.query_filtered::<&mut Text, With<GlyphRightGroupText>>();
    for mut text in right_group_query.iter_mut(world) {
        *text = Text::new(right_group.clone());
    }
}

/// Spawns the glyph pane in the lower left corner
pub fn spawn_glyph_pane(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    // Create the position properties for the glyph pane (bottom left)
    let position_props = UiRect {
        left: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto, // Explicitly set top to Auto to prevent stretching
        right: Val::Auto, // Explicitly set right to Auto for correct sizing
        ..default()
    };

    commands
        .spawn(create_widget_style(
            asset_server,
            PositionType::Absolute,
            position_props,
            GlyphPane,
            "GlyphPane",
        ))
        .with_children(|parent| {
            // Glyph name row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Glyph:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphNameText,
                    ));
                });

            // Unicode value row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Unicode:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphUnicodeText,
                    ));
                });

            // Advance width row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Advance:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphAdvanceText,
                    ));
                });

            // Left side bearing row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("LSB:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphLeftBearingText,
                    ));
                });

            // Right side bearing row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("RSB:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphRightBearingText,
                    ));
                });

            // Left kerning group row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Left Group:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphLeftGroupText,
                    ));
                });

            // Right kerning group row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Right Group:"),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphRightGroupText,
                    ));
                });
        });
}

/// Updates the glyph metrics for the current glyph
pub fn update_glyph_metrics(
    app_state: Res<AppState>,
    text_editor_state: Res<crate::core::state::text_editor::TextEditorState>,
    mut metrics: ResMut<CurrentGlyphMetrics>,
) {
    // Get information from the active sort instead of glyph navigation
    if let Some((_buffer_index, sort_entry)) = text_editor_state.get_active_sort() {
        let glyph_name = sort_entry.kind.glyph_name().to_string();
        
        // Found an active sort, get its details
        metrics.glyph_name = glyph_name.clone();

        // Set the Unicode information by finding the codepoint for this glyph
        if let Some(glyph_data) = app_state.workspace.font.get_glyph(&glyph_name) {
            // Find the first Unicode codepoint for this glyph
            if let Some(first_codepoint) = glyph_data.unicode_values.first() {
                metrics.unicode = format!("{:04X}", *first_codepoint as u32);
            } else {
                metrics.unicode = String::new();
            }

            // Get advance width
            metrics.advance = format!("{}", glyph_data.advance_width as i32);

            // Calculate sidebearings based on glyph outline bounds
            let outline_bounds = if let Some(outline) = &glyph_data.outline {
                if !outline.contours.is_empty() {
                    // Calculate bounds from outline contours
                    let mut min_x = f64::MAX;
                    let mut max_x = f64::MIN;

                    // Iterate through all contours and their points
                    let mut has_points = false;
                    for contour in &outline.contours {
                        for point in &contour.points {
                            has_points = true;
                            if point.x < min_x {
                                min_x = point.x;
                            }
                            if point.x > max_x {
                                max_x = point.x;
                            }
                        }
                    }

                    // If we found valid bounds
                    if has_points && min_x != f64::MAX && max_x != f64::MIN {
                        Some((min_x, max_x))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Calculate LSB and RSB if we have both outline bounds and advance width
            if let Some((min_x, max_x)) = outline_bounds {
                // Left side bearing is the distance from origin to the leftmost point
                let lsb = min_x;

                // Right side bearing is the distance from the rightmost point to the advance width
                let rsb = glyph_data.advance_width as f64 - max_x;

                metrics.left_bearing = format!("{}", lsb as i32);
                metrics.right_bearing = format!("{}", rsb as i32);
            } else {
                // If we couldn't calculate bounds, use placeholder values
                metrics.left_bearing = "0".to_string();
                metrics.right_bearing = "0".to_string();
            }

            // TODO: Get kerning groups when groups are implemented in current architecture
            metrics.left_group = String::new();
            metrics.right_group = String::new();
        } else {
            // No glyph data found
            metrics.advance = "-".to_string();
            metrics.left_bearing = "-".to_string();
            metrics.right_bearing = "-".to_string();
            metrics.left_group = String::new();
            metrics.right_group = String::new();
        }
    } else {
        // No active sort found, clear the metrics
        metrics.glyph_name = String::new();
        metrics.unicode = String::new();
        metrics.advance = "-".to_string();
        metrics.left_bearing = "-".to_string();
        metrics.right_bearing = "-".to_string();
        metrics.left_group = String::new();
        metrics.right_group = String::new();
    }
} 