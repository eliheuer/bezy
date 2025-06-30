//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Inactive sorts show as metrics boxes (similar to existing metrics display).
//! Active sorts show the actual glyph outlines for editing.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics};

use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR, MONO_FONT_PATH};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use std::collections::HashSet;

/// Component to mark text entities that display glyph names for sorts
#[derive(Component)]
pub struct SortGlyphNameText {
    pub sort_entity: Entity,
}

/// Component to mark text entities that display unicode values for sorts
#[derive(Component)]
pub struct SortUnicodeText {
    pub sort_entity: Entity,
}

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&crate::ui::panes::design_space::ViewPort>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<(Entity, &Sort), With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
    mut app_state_events: EventReader<crate::editing::selection::systems::AppStateChanged>,
) {
    // Only log when we actually receive AppStateChanged events (not every frame)
    let event_count = app_state_events.len();
    if event_count > 0 {
        info!("🎨 SORT RENDERER: Received {} AppStateChanged events - updating glyph rendering", event_count);
        // Consume the events
        for _ in app_state_events.read() {}
    }
    // Get viewport for coordinate transformations
    let viewport = match viewports.single() {
        Ok(viewport) => *viewport,
        Err(_) => crate::ui::panes::design_space::ViewPort::default(),
    };

    let font_metrics = &app_state.workspace.info.metrics;

    // Render inactive sorts as metrics boxes with glyph outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, &viewport, sort, font_metrics, &app_state);
    }

    // Render active sorts with full outline detail
    for (_entity, sort) in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, font_metrics, &app_state);
    }
}

/// System to manage text labels (glyph name and unicode) for all sorts
pub fn manage_sort_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
    sorts_query: Query<(Entity, &Sort), (Changed<Sort>, Or<(With<ActiveSort>, With<InactiveSort>)>)>,
    existing_name_text_query: Query<(Entity, &SortGlyphNameText)>,
    existing_unicode_text_query: Query<(Entity, &SortUnicodeText)>,
    all_sorts_query: Query<Entity, With<Sort>>,
    active_sorts_query: Query<(Entity, &Sort), With<ActiveSort>>,
    viewports: Query<&crate::ui::panes::design_space::ViewPort>,
) {
    // Remove text for sorts that no longer exist
    let existing_sort_entities: HashSet<Entity> = all_sorts_query.iter().collect();
    
    // Clean up glyph name text entities
    for (text_entity, sort_name_text) in existing_name_text_query.iter() {
        if !existing_sort_entities.contains(&sort_name_text.sort_entity) {
            commands.entity(text_entity).despawn();
        }
    }
    
    // Clean up unicode text entities
    for (text_entity, sort_unicode_text) in existing_unicode_text_query.iter() {
        if !existing_sort_entities.contains(&sort_unicode_text.sort_entity) {
            commands.entity(text_entity).despawn();
        }
    }

    // Get viewport for coordinate transformations
    let viewport = match viewports.single() {
        Ok(viewport) => *viewport,
        Err(_) => crate::ui::panes::design_space::ViewPort::default(),
    };

    // Create or update text labels for changed sorts
    for (sort_entity, sort) in sorts_query.iter() {
        // Determine text color based on sort state
        let text_color = if active_sorts_query.iter().any(|(entity, _)| entity == sort_entity) {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for inactive sorts
        };

        // Create combined text content: unicode on first line, glyph name wrapping on subsequent lines
        let combined_content = if let Some(unicode_value) = get_unicode_for_glyph(&sort.glyph_name, &app_state) {
            format!("U+{}\n{}", unicode_value, sort.glyph_name)
        } else {
            sort.glyph_name.clone()
        };

        // Calculate available width for text wrapping (sort width minus margins)
        let sort_width = sort.advance_width;
        let text_margin = 16.0; // Margin on all sides
        let available_width = (sort_width - (text_margin * 2.0)).max(120.0).min(sort_width - text_margin); // Conservative bounds
        
        // Debug: ensure we have reasonable text bounds
        // println!("Sort '{}': width={}, available_width={}", sort.glyph_name, sort_width, available_width);
        
        // Handle glyph name text (now combined with unicode)
        let existing_name_text_entity = existing_name_text_query.iter()
            .find(|(_, sort_name_text)| sort_name_text.sort_entity == sort_entity)
            .map(|(entity, _)| entity);

        let name_transform = calculate_glyph_name_transform(sort, &app_state.workspace.info.metrics, &viewport);

        match existing_name_text_entity {
            Some(text_entity) => {
                // Update existing text entity with combined content
                if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                    entity_commands.insert((
                        Text2d(combined_content),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: 24.0, // Even smaller font to fit more text
                            ..default()
                        },
                        TextColor(text_color),
                        TextLayout {
                            justify: JustifyText::Left,
                            linebreak: bevy::text::LineBreak::AnyCharacter,
                        },
                        TextBounds::new_horizontal(available_width), // Re-enable wrapping
                        Anchor::TopLeft,
                        name_transform,
                    ));
                }
            }
            None => {
                // Create new text entity with combined content
                commands.spawn((
                    Text2d(combined_content),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: 24.0, // Even smaller font to fit more text
                        ..default()
                    },
                    TextColor(text_color),
                    TextLayout {
                        justify: JustifyText::Left,
                        linebreak: bevy::text::LineBreak::AnyCharacter,
                    },
                    TextBounds::new_horizontal(available_width), // Re-enable wrapping
                    Anchor::TopLeft,
                    name_transform,
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    SortGlyphNameText { sort_entity },
                    Name::new(format!("GlyphNameText_{:?}", sort_entity)),
                ));
            }
        }

        // Remove any existing unicode text entities since we're now combining them
        let existing_unicode_text_entity = existing_unicode_text_query.iter()
            .find(|(_, sort_unicode_text)| sort_unicode_text.sort_entity == sort_entity)
            .map(|(entity, _)| entity);
        
        if let Some(text_entity) = existing_unicode_text_entity {
            commands.entity(text_entity).despawn();
        }
    }
}

/// System to update positions of text labels when sorts move
pub fn update_sort_label_positions(
    app_state: Res<AppState>,
    sorts_query: Query<&Sort, Changed<Sort>>,
    mut text_query: Query<(&mut Transform, &SortGlyphNameText)>,
    viewports: Query<&crate::ui::panes::design_space::ViewPort>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    
    // Get viewport for coordinate transformations
    let viewport = match viewports.single() {
        Ok(viewport) => *viewport,
        Err(_) => crate::ui::panes::design_space::ViewPort::default(),
    };
    
    // Update text positions (now only glyph name text which contains both unicode and name)
    for (mut text_transform, sort_name_text) in text_query.iter_mut() {
        if let Ok(sort) = sorts_query.get(sort_name_text.sort_entity) {
            *text_transform = calculate_glyph_name_transform(sort, font_metrics, &viewport);
        }
    }
}

/// System to update text label colors when sorts change state (active/inactive)
pub fn update_sort_label_colors(
    active_sorts_query: Query<Entity, (With<Sort>, With<ActiveSort>)>,
    inactive_sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    mut text_query: Query<(&mut TextColor, &SortGlyphNameText)>,
) {
    // Helper function to determine color
    let get_color = |sort_entity: Entity| -> Color {
        if active_sorts_query.get(sort_entity).is_ok() {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else if inactive_sorts_query.get(sort_entity).is_ok() {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for inactive sorts
        } else {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Default color
        }
    };
    
    // Update text colors (now only glyph name text which contains both unicode and name)
    for (mut text_color, sort_name_text) in text_query.iter_mut() {
        *text_color = TextColor(get_color(sort_name_text.sort_entity));
    }
}

/// Calculate the transform for positioning glyph name text in the upper left corner (first line)
fn calculate_glyph_name_transform(sort: &Sort, font_metrics: &FontMetrics, viewport: &crate::ui::panes::design_space::ViewPort) -> Transform {
    let upm = font_metrics.units_per_em as f32;
    let text_margin = 16.0; // Consistent margin on all sides
    
    // Position text in upper left corner with proper margins
    // sort.position.y is the baseline, so UPM top is at sort.position.y + upm
    let design_x = sort.position.x + text_margin; // Margin from left edge of sort
    let design_y = sort.position.y + upm - text_margin; // Position below the UPM top with margin
    
    // Convert from design space to screen space using the viewport
    let design_point = crate::ui::panes::design_space::DPoint::new(design_x, design_y);
    let screen_point = viewport.to_screen(design_point);
    
    Transform::from_translation(Vec3::new(screen_point.x, screen_point.y, 10.0)) // Higher Z to render above sorts
}



/// Get the unicode value for a given glyph name
fn get_unicode_for_glyph(glyph_name: &str, app_state: &AppState) -> Option<String> {
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
        if !glyph_data.unicode_values.is_empty() {
            if let Some(&first_codepoint) = glyph_data.unicode_values.first() {
                return Some(format!("{:04X}", first_codepoint as u32));
            }
        }
    }
    None
}

/// Render an inactive sort with metrics box and glyph outline only
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    viewport: &crate::ui::panes::design_space::ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our internal data
    let glyph = app_state.workspace.font.glyphs.get(&sort.glyph_name);

    if let Some(glyph) = glyph {
        // Convert our internal glyph data to norad format for metrics rendering
        let norad_glyph = glyph.to_norad_glyph();
        
        // First render the metrics box using the inactive color and proper viewport
        crate::rendering::metrics::draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            &norad_glyph,
            font_metrics,
            sort.position,
            SORT_INACTIVE_METRICS_COLOR,
        );
        
        // Then render only the glyph outline (no control handles) if it exists
        if let Some(outline) = &glyph.outline {
            // Draw only the path, no control handles for inactive sorts
            for contour in outline.contours.iter() {
                crate::rendering::glyph_outline::draw_contour_path_at_position(
                    gizmos,
                    viewport,
                    contour,
                    sort.position,
                );
            }
        }
    }
}

/// Render an active sort with full glyph outline and control handles
fn render_active_sort(
    gizmos: &mut Gizmos,
    viewport: &crate::ui::panes::design_space::ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our internal data
    let glyph = app_state.workspace.font.glyphs.get(&sort.glyph_name);

    if let Some(glyph) = glyph {
        // Convert our internal glyph data to norad format for metrics rendering
        let norad_glyph = glyph.to_norad_glyph();
        
        // First render the metrics box using the active color and proper viewport
        crate::rendering::metrics::draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            &norad_glyph,
            font_metrics,
            sort.position,
            SORT_ACTIVE_METRICS_COLOR,
        );
        
        // Then render the full glyph outline with control handles if it exists
        if let Some(outline) = &glyph.outline {
            crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
            
            // Also render the glyph points (on-curve and off-curve)
            crate::rendering::glyph_outline::draw_glyph_points_at_position(
                gizmos,
                viewport,
                outline,
                sort.position,
            );
        }
    }
}