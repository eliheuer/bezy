//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Inactive sorts show as metrics boxes (similar to existing metrics display).
//! Active sorts show the actual glyph outlines for editing.

use crate::editing::sort::{Sort, ActiveSort, InactiveSort};
use crate::core::state::{AppState, FontMetrics};
use crate::ui::panes::design_space::ViewPort;
use crate::ui::theme::{SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR, MONO_FONT_PATH};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::collections::HashSet;

/// Component to mark text entities that display unicode values for sorts
#[derive(Component)]
pub struct SortUnicodeText {
    pub sort_entity: Entity,
}

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    viewports: Query<&ViewPort>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    // Get viewport for coordinate transformations
    let viewport = match viewports.get_single() {
        Ok(viewport) => *viewport,
        Err(_) => ViewPort::default(),
    };

    let font_metrics = &app_state.workspace.info.metrics;

    // Render inactive sorts as metrics boxes with glyph outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, &viewport, sort, font_metrics, &app_state);
    }

    // Render active sorts with full outline detail
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, &viewport, sort, font_metrics, &app_state);
    }
}

/// System to manage unicode text entities for all sorts
pub fn manage_sort_unicode_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
    sorts_query: Query<(Entity, &Sort), (Changed<Sort>, Or<(With<ActiveSort>, With<InactiveSort>)>)>,
    existing_text_query: Query<(Entity, &SortUnicodeText)>,
    all_sorts_query: Query<Entity, With<Sort>>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
) {
    // Remove text for sorts that no longer exist
    let existing_sort_entities: HashSet<Entity> = all_sorts_query.iter().collect();
    for (text_entity, sort_unicode_text) in existing_text_query.iter() {
        if !existing_sort_entities.contains(&sort_unicode_text.sort_entity) {
            commands.entity(text_entity).despawn();
        }
    }

    // Create or update text for changed sorts
    for (sort_entity, sort) in sorts_query.iter() {
        // Check if text entity already exists for this sort
        let existing_text_entity = existing_text_query.iter()
            .find(|(_, sort_unicode_text)| sort_unicode_text.sort_entity == sort_entity)
            .map(|(entity, _)| entity);

        if let Some(unicode_value) = get_unicode_for_glyph(&sort.glyph_name, &app_state) {
            let text_content = format!("U+{}", unicode_value);
            
            // Determine text color based on sort state (match metrics line colors)
            let text_color = if active_sorts_query.get(sort_entity).is_ok() {
                SORT_ACTIVE_METRICS_COLOR // Green for active sorts
            } else {
                Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for better readability on dark background
            };
            
            match existing_text_entity {
                Some(text_entity) => {
                    // Update existing text entity
                    if let Some(mut entity_commands) = commands.get_entity(text_entity) {
                        entity_commands.insert((
                            Text2d(text_content),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: 48.0, // Reduced from 128.0 to fit within sort boundaries
                                ..default()
                            },
                            TextColor(text_color),
                            TextLayout::new_with_justify(JustifyText::Right), // Right-align the text
                            Anchor::TopRight, // Anchor the text at its top-right corner
                            calculate_text_transform(sort, &app_state.workspace.info.metrics),
                        ));
                    }
                }
                                 None => {
                     // Create new text entity
                     commands.spawn((
                         Text2d(text_content),
                         TextFont {
                             font: asset_server.load(MONO_FONT_PATH),
                             font_size: 48.0, // Reduced from 128.0 to fit within sort boundaries
                             ..default()
                         },
                         TextColor(text_color),
                         TextLayout::new_with_justify(JustifyText::Right), // Right-align the text
                         Anchor::TopRight, // Anchor the text at its top-right corner
                         calculate_text_transform(sort, &app_state.workspace.info.metrics),
                         GlobalTransform::default(),
                         Visibility::Visible,
                         InheritedVisibility::default(),
                         ViewVisibility::default(),
                         SortUnicodeText { sort_entity },
                         Name::new(format!("UnicodeText_{:?}", sort_entity)),
                     ));
                 }
            }
        } else if let Some(text_entity) = existing_text_entity {
            // Remove text entity if glyph has no unicode value
            commands.entity(text_entity).despawn();
        }
    }
}

/// System to update positions of unicode text when sorts move
pub fn update_sort_unicode_text_positions(
    app_state: Res<AppState>,
    sorts_query: Query<&Sort, Changed<Sort>>,
    mut text_query: Query<(&mut Transform, &SortUnicodeText)>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    
    for (mut text_transform, sort_unicode_text) in text_query.iter_mut() {
        if let Ok(sort) = sorts_query.get(sort_unicode_text.sort_entity) {
            *text_transform = calculate_text_transform(sort, font_metrics);
        }
    }
}

/// System to update unicode text colors when sorts change state (active/inactive)
pub fn update_sort_unicode_text_colors(
    active_sorts_query: Query<Entity, (With<Sort>, With<ActiveSort>)>,
    inactive_sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    mut text_query: Query<(&mut TextColor, &SortUnicodeText)>,
) {
    for (mut text_color, sort_unicode_text) in text_query.iter_mut() {
        // Determine the color based on whether the sort is active or inactive
        let new_color = if active_sorts_query.get(sort_unicode_text.sort_entity).is_ok() {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else if inactive_sorts_query.get(sort_unicode_text.sort_entity).is_ok() {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for better readability on dark background
        } else {
            continue; // Sort doesn't exist, skip
        };
        
        *text_color = TextColor(new_color);
    }
}

/// Calculate the transform for positioning unicode text in the upper right corner of the sort
fn calculate_text_transform(sort: &Sort, font_metrics: &FontMetrics) -> Transform {
    let upm = font_metrics.units_per_em as f32;
    let width = sort.advance_width;
    
    // Position text in upper right corner with larger margins to ensure it stays within sort bounds
    // Since we're using TopRight anchor, position at the exact spot where we want the top-right of text
    let text_x = sort.position.x + width - 48.0; // Increased margin from right edge (was 32.0)
    let text_y = sort.position.y + upm - 32.0; // Increased margin from top of UPM
    
    Transform::from_translation(Vec3::new(text_x, text_y, 10.0)) // Higher Z to render above sorts
}

/// Get the unicode value for a given glyph name
fn get_unicode_for_glyph(glyph_name: &str, app_state: &AppState) -> Option<String> {
    if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        if let Some(glyph) = default_layer.get_glyph(glyph_name) {
            if let Some(codepoints) = &glyph.codepoints {
                if let Some(&first_codepoint) = codepoints.first() {
                    return Some(format!("{:04X}", first_codepoint as u32));
                }
            }
        }
    }
    None
}

/// Render an inactive sort with metrics box and glyph outline only
fn render_inactive_sort(
    gizmos: &mut Gizmos,
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from the virtual font
    let glyph = if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        default_layer.get_glyph(&sort.glyph_name)
    } else {
        None
    };

    if let Some(glyph) = glyph {
        // First render the metrics box using the inactive color
        crate::rendering::metrics::draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            glyph,
            font_metrics,
            sort.position,
            SORT_INACTIVE_METRICS_COLOR,
        );
        
        // Then render only the glyph outline (no control handles) if it exists
        if let Some(outline) = &glyph.outline {
            // Render each contour in the outline
            for contour in &outline.contours {
                if contour.points.is_empty() {
                    continue;
                }

                // Draw only the path, no control handles for inactive sorts
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
    viewport: &ViewPort,
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from the virtual font
    let glyph = if let Some(default_layer) = app_state.workspace.font.ufo.get_default_layer() {
        default_layer.get_glyph(&sort.glyph_name)
    } else {
        None
    };

    if let Some(glyph) = glyph {
        // First render the metrics box using the active color
        crate::rendering::metrics::draw_metrics_at_position_with_color(
            gizmos,
            viewport,
            glyph,
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