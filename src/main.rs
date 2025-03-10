//! A font editor made with Rust and the Bevy game engine.
mod app;
mod cameras;
mod checkerboard;
mod cli;
mod commands;
mod crypto_toolbar;
mod data;
mod debug;
mod design_space;
mod draw;
mod edit_mode_toolbar;
mod glyph_pane;
mod hud;
mod logger;
mod plugins;
mod selection;
mod setup;
mod tests;
mod text_editor;
mod theme;
mod ufo;
mod virtual_font;

/// A system that updates glyph metrics information for the glyph pane
///
/// This is defined in main.rs because it needs access to multiple
/// modules that aren't accessible from the glyph_pane module.
pub fn update_glyph_metrics(
    app_state: bevy::prelude::Res<data::AppState>,
    cli_args: bevy::prelude::Res<cli::CliArgs>,
    mut metrics: bevy::prelude::ResMut<glyph_pane::CurrentGlyphMetrics>,
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
                                metrics.advance = format!("{}", clean_width);
                                bevy::log::info!(
                                    "Parsed advance width: {}",
                                    metrics.advance
                                );
                            } else {
                                metrics.advance = "?".to_string();
                                bevy::log::warn!("Failed to parse advance width");
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
                        bevy::log::info!("Glyph outline bounds: min_x={}, max_x={}", min_x, max_x);
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
                    
                    metrics.left_bearing = format!("{:.1}", lsb);
                    metrics.right_bearing = format!("{:.1}", rsb);
                    
                    bevy::log::info!("Calculated sidebearings - LSB: {}, RSB: {}", 
                                    metrics.left_bearing, metrics.right_bearing);
                } else {
                    // If we couldn't calculate bounds, use placeholder values
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();
                    
                    bevy::log::warn!("Using placeholder values for sidebearings");
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
        bevy::log::warn!("No glyph found for current selection");
        *metrics = glyph_pane::CurrentGlyphMetrics::default();
    }
}

fn main() {
    // Parse command line arguments
    let cli_args = cli::CliArgs::parse_args();
    // Create and run the app with the CLI arguments
    app::create_app(cli_args).run();
}
