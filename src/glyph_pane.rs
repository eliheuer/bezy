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
    // Create the position properties for the glyph pane (bottom left)
    let position_props = UiRect {
        left: Val::Px(WIDGET_MARGIN),
        bottom: Val::Px(WIDGET_MARGIN),
        top: Val::Auto,  // Explicitly set top to Auto to prevent stretching
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
            // Glyph name
            parent.spawn((
                create_widget_text(
                    asset_server,
                    "Glyph: Loading...",
                    WIDGET_TITLE_FONT_SIZE,
                    TEXT_COLOR,
                ),
                GlyphNameText,
            ));

            // Unicode value
            parent.spawn((
                create_widget_text(
                    asset_server,
                    "Unicode: Loading...",
                    WIDGET_TEXT_FONT_SIZE,
                    TEXT_COLOR,
                ),
                GlyphUnicodeText,
            ));

            // Advance width
            parent.spawn((
                create_widget_text(
                    asset_server,
                    "Advance: Loading...",
                    WIDGET_TEXT_FONT_SIZE,
                    TEXT_COLOR,
                ),
                GlyphAdvanceText,
            ));

            // Left side bearing
            parent.spawn((
                create_widget_text(
                    asset_server,
                    "LSB: Loading...",
                    WIDGET_TEXT_FONT_SIZE,
                    TEXT_COLOR,
                ),
                GlyphLeftBearingText,
            ));

            // Right side bearing
            parent.spawn((
                create_widget_text(
                    asset_server,
                    "RSB: Loading...",
                    WIDGET_TEXT_FONT_SIZE,
                    TEXT_COLOR,
                ),
                GlyphRightBearingText,
            ));
        });
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
        metrics.glyph_name = glyph_name_str.clone();

        // Set the Unicode information
        if let Some(codepoint) = &cli_args.test_unicode {
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
                } else {
                    // If we couldn't calculate bounds, use placeholder values
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();

                    bevy::log::warn!(
                        "Using placeholder values for sidebearings"
                    );
                }
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
