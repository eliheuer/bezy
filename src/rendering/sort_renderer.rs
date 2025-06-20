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
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
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
    for sort in active_sorts_query.iter() {
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
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
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
        let text_color = if active_sorts_query.get(sort_entity).is_ok() {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for inactive sorts
        };

        // Handle glyph name text (first line)
        let existing_name_text_entity = existing_name_text_query.iter()
            .find(|(_, sort_name_text)| sort_name_text.sort_entity == sort_entity)
            .map(|(entity, _)| entity);

        let glyph_name_content = sort.glyph_name.clone();
        let name_transform = calculate_glyph_name_transform(sort, &app_state.workspace.info.metrics, &viewport);

        match existing_name_text_entity {
            Some(text_entity) => {
                // Update existing glyph name text entity
                if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                    entity_commands.insert((
                        Text2d(glyph_name_content),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(text_color),
                        TextLayout::new_with_justify(JustifyText::Left),
                        Anchor::TopLeft,
                        name_transform,
                    ));
                }
            }
            None => {
                // Create new glyph name text entity
                commands.spawn((
                    Text2d(glyph_name_content),
                    TextFont {
                        font: asset_server.load(MONO_FONT_PATH),
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(text_color),
                    TextLayout::new_with_justify(JustifyText::Left),
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

        // Handle unicode text (second line)
        let existing_unicode_text_entity = existing_unicode_text_query.iter()
            .find(|(_, sort_unicode_text)| sort_unicode_text.sort_entity == sort_entity)
            .map(|(entity, _)| entity);

        if let Some(unicode_value) = get_unicode_for_glyph(&sort.glyph_name, &app_state) {
            let unicode_content = format!("U+{}", unicode_value);
            let unicode_transform = calculate_unicode_transform(sort, &app_state.workspace.info.metrics, &viewport);

            match existing_unicode_text_entity {
                Some(text_entity) => {
                    // Update existing unicode text entity
                    if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                        entity_commands.insert((
                            Text2d(unicode_content),
                            TextFont {
                                font: asset_server.load(MONO_FONT_PATH),
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor(text_color),
                            TextLayout::new_with_justify(JustifyText::Left),
                            Anchor::TopLeft,
                            unicode_transform,
                        ));
                    }
                }
                None => {
                    // Create new unicode text entity
                    commands.spawn((
                        Text2d(unicode_content),
                        TextFont {
                            font: asset_server.load(MONO_FONT_PATH),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(text_color),
                        TextLayout::new_with_justify(JustifyText::Left),
                        Anchor::TopLeft,
                        unicode_transform,
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        SortUnicodeText { sort_entity },
                        Name::new(format!("UnicodeText_{:?}", sort_entity)),
                    ));
                }
            }
        } else if let Some(text_entity) = existing_unicode_text_entity {
            // Remove unicode text entity if glyph has no unicode value
            commands.entity(text_entity).despawn();
        }
    }
}

/// System to update positions of text labels when sorts move
pub fn update_sort_label_positions(
    app_state: Res<AppState>,
    sorts_query: Query<&Sort, Changed<Sort>>,
    mut text_queries: ParamSet<(
        Query<(&mut Transform, &SortGlyphNameText)>,
        Query<(&mut Transform, &SortUnicodeText)>,
    )>,
    viewports: Query<&crate::ui::panes::design_space::ViewPort>,
) {
    let font_metrics = &app_state.workspace.info.metrics;
    
    // Get viewport for coordinate transformations
    let viewport = match viewports.single() {
        Ok(viewport) => *viewport,
        Err(_) => crate::ui::panes::design_space::ViewPort::default(),
    };
    
    // Update glyph name text positions
    for (mut text_transform, sort_name_text) in text_queries.p0().iter_mut() {
        if let Ok(sort) = sorts_query.get(sort_name_text.sort_entity) {
            *text_transform = calculate_glyph_name_transform(sort, font_metrics, &viewport);
        }
    }
    
    // Update unicode text positions
    for (mut text_transform, sort_unicode_text) in text_queries.p1().iter_mut() {
        if let Ok(sort) = sorts_query.get(sort_unicode_text.sort_entity) {
            *text_transform = calculate_unicode_transform(sort, font_metrics, &viewport);
        }
    }
}

/// System to update text label colors when sorts change state (active/inactive)
pub fn update_sort_label_colors(
    active_sorts_query: Query<Entity, (With<Sort>, With<ActiveSort>)>,
    inactive_sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    mut text_queries: ParamSet<(
        Query<(&mut TextColor, &SortGlyphNameText)>,
        Query<(&mut TextColor, &SortUnicodeText)>,
    )>,
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
    
    // Update glyph name text colors
    for (mut text_color, sort_name_text) in text_queries.p0().iter_mut() {
        *text_color = TextColor(get_color(sort_name_text.sort_entity));
    }
    
    // Update unicode text colors
    for (mut text_color, sort_unicode_text) in text_queries.p1().iter_mut() {
        *text_color = TextColor(get_color(sort_unicode_text.sort_entity));
    }
}

/// Calculate the transform for positioning glyph name text in the upper left corner (first line)
fn calculate_glyph_name_transform(sort: &Sort, font_metrics: &FontMetrics, viewport: &crate::ui::panes::design_space::ViewPort) -> Transform {
    let upm = font_metrics.units_per_em as f32;
    
    // Position text in upper left corner of the UPM box (like metrics rendering)
    // sort.position.y is the baseline, so UPM top is at sort.position.y + upm
    let design_x = sort.position.x + 16.0; // Small margin from left edge of sort
    let design_y = sort.position.y + upm + 16.0; // Position above the UPM top
    
    // Convert from design space to screen space using the viewport
    let design_point = crate::ui::panes::design_space::DPoint::new(design_x, design_y);
    let screen_point = viewport.to_screen(design_point);
    
    Transform::from_translation(Vec3::new(screen_point.x, screen_point.y, 10.0)) // Higher Z to render above sorts
}

/// Calculate the transform for positioning unicode text in the upper left corner (second line)
fn calculate_unicode_transform(sort: &Sort, font_metrics: &FontMetrics, viewport: &crate::ui::panes::design_space::ViewPort) -> Transform {
    let upm = font_metrics.units_per_em as f32;
    
    // Position text in upper left corner, below the glyph name
    // sort.position.y is the baseline, so UPM top is at sort.position.y + upm
    let design_x = sort.position.x + 16.0; // Small margin from left edge of sort
    let design_y = sort.position.y + upm + 16.0 - 52.0; // Below glyph name (font size + small gap)
    
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