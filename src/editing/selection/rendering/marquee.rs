//! Marquee selection rectangle rendering

use crate::editing::selection::components::SelectionRect;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

/// System to render the selection rectangle during drag operations
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    selection_rect_query: Query<&SelectionRect>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    let rect_count = selection_rect_query.iter().len();
    debug!("render_selection_rect called: {} rects", rect_count);
    if rect_count > 0 {
        for rect in &selection_rect_query {
            info!("SelectionRect: start={:?}, end={:?}", rect.start, rect.end);
        }
    }
    // Skip rendering the selection rectangle if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only render the selection rectangle in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for rect in &selection_rect_query {
        let rect_bounds = Rect::from_corners(rect.start, rect.end);

        // Define the orange color to match selected buttons
        let orange_color = Color::srgb(1.0, 0.6, 0.1);

        // Get the corner points
        let min_x = rect_bounds.min.x;
        let min_y = rect_bounds.min.y;
        let max_x = rect_bounds.max.x;
        let max_y = rect_bounds.max.y;

        // Draw dashed lines for each side of the rectangle
        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, min_y),
            Vec2::new(max_x, min_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, min_y),
            Vec2::new(max_x, max_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, max_y),
            Vec2::new(min_x, max_y),
            orange_color,
            16.0,
            8.0,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, max_y),
            Vec2::new(min_x, min_y),
            orange_color,
            16.0,
            8.0,
        );
    }
}

/// System to render the selection marquee rectangle using Gizmos
pub fn render_selection_marquee(
    mut gizmos: Gizmos,
    selection_rects: Query<&SelectionRect>,
) {
    let count = selection_rects.iter().count();
    // Only log when there are actually rectangles to render
    if count > 0 {
        debug!(
            "[render_selection_marquee] Called, found {} SelectionRect entities",
            count
        );
    }
    for rect in selection_rects.iter() {
        debug!(
            "[render_selection_marquee] Drawing marquee: start={:?}, end={:?}",
            rect.start, rect.end
        );
        let start = rect.start;
        let end = rect.end;
        let color = Color::srgb(1.0, 0.5, 0.0); // Orange

        // Four corners
        let p1 = Vec2::new(start.x, start.y);
        let p2 = Vec2::new(end.x, start.y);
        let p3 = Vec2::new(end.x, end.y);
        let p4 = Vec2::new(start.x, end.y);

        // Draw dashed lines for each edge
        draw_dashed_line(&mut gizmos, p1, p2, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p2, p3, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p3, p4, color, 16.0, 8.0);
        draw_dashed_line(&mut gizmos, p4, p1, color, 16.0, 8.0);
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
    let mut p = start;

    while current_length < total_length {
        let segment_length = if draw { dash_length } else { gap_length };
        let next_length = (current_length + segment_length).min(total_length);
        let next_p = start + direction * next_length;

        if draw {
            gizmos.line_2d(p, next_p, color);
        }

        p = next_p;
        current_length = next_length;
        draw = !draw;
    }
}
