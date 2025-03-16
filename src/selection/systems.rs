use super::components::*;
use super::DragSelectionState;
use crate::cameras::DesignCamera;
use crate::data::AppState;
use crate::draw::AppStateChanged;
use crate::edit_type::EditType;
use crate::selection::nudge::{EditEvent, NudgeState};
use crate::theme::{
    HOVER_CIRCLE_RADIUS_MULTIPLIER, HOVER_POINT_COLOR,
    SELECTED_CIRCLE_RADIUS_MULTIPLIER, SELECTED_CROSS_SIZE_MULTIPLIER,
    SELECTED_POINT_COLOR, SELECTION_POINT_RADIUS,
};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Constants for selection
const SELECTION_MARGIN: f32 = 10.0; // Distance in pixels for selection hit testing

/// System to handle mouse input for selection and hovering
pub fn handle_mouse_input(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    mut drag_state: ResMut<DragSelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
    selected_query: Query<Entity, With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    nudge_state: Res<NudgeState>,
    select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Only process mouse input when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    // Early return if no window
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Early return if no camera
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    // If we're in the middle of a nudging operation, don't process mouse input
    // This prevents selection from being cleared during nudging
    if nudge_state.is_nudging {
        return;
    }

    // Update multi-select state based on shift key
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);
    selection_state.multi_select = shift_pressed;

    // Check for mouse click to start selection
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Get cursor position in world coordinates
        if let Some(cursor_pos) = window.cursor_position().and_then(|pos| {
            camera.viewport_to_world_2d(camera_transform, pos).ok()
        }) {
            // Check if we clicked on a selectable entity
            let mut clicked_entity = None;
            let mut closest_distance = SELECTION_MARGIN;

            for (entity, transform) in selectable_query.iter() {
                let entity_pos = transform.translation().truncate();
                let distance = cursor_pos.distance(entity_pos);

                if distance < closest_distance {
                    closest_distance = distance;
                    clicked_entity = Some(entity);
                }
            }

            if let Some(entity) = clicked_entity {
                // Handle entity selection
                if selection_state.multi_select {
                    // Toggle selection with shift key
                    if selection_state.selected.contains(&entity) {
                        selection_state.selected.remove(&entity);
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    // Clear previous selection
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                    }
                    selection_state.selected.clear();

                    // Select the clicked entity
                    selection_state.selected.insert(entity);
                    commands.entity(entity).insert(Selected);
                }

                // Notify about the edit
                event_writer.send(EditEvent {
                    edit_type: EditType::Normal,
                });
            } else {
                // No entity clicked, start drag selection
                drag_state.is_dragging = true;
                drag_state.start_position = Some(cursor_pos);
                drag_state.current_position = Some(cursor_pos);
                drag_state.is_multi_select = selection_state.multi_select;

                // Save previous selection for potential multi-select operations
                drag_state.previous_selection = selected_query.iter().collect();

                // If not multi-selecting, clear previous selection
                if !selection_state.multi_select {
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                    }
                    selection_state.selected.clear();
                }

                // Create selection rectangle entity
                for entity in &selection_rect_query {
                    commands.entity(entity).despawn();
                }

                commands.spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    SelectionRect {
                        start: cursor_pos,
                        end: cursor_pos,
                    },
                    Name::new("Selection Rectangle"),
                ));
            }
        }
    }

    // Update drag selection
    if drag_state.is_dragging {
        if let Some(cursor_pos) = window.cursor_position().and_then(|pos| {
            camera.viewport_to_world_2d(camera_transform, pos).ok()
        }) {
            drag_state.current_position = Some(cursor_pos);

            // Update selection rectangle
            for rect_entity in &selection_rect_query {
                if let Some(start_pos) = drag_state.start_position {
                    commands.entity(rect_entity).insert(SelectionRect {
                        start: start_pos,
                        end: cursor_pos,
                    });
                }
            }

            // Update selection based on what's inside the rectangle
            if let (Some(start_pos), Some(current_pos)) =
                (drag_state.start_position, drag_state.current_position)
            {
                let rect = Rect::from_corners(start_pos, current_pos);

                // In multi-select mode, start with previous selection
                if drag_state.is_multi_select {
                    // Reset to previous selection
                    for entity in &selected_query {
                        if !drag_state.previous_selection.contains(&entity) {
                            commands.entity(entity).remove::<Selected>();
                            selection_state.selected.remove(&entity);
                        }
                    }

                    for entity in &drag_state.previous_selection {
                        if !selection_state.selected.contains(entity) {
                            commands.entity(*entity).insert(Selected);
                            selection_state.selected.insert(*entity);
                        }
                    }
                } else {
                    // Clear selection for non-multi-select
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                    }
                    selection_state.selected.clear();
                }

                // Add entities in the rectangle to selection
                for (entity, transform) in selectable_query.iter() {
                    let entity_pos = transform.translation().truncate();
                    if rect.contains(entity_pos) {
                        if drag_state.is_multi_select
                            && drag_state.previous_selection.contains(&entity)
                        {
                            // Toggle off if previously selected
                            selection_state.selected.remove(&entity);
                            commands.entity(entity).remove::<Selected>();
                        } else {
                            selection_state.selected.insert(entity);
                            commands.entity(entity).insert(Selected);
                        }
                    }
                }
            }
        }
    }

    // End drag selection
    if mouse_button_input.just_released(MouseButton::Left)
        && drag_state.is_dragging
    {
        drag_state.is_dragging = false;
        drag_state.start_position = None;
        drag_state.current_position = None;

        // Remove selection rectangle
        for entity in &selection_rect_query {
            commands.entity(entity).despawn();
        }

        // Notify about the edit
        event_writer.send(EditEvent {
            edit_type: EditType::Normal,
        });
    }
}

/// System to handle keyboard shortcuts for selection
pub fn handle_selection_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut selection_state: ResMut<SelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Only process shortcuts when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    // Handle Escape key to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
    }

    // Handle Ctrl+A to select all
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyA) {
        // Clear current selection
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();

        // Select all selectable entities
        for entity in &selectable_query {
            commands.entity(entity).insert(Selected);
            selection_state.selected.insert(entity);
        }

        event_writer.send(EditEvent {
            edit_type: EditType::Normal,
        });
    }
}

/// System to update hover state based on mouse position
#[allow(dead_code)] // Disabled per user request - hover functionality removed
pub fn update_hover_state(
    mut _commands: Commands,
    _windows: Query<&Window, With<PrimaryWindow>>,
    _camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    _selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
    _hovered_query: Query<Entity, With<Hovered>>,
    _select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Function disabled - hover functionality removed
    return;
}

/// System to draw the selection rectangle
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    selection_rect_query: Query<&SelectionRect>,
    select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Only render the selection rectangle in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for rect in &selection_rect_query {
        let rect_bounds = Rect::from_corners(rect.start, rect.end);

        // Define the orange color to match selected buttons (similar to PRESSED_BUTTON in theme.rs)
        let orange_color = Color::srgb(1.0, 0.6, 0.1);

        // Create dashed lines by drawing multiple small segments
        // Get the corner points
        let min_x = rect_bounds.min.x;
        let min_y = rect_bounds.min.y;
        let max_x = rect_bounds.max.x;
        let max_y = rect_bounds.max.y;

        // Define dash properties
        let dash_length = 10.0;
        let gap_length = 5.0;

        // Draw dashed lines for each side of the rectangle
        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, min_y),
            Vec2::new(max_x, min_y),
            dash_length,
            gap_length,
            orange_color,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, min_y),
            Vec2::new(max_x, max_y),
            dash_length,
            gap_length,
            orange_color,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(max_x, max_y),
            Vec2::new(min_x, max_y),
            dash_length,
            gap_length,
            orange_color,
        );

        draw_dashed_line(
            &mut gizmos,
            Vec2::new(min_x, max_y),
            Vec2::new(min_x, min_y),
            dash_length,
            gap_length,
            orange_color,
        );
    }
}

// Helper function to draw a dashed line between two points
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    dash_length: f32,
    gap_length: f32,
    color: Color,
) {
    let direction = (end - start).normalize();
    let total_length = start.distance(end);

    let segment_length = dash_length + gap_length;
    let num_segments = (total_length / segment_length).ceil() as usize;

    for i in 0..num_segments {
        let segment_start = start + direction * (i as f32 * segment_length);
        let raw_segment_end = segment_start + direction * dash_length;

        // Make sure we don't go past the end point
        let segment_end = if raw_segment_end.distance(start) > total_length {
            end
        } else {
            raw_segment_end
        };

        gizmos.line_2d(segment_start, segment_end, color);
    }
}

/// System to draw visual indicators for selected entities
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<
        (&GlobalTransform, &crate::selection::components::PointType),
        With<Selected>,
    >,
    select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Only render selection indicators in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();

        // Different rendering based on point type
        if point_type.is_on_curve && crate::theme::USE_SQUARE_FOR_ON_CURVE {
            // Draw a square for on-curve points
            let half_size = crate::theme::SELECTION_POINT_RADIUS
                / crate::theme::ON_CURVE_SQUARE_ADJUSTMENT;

            // First draw a filled circle inside the square
            gizmos.circle_2d(
                position,
                half_size * crate::theme::ON_CURVE_INNER_CIRCLE_RATIO,
                crate::theme::SELECTED_POINT_COLOR,
            );

            // Then draw the square outline
            let top_left =
                Vec2::new(position.x - half_size, position.y + half_size);
            let top_right =
                Vec2::new(position.x + half_size, position.y + half_size);
            let bottom_right =
                Vec2::new(position.x + half_size, position.y - half_size);
            let bottom_left =
                Vec2::new(position.x - half_size, position.y - half_size);

            // Draw the square sides
            gizmos.line_2d(
                top_left,
                top_right,
                crate::theme::SELECTED_POINT_COLOR,
            );
            gizmos.line_2d(
                top_right,
                bottom_right,
                crate::theme::SELECTED_POINT_COLOR,
            );
            gizmos.line_2d(
                bottom_right,
                bottom_left,
                crate::theme::SELECTED_POINT_COLOR,
            );
            gizmos.line_2d(
                bottom_left,
                top_left,
                crate::theme::SELECTED_POINT_COLOR,
            );
        } else {
            // Draw a circle for off-curve points
            gizmos.circle_2d(
                position,
                crate::theme::SELECTION_POINT_RADIUS
                    * crate::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER,
                crate::theme::SELECTED_POINT_COLOR,
            );

            // For off-curve points, also draw a smaller inner circle
            if !point_type.is_on_curve {
                gizmos.circle_2d(
                    position,
                    crate::theme::SELECTION_POINT_RADIUS
                        * crate::theme::OFF_CURVE_INNER_CIRCLE_RATIO,
                    crate::theme::SELECTED_POINT_COLOR,
                );
            }
        }

        // Always draw the crosshair for all selected points
        let line_size = crate::theme::SELECTION_POINT_RADIUS
            * crate::theme::SELECTED_CROSS_SIZE_MULTIPLIER;
        gizmos.line_2d(
            Vec2::new(position.x - line_size, position.y),
            Vec2::new(position.x + line_size, position.y),
            crate::theme::SELECTED_POINT_COLOR,
        );
        gizmos.line_2d(
            Vec2::new(position.x, position.y - line_size),
            Vec2::new(position.x, position.y + line_size),
            crate::theme::SELECTED_POINT_COLOR,
        );
    }
}

/// System to draw visual indicators for hovered entities
#[allow(dead_code)] // Disabled per user request - hover functionality removed
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<
        (&GlobalTransform, &crate::selection::components::PointType),
        With<Hovered>,
    >,
    _select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
) {
    // Function disabled - hover functionality removed
    return;
}

#[allow(dead_code)]
pub fn clear_selection_on_app_change(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    mut selection_state: ResMut<SelectionState>,
    mut events: EventReader<AppStateChanged>,
) {
    for _ in events.read() {
        // Clear the selection when app state changes (e.g., when codepoint changes)
        selection_state.selected.clear();

        // Also remove the Selected component from all entities
        for entity in &query {
            commands.entity(entity).remove::<Selected>();
        }

        info!("Selection cleared due to app state change");
    }
}

/// System to update the actual glyph data when a point is nudged
pub fn update_glyph_data_from_selection(
    query: Query<
        (&Transform, &GlyphPointReference),
        (With<Selected>, Changed<Transform>),
    >,
    mut app_state: ResMut<AppState>,
    // Track if we're in a nudging operation
    _nudge_state: Res<crate::selection::nudge::NudgeState>,
) {
    // Early return if no points were nudged
    if query.is_empty() {
        return;
    }

    // Only modify app_state after detaching its change detection
    let app_state = app_state.bypass_change_detection();

    // Process each nudged point
    for (transform, point_ref) in query.iter() {
        // Convert the glyph name from String to GlyphName
        let glyph_name = norad::GlyphName::from(&*point_ref.glyph_name);

        // Try to get the glyph
        if let Some(default_layer) =
            app_state.workspace.font_mut().ufo.get_default_layer_mut()
        {
            if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
                // Get the outline
                if let Some(outline) = glyph.outline.as_mut() {
                    // Make sure the contour index is valid
                    if point_ref.contour_index < outline.contours.len() {
                        let contour =
                            &mut outline.contours[point_ref.contour_index];

                        // Make sure the point index is valid
                        if point_ref.point_index < contour.points.len() {
                            // Update the point position
                            let point =
                                &mut contour.points[point_ref.point_index];
                            // Use direct assignment since both are f32
                            point.x = transform.translation.x;
                            point.y = transform.translation.y;

                            info!(
                                "Updated glyph data for point {} in contour {} of glyph {}",
                                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
                            );
                        }
                    }
                }
            }
        }
    }
}

/// System to handle key releases for arrow keys to maintain selection
pub fn handle_key_releases(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nudge_state: ResMut<NudgeState>,
) {
    // Check if any previously pressed arrow key was released
    if let Some(last_key) = nudge_state.last_key_pressed {
        if keyboard_input.just_released(last_key) {
            // Clear the last pressed key but maintain nudging state
            // This ensures selection isn't lost when arrow keys are released
            nudge_state.last_key_pressed = None;

            // Note: We deliberately don't reset the nudging state here
            // to ensure selection is maintained through multiple nudges
        }
    }
}
