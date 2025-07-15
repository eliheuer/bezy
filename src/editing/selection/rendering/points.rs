//! Point entity rendering for selection system

use crate::core::state::{AppState, TextEditorState};
use crate::editing::selection::components::{
    GlyphPointReference, Hovered, PointType, Selected,
};
use crate::editing::selection::nudge::NudgeState;
use crate::editing::selection::DragPointState;
use crate::rendering::cameras::DesignCamera;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::{
    HANDLE_LINE_COLOR, OFF_CURVE_INNER_CIRCLE_RATIO, OFF_CURVE_POINT_COLOR,
    OFF_CURVE_POINT_RADIUS, ON_CURVE_INNER_CIRCLE_RATIO, ON_CURVE_POINT_COLOR,
    ON_CURVE_POINT_RADIUS, ON_CURVE_SQUARE_ADJUSTMENT,
    SELECTED_CIRCLE_RADIUS_MULTIPLIER, SELECTED_POINT_COLOR,
    SELECTION_POINT_RADIUS, USE_SQUARE_FOR_ON_CURVE,
};
use bevy::prelude::*;

/// System to render selected entities with visual feedback
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<(&GlobalTransform, &PointType), With<Selected>>,
    drag_point_state: Res<DragPointState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    nudge_state: Res<NudgeState>,
) {
    // CRITICAL: ALWAYS skip this system during nudging
    if nudge_state.is_nudging {
        return;
    }

    let selected_count = selected_query.iter().count();
    if selected_count > 0 {
        debug!("Selection: Rendering {} selected entities", selected_count);
    }

    // Skip rendering if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    let is_dragging = drag_point_state.is_dragging;
    let size_multiplier = if is_dragging { 1.25 } else { 1.0 };

    let selection_color = if is_dragging {
        Color::srgb(1.0, 0.7, 0.2) // Brighter orange during dragging
    } else {
        SELECTED_POINT_COLOR
    };

    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        let position_3d = Vec3::new(
            position.x,
            position.y,
            transform.translation().z + 100.0,
        );
        let position_2d = position_3d.truncate();

        // Different rendering based on point type
        if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            let half_size = SELECTION_POINT_RADIUS / ON_CURVE_SQUARE_ADJUSTMENT
                * size_multiplier;

            gizmos.rect_2d(
                position_2d,
                Vec2::new(half_size * 2.0, half_size * 2.0),
                selection_color,
            );

            gizmos.circle_2d(
                position_2d,
                half_size * ON_CURVE_INNER_CIRCLE_RATIO,
                selection_color,
            );
        } else {
            gizmos.circle_2d(
                position_2d,
                SELECTION_POINT_RADIUS
                    * SELECTED_CIRCLE_RADIUS_MULTIPLIER
                    * size_multiplier,
                selection_color,
            );

            if !point_type.is_on_curve {
                gizmos.circle_2d(
                    position_2d,
                    SELECTION_POINT_RADIUS
                        * OFF_CURVE_INNER_CIRCLE_RATIO
                        * size_multiplier,
                    selection_color,
                );
            }
        }

        // Draw crosshair for all selected points
        let line_size = if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            SELECTION_POINT_RADIUS / ON_CURVE_SQUARE_ADJUSTMENT
        } else {
            SELECTION_POINT_RADIUS * SELECTED_CIRCLE_RADIUS_MULTIPLIER
        };

        let line_size = line_size * size_multiplier;

        gizmos.line_2d(
            Vec2::new(position_2d.x - line_size, position_2d.y),
            Vec2::new(position_2d.x + line_size, position_2d.y),
            selection_color,
        );

        gizmos.line_2d(
            Vec2::new(position_2d.x, position_2d.y - line_size),
            Vec2::new(position_2d.x, position_2d.y + line_size),
            selection_color,
        );

        // Make crosshairs thicker during dragging
        if is_dragging {
            let offset = 0.5;
            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y + offset),
                Vec2::new(position_2d.x + line_size, position_2d.y + offset),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y - offset),
                Vec2::new(position_2d.x + line_size, position_2d.y - offset),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x + offset, position_2d.y - line_size),
                Vec2::new(position_2d.x + offset, position_2d.y + line_size),
                selection_color,
            );

            gizmos.line_2d(
                Vec2::new(position_2d.x - offset, position_2d.y - line_size),
                Vec2::new(position_2d.x - offset, position_2d.y + line_size),
                selection_color,
            );
        }
    }
}

/// System to render all point entities (not just selected ones)
pub fn render_all_point_entities(
    mut gizmos: Gizmos,
    point_entities: Query<
        (Entity, &GlobalTransform, &PointType),
        With<SortPointEntity>,
    >,
    // Add a separate query to count all SortPointEntity components
    all_sort_points: Query<Entity, With<SortPointEntity>>,
    selected_query: Query<Entity, With<Selected>>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<DesignCamera>,
    >,
    nudge_state: Res<NudgeState>,
) {
    // Skip during nudging
    if nudge_state.is_nudging {
        return;
    }

    let point_count = point_entities.iter().count();
    let all_sort_point_count = all_sort_points.iter().count();
    
    // Only log when there are actually point entities to render, or when there's a mismatch
    if point_count > 0 || point_count != all_sort_point_count {
        debug!(
            "[render_all_point_entities] Called, found {} point entities, {} total SortPointEntity components",
            point_count, all_sort_point_count
        );
        
        if point_count != all_sort_point_count {
            warn!("Component mismatch: {} points have full components, but {} have SortPointEntity", 
                  point_count, all_sort_point_count);
        }
    }

    // Debug camera information only when we have points to render
    if point_count > 0 {
        if let Ok((_camera, camera_transform, projection)) = camera_query.single() {
            let camera_pos = camera_transform.translation();
            let camera_scale = match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            };
            debug!("[render_all_point_entities] Camera: pos=({:.1}, {:.1}, {:.1}), scale={:.3}", 
                  camera_pos.x, camera_pos.y, camera_pos.z, camera_scale);
        } else {
            warn!("[render_all_point_entities] No camera found");
        }
    }

    for (i, (entity, transform, point_type)) in
        point_entities.iter().enumerate()
    {
        // Skip if selected (rendered by render_selected_entities)
        if selected_query.get(entity).is_ok() {
            continue;
        }

        let position = transform.translation().truncate();

        if i < 5 {
            info!("[render_all_point_entities] Rendering point {} at ({:.1}, {:.1}), is_on_curve={}", 
                  i, position.x, position.y, point_type.is_on_curve);
        }

        let (size, color) = if point_type.is_on_curve {
            (ON_CURVE_POINT_RADIUS, ON_CURVE_POINT_COLOR)
        } else {
            (OFF_CURVE_POINT_RADIUS, OFF_CURVE_POINT_COLOR)
        };

        if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            let half_size = size / ON_CURVE_SQUARE_ADJUSTMENT;
            let square_size = Vec2::new(size * 2.0, size * 2.0);
            gizmos.rect_2d(position, square_size, color);

            let inner_radius = half_size * ON_CURVE_INNER_CIRCLE_RATIO;
            gizmos.circle_2d(position, inner_radius, color);
        } else {
            gizmos.circle_2d(position, size, color);

            let inner_radius = size * OFF_CURVE_INNER_CIRCLE_RATIO;
            gizmos.circle_2d(position, inner_radius, color);
        }
    }
}

/// System to render hovered entities (disabled for now)
#[allow(dead_code)]
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<(&GlobalTransform, &PointType), With<Hovered>>,
) {
    // Hover functionality is disabled per user request
}

/// Render control handles between on-curve points and their off-curve control points
pub fn render_control_handles(
    mut gizmos: Gizmos,
    point_entities: Query<
        (&GlobalTransform, &PointType, &GlyphPointReference),
        With<SortPointEntity>,
    >,
    text_editor_state: Res<TextEditorState>,
    app_state: Res<AppState>,
    nudge_state: Res<NudgeState>,
) {
    // Skip handle rendering during nudging
    if nudge_state.is_nudging {
        debug!("[render_control_handles] SKIPPING - nudging in progress");
        return;
    }

    let Some((_active_sort_index, active_sort)) =
        text_editor_state.get_active_sort()
    else {
        return;
    };

    let glyph_name = active_sort.kind.glyph_name();
    let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name)
    else {
        return;
    };

    let Some(outline) = &glyph_data.outline else {
        return;
    };

    // Group points by contour
    let mut contour_points: Vec<Vec<(Vec2, bool, usize)>> =
        vec![Vec::new(); outline.contours.len()];

    let mut total_points = 0;
    let mut unique_positions = std::collections::HashSet::new();

    for (transform, point_type, glyph_ref) in point_entities.iter() {
        let position = transform.translation().truncate();
        let is_on_curve = point_type.is_on_curve;
        let contour_index = glyph_ref.contour_index;
        let point_index = glyph_ref.point_index;

        total_points += 1;
        unique_positions.insert((position.x.to_bits(), position.y.to_bits()));

        if total_points <= 5 {
            debug!(
                "[render_control_handles] Point {}: pos=({:.1}, {:.1}), contour={}, index={}, on_curve={}",
                total_points, position.x, position.y, contour_index, point_index, is_on_curve
            );
        }
        
        // Check if this position is exactly zero - this indicates a transform propagation issue
        if position.x == 0.0 && position.y == 0.0 {
            warn!(
                "[render_control_handles] Point {} at ZERO position - GlobalTransform propagation issue!",
                total_points
            );
        }

        if contour_index < contour_points.len() {
            contour_points[contour_index].push((
                position,
                is_on_curve,
                point_index,
            ));
        }
    }

    if total_points > 0 {
        debug!(
            "[render_control_handles] Total points: {}, Unique positions: {}",
            total_points,
            unique_positions.len()
        );
        if unique_positions.len() == 1 && total_points > 1 {
            warn!(
                "[render_control_handles] ALL {} POINTS ARE AT THE SAME POSITION!",
                total_points
            );
        }
    }

    // Sort points within each contour by original index
    for contour_points in &mut contour_points {
        contour_points.sort_by_key(|(_, _, index)| *index);
    }

    // Render handles for each contour
    for contour_points in contour_points {
        if contour_points.len() < 2 {
            continue;
        }
        render_contour_handles(&mut gizmos, &contour_points);
    }
}

/// Render handles for a single contour
fn render_contour_handles(
    gizmos: &mut Gizmos,
    contour_points: &[(Vec2, bool, usize)],
) {
    let handle_color = HANDLE_LINE_COLOR;
    let len = contour_points.len();
    if len < 2 {
        return;
    }

    let mut handle_count = 0;
    for i in 0..len {
        let (curr_pos, curr_on, curr_idx) = contour_points[i];
        let next_idx = (i + 1) % len;
        let (next_pos, next_on, next_orig_idx) = contour_points[next_idx];

        // Draw handle lines between on-curve and off-curve points
        if (curr_on && !next_on) || (!curr_on && next_on) {
            handle_count += 1;
            if handle_count <= 3 {
                debug!(
                    "[render_contour_handles] Handle {}: from ({:.1}, {:.1})[idx={}] to ({:.1}, {:.1})[idx={}], curr_on={}, next_on={}",
                    handle_count, curr_pos.x, curr_pos.y, curr_idx, next_pos.x, next_pos.y, next_orig_idx, curr_on, next_on
                );
            }
            gizmos.line_2d(curr_pos, next_pos, handle_color);
        }
    }
    
    if handle_count > 0 {
        debug!("[render_contour_handles] Drew {} handle lines for contour with {} points", handle_count, len);
    }
}
