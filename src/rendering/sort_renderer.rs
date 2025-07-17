//! Sort rendering system
//!
//! Handles the visual representation of sorts in both active and inactive modes.
//! Inactive sorts show as metrics boxes (similar to existing metrics display).
//! Active sorts show the actual glyph outlines for editing.

use crate::core::state::{AppState, FontMetrics, SortLayoutMode, FontIRAppState};
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use crate::rendering::cameras::DesignCamera;
use crate::rendering::sort_visuals::{render_sort_visuals, render_fontir_sort_visuals, SortRenderStyle};
use kurbo::BezPath;

use crate::ui::theme::{
    MONO_FONT_PATH, SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR,
};
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
#[allow(clippy::too_many_arguments)]
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<
        (
            Entity,
            &Sort,
            &Transform,
            Option<&crate::editing::selection::components::Selected>,
        ),
        With<ActiveSort>,
    >,
    inactive_sorts_query: Query<
        (
            Entity,
            &Sort,
            &Transform,
            Option<&crate::editing::selection::components::Selected>,
        ),
        With<InactiveSort>,
    >,
    // Additional parameters for live rendering
    nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
    point_query: Query<
        (
            Entity,
            &Transform,
            &crate::editing::selection::components::GlyphPointReference,
            &crate::editing::selection::components::PointType,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
    selected_query: Query<
        Entity,
        With<crate::editing::selection::components::Selected>,
    >,
) {
    // Check which state is available and get font metrics
    let fontir_font_metrics;
    let (font_metrics, app_state_deref) = if let Some(app_state) = app_state.as_ref() {
        (&app_state.workspace.info.metrics, Some(app_state))
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        // Using FontIR - create FontMetrics from FontIR data
        let fontir_metrics = fontir_state.get_font_metrics();
        fontir_font_metrics = crate::core::state::FontMetrics {
            units_per_em: fontir_metrics.units_per_em as f64,
            ascender: fontir_metrics.ascender.map(|a| a as f64),
            descender: fontir_metrics.descender.map(|d| d as f64),
            line_height: fontir_metrics.line_gap.unwrap_or(0.0) as f64,
            x_height: None,
            cap_height: None,
            italic_angle: None,
        };
        (&fontir_font_metrics, None)
    } else {
        error!("Neither AppState nor FontIRAppState available for sort rendering");
        return;
    };

    // Render inactive sorts (both buffer and freeform)
    for (entity, sort, transform, selected) in inactive_sorts_query.iter() {
        let position = transform.translation.truncate();

        // Choose render style based on layout mode
        let render_style = match sort.layout_mode {
            SortLayoutMode::Text => SortRenderStyle::TextBuffer,
            SortLayoutMode::Freeform => SortRenderStyle::Freeform,
        };

        let is_selected = selected.is_some();
        let metrics_color = if is_selected {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for selected inactive sorts
        } else {
            SORT_INACTIVE_METRICS_COLOR
        };

        // Try FontIR first, fallback to old system
        if let Some(fontir_state) = &fontir_app_state {
            // Use FontIR rendering
            // For now, use a placeholder advance width since we haven't implemented metric extraction from FontIR yet
            let advance_width = 600.0; // TODO: Get from FontIR
            
            render_fontir_sort_visuals(
                &mut gizmos,
                fontir_state,
                &sort.glyph_name,
                advance_width,
                font_metrics,
                position,
                metrics_color,
                render_style,
            );
        } else if let Some(app_state) = app_state_deref {
            if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&sort.glyph_name) {
            // Fallback to old system
            let advance_width = glyph_data.advance_width as f32;

            crate::rendering::sort_visuals::render_sort_visuals_with_live_sync(
                &mut gizmos,
                &glyph_data.outline,
                advance_width,
                font_metrics,
                position,
                metrics_color,
                render_style,
                // Live rendering parameters
                Some(entity),
                Some(transform),
                Some(&sort.glyph_name),
                Some(&point_query),
                Some(&selected_query),
                Some(&*app_state),
                Some(&*nudge_state),
            );
            }
        }
    }

    // Render active sorts (both buffer and freeform)
    for (entity, sort, transform, _selected) in active_sorts_query.iter() {
        let position = transform.translation.truncate();

        // Choose render style based on layout mode
        let render_style = match sort.layout_mode {
            SortLayoutMode::Text => SortRenderStyle::TextBuffer,
            SortLayoutMode::Freeform => SortRenderStyle::Freeform,
        };

        // Active sorts are green, selected active sorts might be a different shade
        let metrics_color = SORT_ACTIVE_METRICS_COLOR; // Green for active sorts

        // Try FontIR first, fallback to old system
        if let Some(fontir_state) = &fontir_app_state {
            // Use FontIR rendering
            // For now, use a placeholder advance width since we haven't implemented metric extraction from FontIR yet
            let advance_width = 600.0; // TODO: Get from FontIR
            
            render_fontir_sort_visuals(
                &mut gizmos,
                fontir_state,
                &sort.glyph_name,
                advance_width,
                font_metrics,
                position,
                metrics_color,
                render_style,
            );
        } else if let Some(app_state) = app_state_deref {
            if let Some(glyph_data) = app_state.workspace.font.glyphs.get(&sort.glyph_name) {
            // Fallback to old system
            let advance_width = glyph_data.advance_width as f32;

            crate::rendering::sort_visuals::render_sort_visuals_with_live_sync(
                &mut gizmos,
                &glyph_data.outline,
                advance_width,
                font_metrics,
                position,
                metrics_color,
                render_style,
                // Live rendering parameters
                Some(entity),
                Some(transform),
                Some(&sort.glyph_name),
                Some(&point_query),
                Some(&selected_query),
                Some(&*app_state),
                Some(&*nudge_state),
            );
            }
        }
    }
}

/// System to manage text labels (glyph name and unicode) for all sorts
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn manage_sort_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Option<Res<AppState>>,
    sorts_query: Query<
        (Entity, &Sort, &Transform),
        (Changed<Sort>, Or<(With<ActiveSort>, With<InactiveSort>)>),
    >,
    existing_name_text_query: Query<(Entity, &SortGlyphNameText)>,
    existing_unicode_text_query: Query<(Entity, &SortUnicodeText)>,
    all_sorts_query: Query<Entity, With<Sort>>,
    active_sorts_query: Query<(Entity, &Sort), With<ActiveSort>>,
) {
    // Early return if AppState not available
    let Some(app_state) = app_state else {
        warn!("Sort labels skipped - AppState not available (using FontIR)");
        return;
    };
    
    // Remove text for sorts that no longer exist
    let existing_sort_entities: HashSet<Entity> =
        all_sorts_query.iter().collect();

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

    // Create or update text labels for changed sorts
    for (sort_entity, sort, transform) in sorts_query.iter() {
        // Determine text color based on sort state
        let text_color = if active_sorts_query
            .iter()
            .any(|(entity, _)| entity == sort_entity)
        {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for inactive sorts
        };

        // Create combined text content: unicode on first line, glyph name wrapping on subsequent lines
        let combined_content = if let Some(unicode_value) =
            get_unicode_for_glyph(&sort.glyph_name, &app_state)
        {
            format!("U+{}\n{}", unicode_value, sort.glyph_name)
        } else {
            sort.glyph_name.clone()
        };

        // Calculate available width for text wrapping (sort width minus margins)
        let sort_width = if let Some(glyph_data) =
            app_state.workspace.font.glyphs.get(&sort.glyph_name)
        {
            glyph_data.advance_width as f32
        } else {
            600.0 // Default fallback
        };
        let text_margin = 16.0; // Margin on all sides
        let available_width = (sort_width - (text_margin * 2.0))
            .max(120.0)
            .min(sort_width - text_margin); // Conservative bounds

        // Debug: ensure we have reasonable text bounds
        // println!("Sort '{}': width={}, available_width={}", sort.glyph_name, sort_width, available_width);

        // Handle glyph name text (now combined with unicode)
        let existing_name_text_entity = existing_name_text_query
            .iter()
            .find(|(_, sort_name_text)| {
                sort_name_text.sort_entity == sort_entity
            })
            .map(|(entity, _)| entity);

        let name_transform = calculate_glyph_name_transform(
            sort,
            transform,
            &app_state.workspace.info.metrics,
        );

        match existing_name_text_entity {
            Some(text_entity) => {
                // Update existing text entity with combined content
                if let Ok(mut entity_commands) =
                    commands.get_entity(text_entity)
                {
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
                    Name::new(format!("GlyphNameText_{sort_entity:?}")),
                ));
            }
        }

        // Remove any existing unicode text entities since we're now combining them
        let existing_unicode_text_entity = existing_unicode_text_query
            .iter()
            .find(|(_, sort_unicode_text)| {
                sort_unicode_text.sort_entity == sort_entity
            })
            .map(|(entity, _)| entity);

        if let Some(text_entity) = existing_unicode_text_entity {
            commands.entity(text_entity).despawn();
        }
    }
}

/// System to update positions of text labels when sorts move
pub fn update_sort_label_positions(
    app_state: Option<Res<AppState>>,
    sorts_query: Query<(&Sort, &Transform), Changed<Sort>>,
    mut text_query: Query<(&mut Transform, &SortGlyphNameText)>,
) {
    // Early return if AppState not available
    let Some(app_state) = app_state else {
        warn!("Sort label position updates skipped - AppState not available (using FontIR)");
        return;
    };
    let font_metrics = &app_state.workspace.info.metrics;

    // Update text positions (now only glyph name text which contains both unicode and name)
    for (mut text_transform, sort_name_text) in text_query.iter_mut() {
        if let Ok((sort, transform)) =
            sorts_query.get(sort_name_text.sort_entity)
        {
            *text_transform =
                calculate_glyph_name_transform(sort, transform, font_metrics);
        }
    }
}

/// System to update text label colors when sorts change state (active/inactive)
pub fn update_sort_label_colors(
    active_sorts_query: Query<Entity, (With<Sort>, With<ActiveSort>)>,
    _inactive_sorts_query: Query<Entity, (With<Sort>, With<InactiveSort>)>,
    mut text_query: Query<(&mut TextColor, &SortGlyphNameText)>,
) {
    // Helper function to determine color
    let get_color = |sort_entity: Entity| -> Color {
        if active_sorts_query.get(sort_entity).is_ok() {
            SORT_ACTIVE_METRICS_COLOR // Green for active sorts
        } else {
            Color::srgba(0.8, 0.8, 0.8, 0.9) // Light gray for inactive sorts and default
        }
    };

    // Update text colors (now only glyph name text which contains both unicode and name)
    for (mut text_color, sort_name_text) in text_query.iter_mut() {
        *text_color = TextColor(get_color(sort_name_text.sort_entity));
    }
}

/// Calculate the transform for positioning glyph name text in the upper left corner (first line)
fn calculate_glyph_name_transform(
    _sort: &Sort,
    transform: &Transform,
    font_metrics: &FontMetrics,
) -> Transform {
    let upm = font_metrics.units_per_em as f32;
    let text_margin = 16.0; // Consistent margin on all sides
    let position = transform.translation.truncate();

    // Position text in upper left corner with proper margins
    // position.y is the baseline, so UPM top is at position.y + upm
    let design_x = position.x + text_margin; // Margin from left edge of sort
    let design_y = position.y + upm - text_margin; // Position below the UPM top with margin

    // Use design space coordinates directly since ViewPort is deprecated
    Transform::from_translation(Vec3::new(design_x, design_y, 10.0)) // Higher Z to render above sorts
}

/// Get the unicode value for a given glyph name
fn get_unicode_for_glyph(
    glyph_name: &str,
    app_state: &AppState,
) -> Option<String> {
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
        if !glyph_data.unicode_values.is_empty() {
            if let Some(&first_codepoint) = glyph_data.unicode_values.first() {
                return Some(format!("{:04X}", first_codepoint as u32));
            }
        }
    }
    None
}
