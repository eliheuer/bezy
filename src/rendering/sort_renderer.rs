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

/// Component to mark text entities that display unicode values for sorts
#[derive(Component)]
pub struct SortUnicodeText {
    pub sort_entity: Entity,
}

/// System to render all sorts in the design space
pub fn render_sorts_system(
    mut gizmos: Gizmos,
    app_state: Res<AppState>,
    _sorts_query: Query<&Sort>,
    active_sorts_query: Query<&Sort, With<ActiveSort>>,
    inactive_sorts_query: Query<&Sort, With<InactiveSort>>,
) {
    let font_metrics = &app_state.workspace.info.metrics;

    // Render inactive sorts as metrics boxes with glyph outlines
    for sort in inactive_sorts_query.iter() {
        render_inactive_sort(&mut gizmos, sort, font_metrics, &app_state);
    }

    // Render active sorts with full outline detail
    for sort in active_sorts_query.iter() {
        render_active_sort(&mut gizmos, sort, font_metrics, &app_state);
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
                    if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
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
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
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
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our thread-safe font data
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        // First render the metrics box using the inactive color
        draw_sort_metrics_box(
            gizmos,
            glyph_data,
            font_metrics,
            sort.position,
            SORT_INACTIVE_METRICS_COLOR,
        );
        
        // Then render only the glyph outline (no control handles) if it exists
        if let Some(outline) = &glyph_data.outline {
            // Render each contour in the outline
            for contour in &outline.contours {
                if contour.points.is_empty() {
                    continue;
                }

                // Draw only the path, no control handles for inactive sorts
                draw_contour_path_direct(
                    gizmos,
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
    sort: &Sort,
    font_metrics: &FontMetrics,
    app_state: &AppState,
) {
    // Get the glyph from our thread-safe font data
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
        // First render the metrics box using the active color
        draw_sort_metrics_box(
            gizmos,
            glyph_data,
            font_metrics,
            sort.position,
            SORT_ACTIVE_METRICS_COLOR,
        );
        
        // Then render the full glyph outline with control handles if it exists
        if let Some(outline) = &glyph_data.outline {
            draw_glyph_outline_direct(
                gizmos,
                outline,
                sort.position,
            );
            
            // Also render the glyph points (on-curve and off-curve)
            draw_glyph_points_direct(
                gizmos,
                outline,
                sort.position,
            );
        }
    }
}

/// Draw metrics box for a sort using GlyphData
fn draw_sort_metrics_box(
    gizmos: &mut Gizmos,
    glyph_data: &crate::core::state::GlyphData,
    font_metrics: &FontMetrics,
    position: Vec2,
    color: Color,
) {
    let upm = font_metrics.units_per_em as f32;
    let ascender = font_metrics.ascender.unwrap_or(800.0) as f32;
    let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
    let width = glyph_data.advance_width as f32;

    // All coordinates are offset by the position
    let offset_x = position.x;
    let offset_y = position.y;

    // Draw the UPM bounding box (from 0 to UPM height) - this is the key fix!
    gizmos.rect_2d(
        Vec2::new(offset_x + width / 2.0, offset_y + upm / 2.0),
        Vec2::new(width, upm),
        color,
    );

    // Draw baseline (most important)
    gizmos.line_2d(
        Vec2::new(offset_x, offset_y),
        Vec2::new(offset_x + width, offset_y),
        color,
    );

    // Optional: Draw ascender and descender lines for reference
    if ascender != 0.0 {
        gizmos.line_2d(
            Vec2::new(offset_x, offset_y + ascender),
            Vec2::new(offset_x + width, offset_y + ascender),
            color,
        );
    }

    if descender != 0.0 {
        gizmos.line_2d(
            Vec2::new(offset_x, offset_y + descender),
            Vec2::new(offset_x + width, offset_y + descender),
            color,
        );
    }
}

/// Draw a contour path directly without viewport transformation using kurbo
fn draw_contour_path_direct(
    gizmos: &mut Gizmos,
    contour: &crate::core::state::ContourData,
    offset: Vec2,
) {
    let points = &contour.points;
    if points.is_empty() {
        return;
    }

    // Convert contour to kurbo BezPath
    let bez_path = contour_to_bezpath(contour, offset);
    
    // Draw the path using kurbo's flatten method for smooth curves
    draw_bezpath_direct(gizmos, &bez_path, crate::ui::theme::PATH_LINE_COLOR);
}

/// Convert a contour to a kurbo BezPath with offset (matching original algorithm)
fn contour_to_bezpath(contour: &crate::core::state::ContourData, offset: Vec2) -> kurbo::BezPath {
    use kurbo::{BezPath, Point};
    
    let mut path = BezPath::new();
    let points = &contour.points;
    
    if points.is_empty() {
        return path;
    }

    // Find segments between on-curve points (matching original algorithm)
    let mut segment_start_idx = 0;
    let mut last_was_on_curve = false;

    // Handle the special case where the first point might be off-curve
    if !is_on_curve_point(&points[0]) {
        // Find the last on-curve point to start with
        let mut last_on_curve_idx = points.len() - 1;
        while last_on_curve_idx > 0 && !is_on_curve_point(&points[last_on_curve_idx]) {
            last_on_curve_idx -= 1;
        }

        if is_on_curve_point(&points[last_on_curve_idx]) {
            segment_start_idx = last_on_curve_idx;
            last_was_on_curve = true;
        }
    } else {
        last_was_on_curve = true;
    }

    let mut path_started = false;

    // Iterate through all points to identify and draw segments (including wrap-around)
    for i in 0..points.len() + 1 {
        let point_idx = i % points.len();
        let is_on = is_on_curve_point(&points[point_idx]);

        if is_on && last_was_on_curve {
            // Two consecutive on-curve points - straight line
            let start_point = &points[segment_start_idx];
            let end_point = &points[point_idx];

            let start_pos = Point::new(
                start_point.x as f64 + offset.x as f64,
                start_point.y as f64 + offset.y as f64,
            );
            let end_pos = Point::new(
                end_point.x as f64 + offset.x as f64,
                end_point.y as f64 + offset.y as f64,
            );

            if !path_started {
                path.move_to(start_pos);
                path_started = true;
            }
            path.line_to(end_pos);

            segment_start_idx = point_idx;
        } else if is_on {
            // Found the end of a curve segment (on-curve point after off-curve points)
            // Collect all points in this segment
            let mut segment_points = Vec::new();
            let mut idx = segment_start_idx;

            // Collect all points from segment_start_idx to point_idx (inclusive)
            loop {
                let pt = &points[idx];
                segment_points.push(Point::new(
                    pt.x as f64 + offset.x as f64,
                    pt.y as f64 + offset.y as f64,
                ));
                idx = (idx + 1) % points.len();
                if idx == (point_idx + 1) % points.len() {
                    break;
                }
            }

            // Draw the appropriate curve based on number of points
            add_curve_segment_to_path(&mut path, &segment_points, &mut path_started);

            // Update for next segment
            segment_start_idx = point_idx;
        }

        last_was_on_curve = is_on;
    }

    // Close the path
    path.close_path();
    path
}

/// Add a curve segment to the kurbo path based on the collected points
fn add_curve_segment_to_path(
    path: &mut kurbo::BezPath,
    segment_points: &[kurbo::Point],
    path_started: &mut bool,
) {
    if segment_points.len() < 2 {
        return;
    }

    if !*path_started {
        path.move_to(segment_points[0]);
        *path_started = true;
    }

    if segment_points.len() == 2 {
        // Simple line segment between two on-curve points
        path.line_to(segment_points[1]);
    } else if segment_points.len() == 4 {
        // Cubic Bezier curve: on-curve, off-curve, off-curve, on-curve
        path.curve_to(segment_points[1], segment_points[2], segment_points[3]);
    } else if segment_points.len() == 3 {
        // Quadratic curve: on-curve, off-curve, on-curve
        path.quad_to(segment_points[1], segment_points[2]);
    } else {
        // For other cases, fallback to line
        path.line_to(segment_points[segment_points.len() - 1]);
    }
}

/// Draw a kurbo BezPath using line segments for rendering
fn draw_bezpath_direct(gizmos: &mut Gizmos, path: &kurbo::BezPath, color: bevy::prelude::Color) {
    let mut prev_point: Option<Vec2> = None;
    
    // Use the newer kurbo flatten API with callback
    path.flatten(0.5, |element| {
        match element {
            kurbo::PathEl::MoveTo(point) => {
                prev_point = Some(Vec2::new(point.x as f32, point.y as f32));
            }
            kurbo::PathEl::LineTo(point) => {
                let current = Vec2::new(point.x as f32, point.y as f32);
                if let Some(prev) = prev_point {
                    gizmos.line_2d(prev, current, color);
                }
                prev_point = Some(current);
            }
            kurbo::PathEl::ClosePath => {
                // Kurbo handles path closing in the flattening process
            }
            _ => {
                // Other elements should already be flattened to lines
            }
        }
    });
}

/// Draw glyph outline directly without viewport transformation
fn draw_glyph_outline_direct(
    gizmos: &mut Gizmos,
    outline: &crate::core::state::OutlineData,
    offset: Vec2,
) {
    for contour in &outline.contours {
        if contour.points.is_empty() {
            continue;
        }
        draw_contour_path_direct(gizmos, contour, offset);
    }
}

/// Draw glyph points directly without viewport transformation
fn draw_glyph_points_direct(
    gizmos: &mut Gizmos,
    outline: &crate::core::state::OutlineData,
    offset: Vec2,
) {
    use crate::ui::theme::{ON_CURVE_POINT_COLOR, OFF_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS, OFF_CURVE_POINT_RADIUS};
    
    for contour in &outline.contours {
        for point in &contour.points {
            let pos = Vec2::new(
                point.x as f32 + offset.x,
                point.y as f32 + offset.y,
            );

            let (color, radius) = if is_on_curve_point(point) {
                (ON_CURVE_POINT_COLOR, ON_CURVE_POINT_RADIUS)
            } else {
                (OFF_CURVE_POINT_COLOR, OFF_CURVE_POINT_RADIUS)
            };

            gizmos.circle_2d(pos, radius, color);
        }
    }
}

/// Check if a point is on-curve


fn is_on_curve_point(point: &crate::core::state::PointData) -> bool {
    matches!(point.point_type, crate::core::state::PointTypeData::Move | 
                               crate::core::state::PointTypeData::Line |
                               crate::core::state::PointTypeData::Curve)
}