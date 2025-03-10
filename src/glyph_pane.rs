//! The panel that displays glyph metrics information
//!
//! This component displays the current glyph, its name, unicode, advance width,
//! and side bearings in the lower left corner of the window.

use crate::cli;
use crate::data;
use crate::theme::*;
use bevy::prelude::*;

/// Resource to store current glyph metrics for display
#[derive(Resource, Default)]
pub struct CurrentGlyphMetrics {
    pub glyph_name: String,
    pub unicode: String,
    pub advance: String,
    pub left_bearing: String,
    pub right_bearing: String,
}

/// Component marker for the glyph pane
#[derive(Component)]
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

/// Component marker for the glyph outline preview
#[derive(Component)]
pub struct GlyphOutlinePreview;

/// Component marker for glyph outline lines in the preview
#[derive(Component)]
struct GlyphOutlineLine;

/// Plugin that adds the glyph pane functionality
pub struct GlyphPanePlugin;

impl Plugin for GlyphPanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGlyphMetrics>().add_systems(
            Update,
            (
                update_glyph_pane,
                update_glyph_metrics,
                update_glyph_outline_preview,
            ),
        );
    }
}

/// System to update the glyph pane display
///
/// Currently displays values from the CurrentGlyphMetrics resource.
///
/// TO DO:
/// - Create a system in draw.rs to update the CurrentGlyphMetrics resource
/// - The system should extract data from the current glyph
fn update_glyph_pane(world: &mut World) {
    // Get the current metrics resource
    let metrics = world.resource::<CurrentGlyphMetrics>();

    // Log the current metrics for debugging
    bevy::log::info!("GlyphPane: Current metrics - Name: '{}', Unicode: '{}', Advance: '{}', LSB: '{}', RSB: '{}'", 
        metrics.glyph_name, metrics.unicode, metrics.advance, metrics.left_bearing, metrics.right_bearing);

    // Format the values for display
    let glyph_name = if metrics.glyph_name.is_empty() {
        "Glyph: (No glyph selected)".to_string()
    } else {
        format!("Glyph: {}", metrics.glyph_name)
    };

    let unicode = if metrics.unicode.is_empty() {
        "Unicode: None".to_string()
    } else {
        format!("Unicode: {}", metrics.unicode.to_uppercase())
    };

    let advance = if metrics.advance.is_empty() {
        "Advance: --".to_string()
    } else {
        format!("Advance: {}", metrics.advance)
    };

    let lsb = if metrics.left_bearing.is_empty() {
        "LSB: --".to_string()
    } else {
        format!("LSB: {}", metrics.left_bearing)
    };

    let rsb = if metrics.right_bearing.is_empty() {
        "RSB: --".to_string()
    } else {
        format!("RSB: {}", metrics.right_bearing)
    };

    // Log the formatted strings for debugging
    bevy::log::info!(
        "GlyphPane: Formatted display - {}, {}, {}, {}, {}",
        glyph_name,
        unicode,
        advance,
        lsb,
        rsb
    );

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
}

/// Spawns the glyph pane in the lower left corner
pub fn spawn_glyph_pane(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let font = asset_server.load(DEFAULT_FONT_PATH);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(32.0),
                left: Val::Px(32.0),
                padding: UiRect::all(Val::Px(16.0)),
                margin: UiRect::all(Val::Px(0.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND_COLOR),
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
            BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
            GlyphPane,
        ))
        .with_children(|parent| {
            // Glyph outline preview
            parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    margin: UiRect {
                        bottom: Val::Px(16.0),
                        ..default()
                    },
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                GlyphOutlinePreview,
            ));

            // Glyph name
            parent.spawn((
                Text::new("Glyph: Loading..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                GlyphNameText,
            ));

            // Unicode value
            parent.spawn((
                Text::new("Unicode: Loading..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                GlyphUnicodeText,
            ));

            // Advance width
            parent.spawn((
                Text::new("Advance: Loading..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                GlyphAdvanceText,
            ));

            // Left side bearing
            parent.spawn((
                Text::new("LSB: Loading..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                GlyphLeftBearingText,
            ));

            // Right side bearing
            parent.spawn((
                Text::new("RSB: Loading..."),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                GlyphRightBearingText,
            ));
        });
}

/// Draws the current glyph outline in the glyph pane preview
fn update_glyph_outline_preview(
    app_state: Res<data::AppState>,
    cli_args: Res<cli::CliArgs>,
    mut commands: Commands,
    preview_query: Query<(Entity, &Node), With<GlyphOutlinePreview>>,
    outline_lines_query: Query<Entity, With<GlyphOutlineLine>>,
) {
    // Skip if no outline preview entity found
    if preview_query.is_empty() {
        return;
    }

    // Clean up existing outline lines
    for entity in outline_lines_query.iter() {
        commands.entity(entity).despawn();
    }

    // Get the glyph information
    if app_state.workspace.font.ufo.font_info.is_none() {
        return;
    }

    if let Some(glyph_name) = cli_args.find_glyph(&app_state.workspace.font.ufo)
    {
        if let Some(default_layer) =
            app_state.workspace.font.ufo.get_default_layer()
        {
            if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                if let Some(outline) = &glyph.outline {
                    // Get the preview container for sizing
                    let (preview_entity, node) = preview_query.single();

                    // Calculate the bounding box of the glyph
                    let mut min_x = f32::MAX;
                    let mut min_y = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut max_y = f32::MIN;
                    let mut has_points = false;

                    // Find glyph bounds
                    for contour in &outline.contours {
                        for point in &contour.points {
                            has_points = true;
                            min_x = min_x.min(point.x);
                            max_x = max_x.max(point.x);
                            min_y = min_y.min(point.y);
                            max_y = max_y.max(point.y);
                        }
                    }

                    if !has_points {
                        return;
                    }

                    // Get advance width if available
                    let advance_width = glyph
                        .advance
                        .as_ref()
                        .map(|a| a.width as f32)
                        .unwrap_or(0.0);

                    // Extend bounds to include advance width
                    max_x = max_x.max(advance_width);

                    // Calculate scaling to fit the preview area with padding
                    let padding = 20.0; // Padding in pixels

                    // Get the dimensions of the preview area
                    // For simplicity, use fixed sizes since we know the container is 200x200
                    let preview_width = 200.0 - padding * 2.0;
                    let preview_height = 200.0 - padding * 2.0;

                    let glyph_width = max_x - min_x;
                    let glyph_height = max_y - min_y;

                    // Avoid division by zero
                    if glyph_width <= 0.0 || glyph_height <= 0.0 {
                        return;
                    }

                    let scale_x = preview_width / glyph_width;
                    let scale_y = preview_height / glyph_height;
                    let scale = scale_x.min(scale_y);

                    // Offset to center the glyph in the preview
                    let offset_x =
                        padding + (preview_width - glyph_width * scale) / 2.0;
                    let offset_y =
                        padding + (preview_height - glyph_height * scale) / 2.0;

                    // Spawn a child entity to represent the glyph outlines
                    commands.entity(preview_entity).with_children(|parent| {
                        // Draw each contour
                        for contour in &outline.contours {
                            draw_contour_lines(
                                parent, contour, min_x, min_y, scale, offset_x,
                                offset_y,
                            );
                        }
                    });
                }
            }
        }
    }
}

/// Helper function to draw a contour as lines in the preview
fn draw_contour_lines(
    parent: &mut ChildBuilder,
    contour: &norad::Contour,
    min_x: f32,
    min_y: f32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // We'll need to draw lines between on-curve points
    let mut on_curve_points = Vec::new();

    // Collect all on-curve points
    for (i, point) in points.iter().enumerate() {
        if is_on_curve(point) {
            on_curve_points.push((i, point));
        }
    }

    // If we have fewer than 2 on-curve points, we can't draw any lines
    if on_curve_points.len() < 2 {
        return;
    }

    // Draw lines between on-curve points using multiple small line segments
    // This is a workaround since Bevy UI doesn't support rotations directly
    for i in 0..on_curve_points.len() {
        let (_, start_point) = on_curve_points[i];
        let (_, end_point) = on_curve_points[(i + 1) % on_curve_points.len()];

        // Transform points to preview space
        let start_x = (start_point.x - min_x) * scale + offset_x;
        let start_y = (start_point.y - min_y) * scale + offset_y;
        let end_x = (end_point.x - min_x) * scale + offset_x;
        let end_y = (end_point.y - min_y) * scale + offset_y;

        // Use Bresenham's line algorithm to draw line segments
        draw_line(parent, start_x, start_y, end_x, end_y);
    }
}

/// Helper function to draw a straight line between two points using multiple small segments
fn draw_line(parent: &mut ChildBuilder, x1: f32, y1: f32, x2: f32, y2: f32) {
    // Line color and thickness
    let line_color = Color::WHITE;
    let thickness = 1.5;

    // Calculate line length
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length = (dx * dx + dy * dy).sqrt();

    // If very short, just draw a single segment
    if length < 2.0 {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x1),
                top: Val::Px(y1),
                width: Val::Px(thickness),
                height: Val::Px(thickness),
                ..default()
            },
            BackgroundColor(line_color),
            GlyphOutlineLine,
        ));
        return;
    }

    // We'll use a simple line drawing algorithm with segments
    // Number of segments based on length
    let num_segments = (length / 2.0).ceil().max(1.0) as usize;

    for i in 0..num_segments {
        let t = i as f32 / num_segments as f32;
        let next_t = (i + 1) as f32 / num_segments as f32;

        let x = x1 + dx * t;
        let y = y1 + dy * t;
        let next_x = x1 + dx * next_t;
        let next_y = y1 + dy * next_t;

        // For each segment, we'll place a small rectangular node
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                width: Val::Px(thickness),
                height: Val::Px(thickness),
                ..default()
            },
            BackgroundColor(line_color),
            GlyphOutlineLine,
        ));
    }
}

/// Helper function to check if a point is on-curve
fn is_on_curve(point: &norad::ContourPoint) -> bool {
    matches!(
        point.typ,
        norad::PointType::Move
            | norad::PointType::Line
            | norad::PointType::Curve
            | norad::PointType::QCurve
    )
}

/// Updates the glyph metrics for the current glyph
pub fn update_glyph_metrics(
    app_state: bevy::prelude::Res<data::AppState>,
    cli_args: bevy::prelude::Res<cli::CliArgs>,
    mut metrics: bevy::prelude::ResMut<CurrentGlyphMetrics>,
) {
    // Extract information from the current state

    // If no font is loaded, just return (resource will have default empty values)
    if app_state.workspace.font.ufo.font_info.is_none() {
        bevy::log::warn!("No font loaded, skipping glyph metrics update");
        return;
    }

    // Get information about the current glyph
    if let Some(glyph_name) = cli_args.find_glyph(&app_state.workspace.font.ufo)
    {
        // Found a glyph, get its details
        let glyph_name_str = glyph_name.to_string();
        bevy::log::info!("Updating metrics for glyph: {}", glyph_name_str);

        metrics.glyph_name = glyph_name_str.clone();

        // Set the Unicode information
        if let Some(codepoint) = &cli_args.test_unicode {
            bevy::log::info!("Glyph Unicode: {}", codepoint);
            metrics.unicode = codepoint.clone();
        } else {
            metrics.unicode = String::new();
        }

        // Try to get the glyph to extract its metrics
        if let Some(default_layer) =
            app_state.workspace.font.ufo.get_default_layer()
        {
            if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                // Get advance width
                bevy::log::info!("Glyph advance: {:?}", glyph.advance);

                // Extract the advance width value
                let mut advance_width: f32 = 0.0;
                if let Some(advance) = &glyph.advance {
                    // Extract just the width value
                    let advance_str = format!("{:?}", advance);
                    if let Some(width_pos) = advance_str.find("width:") {
                        if let Some(end_pos) =
                            advance_str[width_pos..].find("}")
                        {
                            let width_str = &advance_str
                                [width_pos + 6..width_pos + end_pos];
                            let clean_width =
                                width_str.trim().trim_end_matches(',');

                            if let Ok(width) = clean_width.parse::<f32>() {
                                advance_width = width;
                                metrics.advance = format!("{}", width as i32);
                                bevy::log::info!(
                                    "Parsed advance width: {}",
                                    metrics.advance
                                );
                            } else {
                                metrics.advance = "?".to_string();
                                bevy::log::warn!(
                                    "Failed to parse advance width"
                                );
                            }
                        } else {
                            metrics.advance = "?".to_string();
                        }
                    } else {
                        metrics.advance = "?".to_string();
                    }
                } else {
                    metrics.advance = "-".to_string();
                }

                // Calculate sidebearings based on glyph outline bounds
                // First, get the outline bounding box
                let outline_bounds = if let Some(outline) = &glyph.outline {
                    // Calculate bounds from outline contours
                    let mut min_x = f32::MAX;
                    let mut max_x = f32::MIN;

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
                    if has_points && min_x != f32::MAX && max_x != f32::MIN {
                        bevy::log::info!(
                            "Glyph outline bounds: min_x={}, max_x={}",
                            min_x,
                            max_x
                        );
                        Some((min_x, max_x))
                    } else {
                        bevy::log::warn!("Glyph has empty or invalid outline");
                        None
                    }
                } else {
                    bevy::log::warn!("Glyph has no outline");
                    None
                };

                // Calculate LSB and RSB if we have both outline bounds and advance width
                if let Some((min_x, max_x)) = outline_bounds {
                    // Left side bearing is the distance from origin to the leftmost point
                    let lsb = min_x;

                    // Right side bearing is the distance from the rightmost point to the advance width
                    let rsb = advance_width - max_x;

                    metrics.left_bearing = format!("{}", lsb as i32);
                    metrics.right_bearing = format!("{}", rsb as i32);

                    bevy::log::info!(
                        "Calculated sidebearings - LSB: {}, RSB: {}",
                        metrics.left_bearing,
                        metrics.right_bearing
                    );
                } else {
                    // If we couldn't calculate bounds, use placeholder values
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();

                    bevy::log::warn!(
                        "Using placeholder values for sidebearings"
                    );
                }

                bevy::log::info!("Updated glyph metrics successfully");
            } else {
                bevy::log::warn!("Failed to get glyph from default layer");
            }
        } else {
            bevy::log::warn!("No default layer in the font");
        }
    } else {
        // No glyph found, clear the metrics
        metrics.glyph_name = String::new();
        metrics.unicode = String::new();
        metrics.advance = "-".to_string();
        metrics.left_bearing = "-".to_string();
        metrics.right_bearing = "-".to_string();

        bevy::log::warn!("No glyph selected, cleared metrics");
    }
}
