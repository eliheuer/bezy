//! Selection rendering systems
//!
//! This module handles all visual rendering for selection-related features:
//! - Selected point highlighting
//! - Selection marquee/rectangle
//! - Hover effects
//! - Control handle rendering for selected points
//!
//! All actual selection logic (what is selected, hit testing, etc.)
//! remains in the editing/selection module.

use crate::core::state::{AppState, FontIRAppState, TextEditorState};
use crate::editing::selection::components::{
    GlyphPointReference, Hovered, PointType, Selected, SelectionRect,
};
use crate::editing::selection::nudge::NudgeState;
use crate::editing::selection::{DragPointState, DragSelectionState};
use crate::editing::sort::ActiveSort;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;

/// Renders the selection marquee rectangle during drag selection
pub fn render_selection_marquee(
    mut gizmos: Gizmos,
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<&SelectionRect>,
    theme: Res<CurrentTheme>,
) {
    if !drag_state.is_dragging {
        return;
    }

    // Try to find the selection rect from the query first
    if let Some(rect) = marquee_query.iter().next() {
        info!(
            "[render_selection_marquee] Drawing marquee: start={:?}, end={:?}",
            rect.start, rect.end
        );
        let start = rect.start;
        let end = rect.end;
        let color = theme.action_color();

        // Four corners
        let p1 = Vec2::new(start.x, start.y);
        let p2 = Vec2::new(end.x, start.y);
        let p3 = Vec2::new(end.x, end.y);
        let p4 = Vec2::new(start.x, end.y);

        // gizmos, start, end, color, dash_length, gap_length
        draw_dashed_line(&mut gizmos, p1, p2, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p2, p3, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p3, p4, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p4, p1, color, 8.0, 4.0);
    }
}

/// Helper to draw a dashed line between two points
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    let total_length = (end - start).length();
    let direction = (end - start).normalize_or_zero();
    let mut current_length = 0.0;
    let mut draw = true;
    let mut current_position = start;

    while current_length < total_length {
        let segment_length = if draw { dash_length } else { gap_length };
        let next_length = (current_length + segment_length).min(total_length);
        let segment_end_position = start + direction * next_length;

        if draw {
            gizmos.line_2d(current_position, segment_end_position, color);
        }

        current_position = segment_end_position;
        current_length = next_length;
        draw = !draw;
    }
}

/// Renders visual feedback for selected entities
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<(&GlobalTransform, &PointType), With<Selected>>,
    drag_point_state: Res<DragPointState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    nudge_state: Res<NudgeState>,
) {
    // Skip during nudging - live renderer handles everything
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

    let selection_color = SELECTED_POINT_COLOR;
    let is_dragging = drag_point_state.is_dragging;

    // Render each selected point with selection styling
    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        let position_3d = Vec3::new(
            position.x,
            position.y,
            transform.translation().z + SELECTION_Z_DEPTH_OFFSET,
        );
        let position_2d = position_3d.truncate();

        // Draw selection indicator with same shape and size as unselected points
        let point_radius = if point_type.is_on_curve {
            if USE_SQUARE_FOR_ON_CURVE {
                let adjusted_radius =
                    ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT;
                gizmos.rect_2d(
                    position_2d,
                    Vec2::splat(adjusted_radius * 2.0),
                    selection_color,
                );
                adjusted_radius
            } else {
                gizmos.circle_2d(
                    position_2d,
                    ON_CURVE_POINT_RADIUS,
                    selection_color,
                );
                ON_CURVE_POINT_RADIUS
            }
        } else {
            // Off-curve point - always a circle
            gizmos.circle_2d(
                position_2d,
                OFF_CURVE_POINT_RADIUS,
                selection_color,
            );
            OFF_CURVE_POINT_RADIUS
        };

        // Draw cross at the point, sized to match point radius
        let line_size = point_radius;
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

        // Make lines thicker when dragging
        if is_dragging {
            let offset = 0.5;
            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y + offset),
                Vec2::new(position_2d.x + line_size, position_2d.y + offset),
                selection_color,
            );
            gizmos.line_2d(
                Vec2::new(position_2d.x + offset, position_2d.y - line_size),
                Vec2::new(position_2d.x + offset, position_2d.y + line_size),
                selection_color,
            );
        }
    }
}

/// Renders all point entities (not just selected ones)
/// This is used to visualize all points in the active sort for debugging
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_all_point_entities(
    mut gizmos: Gizmos,
    point_entities: Query<
        (Entity, &GlobalTransform, &PointType),
        (With<SortPointEntity>, Without<Selected>),
    >,
    selected_query: Query<Entity, With<Selected>>,
    nudge_state: Res<NudgeState>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<Camera2d>,
    >,
) {
    // Skip during nudging
    if nudge_state.is_nudging {
        return;
    }

    let point_count = point_entities.iter().count();

    // Debug camera information only when we have points to render
    if point_count > 0 {
        if let Ok((_camera, camera_transform, projection)) =
            camera_query.single()
        {
            let camera_pos = camera_transform.translation();
            let camera_scale = match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            };
            debug!(
                "[render_all_point_entities] Camera pos: ({:.1}, {:.1}, {:.1}), scale: {:.3}",
                camera_pos.x, camera_pos.y, camera_pos.z, camera_scale
            );
        }
    }

    for (i, (entity, transform, point_type)) in
        point_entities.iter().enumerate()
    {
        // Skip if selected (already rendered by render_selected_entities)
        if selected_query.get(entity).is_ok() {
            continue;
        }

        let position = transform.translation().truncate();

        if i < 5 {
            info!("[render_all_point_entities] Rendering point {} at ({:.1}, {:.1}), is_on_curve={}", 
                i, position.x, position.y, point_type.is_on_curve);
        }

        // Draw point based on type
        if point_type.is_on_curve {
            if USE_SQUARE_FOR_ON_CURVE {
                let adjusted_radius =
                    ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT;
                gizmos.rect_2d(
                    position,
                    Vec2::splat(adjusted_radius * 2.0),
                    ON_CURVE_POINT_COLOR,
                );
                if ON_CURVE_INNER_CIRCLE_RATIO > 0.0 {
                    let inner_radius =
                        adjusted_radius * ON_CURVE_INNER_CIRCLE_RATIO;
                    gizmos.rect_2d(
                        position,
                        Vec2::splat(inner_radius * 2.0),
                        ON_CURVE_POINT_COLOR,
                    );
                }
            } else {
                gizmos.circle_2d(
                    position,
                    ON_CURVE_POINT_RADIUS,
                    ON_CURVE_POINT_COLOR,
                );
                if ON_CURVE_INNER_CIRCLE_RATIO > 0.0 {
                    let inner_radius =
                        ON_CURVE_POINT_RADIUS * ON_CURVE_INNER_CIRCLE_RATIO;
                    gizmos.circle_2d(
                        position,
                        inner_radius,
                        ON_CURVE_POINT_COLOR,
                    );
                }
            }
        } else {
            // Off-curve point
            gizmos.circle_2d(
                position,
                OFF_CURVE_POINT_RADIUS,
                OFF_CURVE_POINT_COLOR,
            );
            if OFF_CURVE_INNER_CIRCLE_RATIO > 0.0 {
                let inner_radius =
                    OFF_CURVE_POINT_RADIUS * OFF_CURVE_INNER_CIRCLE_RATIO;
                gizmos.circle_2d(position, inner_radius, OFF_CURVE_POINT_COLOR);
            }
        }
    }
}

/// Render control handles between on-curve points and their off-curve control points
pub fn render_control_handles(
    mut gizmos: Gizmos,
    point_entities: Query<
        (&GlobalTransform, &PointType, &GlyphPointReference),
        With<SortPointEntity>,
    >,
    text_editor_state: Res<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    nudge_state: Res<NudgeState>,
) {
    // Skip handle rendering during nudging
    if nudge_state.is_nudging {
        debug!("[render_control_handles] SKIPPING - nudging in progress");
        return;
    }

    // Try FontIR first, then fallback to AppState
    if let Some(fontir_state) = fontir_app_state {
        render_fontir_control_handles(&mut gizmos, &point_entities, &text_editor_state, &fontir_state);
        return;
    }
    
    // Early return if AppState not available
    let Some(app_state) = app_state else {
        debug!("[render_control_handles] Skipping - neither FontIR nor AppState available");
        return;
    };

    // Get the active sort to find the glyph data
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

    // Log summary of what we found
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

    for i in 0..len {
        let (curr_pos, curr_on, _) = contour_points[i];
        let next_idx = (i + 1) % len;
        let (next_pos, next_on, _) = contour_points[next_idx];

        // Draw handle lines between on-curve and off-curve points
        if (curr_on && !next_on) || (!curr_on && next_on) {
            gizmos.line_2d(curr_pos, next_pos, handle_color);
        }
    }
}

/// Renders hovered entities (currently disabled)
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<(&GlobalTransform, &PointType), With<Hovered>>,
) {
    // Hover functionality is disabled per user request
}

/// Render control handles for FontIR-based glyphs
fn render_fontir_control_handles(
    gizmos: &mut Gizmos,
    point_entities: &Query<
        (&GlobalTransform, &PointType, &GlyphPointReference),
        With<SortPointEntity>,
    >,
    text_editor_state: &TextEditorState,
    fontir_state: &FontIRAppState,
) {
    // Get the active sort to find the glyph data
    let Some((_active_sort_index, active_sort)) = text_editor_state.get_active_sort() else {
        return;
    };

    let glyph_name = active_sort.kind.glyph_name();
    
    // Group points by contour
    let mut contours: std::collections::HashMap<usize, Vec<(Vec2, bool, usize)>> = std::collections::HashMap::new();
    
    for (global_transform, point_type, glyph_ref) in point_entities.iter() {
        if glyph_ref.glyph_name == glyph_name {
            let world_pos = global_transform.translation().truncate();
            let is_on_curve = point_type.is_on_curve;
            
            contours
                .entry(glyph_ref.contour_index)
                .or_default()
                .push((world_pos, is_on_curve, glyph_ref.point_index));
        }
    }
    
    // Sort points in each contour by their index
    for (_, contour_points) in contours.iter_mut() {
        contour_points.sort_by_key(|(_, _, idx)| *idx);
    }
    
    // Draw handles for each contour
    let handle_color = HANDLE_LINE_COLOR;
    
    for (_, contour_points) in contours.iter() {
        let len = contour_points.len();
        if len < 2 {
            continue;
        }
        
        // Draw handle lines between consecutive points where one is on-curve and one is off-curve
        for i in 0..len {
            let (curr_pos, curr_on, _) = contour_points[i];
            let next_idx = (i + 1) % len;
            let (next_pos, next_on, _) = contour_points[next_idx];
            
            // Draw handle lines between on-curve and off-curve points
            if (curr_on && !next_on) || (!curr_on && next_on) {
                gizmos.line_2d(curr_pos, next_pos, handle_color);
            }
        }
    }
}
