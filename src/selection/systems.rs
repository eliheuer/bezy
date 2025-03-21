use super::components::*;
use super::DragSelectionState;
use crate::cameras::DesignCamera;
use crate::data::AppState;
use crate::draw::AppStateChanged;
use crate::edit_type::EditType;
use crate::selection::nudge::{EditEvent, NudgeState};
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
    selectable_query: Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>), With<Selectable>>,
    selected_query: Query<Entity, With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    nudge_state: Res<NudgeState>,
    select_mode: Option<
        Res<crate::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    ui_hover_state: Res<crate::ui_interaction::UiHoverState>,
) {
    // Log at the beginning of each frame
    info!("Selection system running - current selected entities: {}", selection_state.selected.len());
    
    // Skip if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            info!("Selection skipped - knife mode active");
            return;
        }
    }

    // Only process when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            info!("Selection skipped - select mode not active");
            return;
        }
    }

    // Don't process selection when hovering over UI
    if ui_hover_state.is_hovering_ui {
        info!("Selection skipped - hovering over UI");
        return;
    }

    // Early return if no window
    let Ok(window) = windows.get_single() else {
        info!("Selection skipped - no window");
        return;
    };

    // Early return if no camera
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        info!("Selection skipped - no camera");
        return;
    };

    // If we're in the middle of a nudging operation, don't process mouse input
    // This prevents selection from being cleared during nudging
    if nudge_state.is_nudging {
        info!("Selection skipped - nudging in progress");
        return;
    }

    // Update multi-select state based on shift key
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);
    selection_state.multi_select = shift_pressed;

    // Check for mouse click to start selection
    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("Mouse button pressed - checking for selection");
        
        // Get cursor position in world coordinates
        if let Some(cursor_pos) = window.cursor_position().and_then(|pos| {
            camera.viewport_to_world_2d(camera_transform, pos).ok()
        }) {
            info!("Cursor position in world: ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);
            
            // Check if we clicked on a selectable entity
            let mut clicked_entity = None;
            let mut closest_distance = SELECTION_MARGIN;
            let mut debug_distances = Vec::new();

            for (entity, transform, point_ref) in selectable_query.iter() {
                let entity_pos = transform.translation().truncate();
                let distance = cursor_pos.distance(entity_pos);
                
                debug_distances.push((entity, entity_pos, distance));

                if distance < closest_distance {
                    closest_distance = distance;
                    clicked_entity = Some((entity, point_ref));
                }
            }
            
            // Log all close points for debugging
            for (entity, pos, dist) in debug_distances.iter().filter(|(_, _, d)| *d < SELECTION_MARGIN * 2.0) {
                info!("  Point entity {:?} at ({:.1}, {:.1}) distance: {:.2}", 
                      entity, pos.x, pos.y, dist);
            }

            if let Some((entity, point_ref)) = clicked_entity {
                info!("Entity clicked: {:?} distance: {:.2}", entity, closest_distance);
                
                if let Some(glyph_ref) = point_ref {
                    info!("  Glyph point: {} contour: {} point: {}", 
                          glyph_ref.glyph_name, glyph_ref.contour_index, glyph_ref.point_index);
                }
                
                // Handle entity selection
                if selection_state.multi_select {
                    // Toggle selection with shift key
                    if selection_state.selected.contains(&entity) {
                        info!("  Deselecting entity (multi-select) {:?}", entity);
                        selection_state.selected.remove(&entity);
                        commands.entity(entity).remove::<Selected>();
                        info!("    -> Command to remove Selected component from entity {:?} queued", entity);
                    } else {
                        info!("  Adding entity to selection (multi-select) {:?}", entity);
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                        info!("    -> Command to add Selected component to entity {:?} queued", entity);
                    }
                } else {
                    // Clear previous selection
                    info!("  Clearing previous selection of {} entities", selected_query.iter().count());
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                        info!("    -> Command to remove Selected component from entity {:?} queued", entity);
                    }
                    selection_state.selected.clear();

                    // Select the clicked entity
                    info!("  Selecting new entity {:?}", entity);
                    selection_state.selected.insert(entity);
                    commands.entity(entity).insert(Selected);
                    info!("    -> Command to add Selected component to entity {:?} queued", entity);
                }

                // Notify about the edit
                event_writer.send(EditEvent {
                    edit_type: EditType::Normal,
                });
                
                info!("Selection updated. Current selection count: {}", selection_state.selected.len());
            } else {
                info!("No entity clicked, starting drag selection");
                // No entity clicked, start drag selection
                drag_state.is_dragging = true;
                drag_state.start_position = Some(cursor_pos);
                drag_state.current_position = Some(cursor_pos);
                drag_state.is_multi_select = selection_state.multi_select;

                // Save previous selection for potential multi-select operations
                drag_state.previous_selection = selected_query.iter().collect();

                // If not multi-selecting, clear previous selection
                if !selection_state.multi_select {
                    info!("  Clearing previous selection for drag operation");
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                        info!("    -> Command to remove Selected component from entity {:?} queued", entity);
                    }
                    selection_state.selected.clear();
                }

                // Clean up any existing selection rectangle entities
                for entity in &selection_rect_query {
                    commands.entity(entity).despawn_recursive();
                }

                // Create a fresh selection rectangle entity
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

            // Update selection rectangle - only if we have a rectangle entity
            if !selection_rect_query.is_empty() {
                for rect_entity in &selection_rect_query {
                    if let Some(start_pos) = drag_state.start_position {
                        commands.entity(rect_entity).insert(SelectionRect {
                            start: start_pos,
                            end: cursor_pos,
                        });
                    }
                }
            } else {
                // Create selection rectangle entity if it doesn't exist
                if let Some(start_pos) = drag_state.start_position {
                    commands.spawn((
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        SelectionRect {
                            start: start_pos,
                            end: cursor_pos,
                        },
                        Name::new("Selection Rectangle"),
                    ));
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
                            info!("  -> Command to remove Selected component from entity {:?} queued (drag)", entity);
                        }
                    }

                    for entity in &drag_state.previous_selection {
                        if !selection_state.selected.contains(entity) {
                            commands.entity(*entity).insert(Selected);
                            selection_state.selected.insert(*entity);
                            info!("  -> Command to add Selected component to entity {:?} queued (drag restore)", entity);
                        }
                    }
                } else {
                    // Clear selection for non-multi-select
                    for entity in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                        info!("  -> Command to remove Selected component from entity {:?} queued (drag clear)", entity);
                    }
                    selection_state.selected.clear();
                }

                // Add entities in the rectangle to selection
                for (entity, transform, _) in selectable_query.iter() {
                    let entity_pos = transform.translation().truncate();
                    if rect.contains(entity_pos) {
                        if drag_state.is_multi_select
                            && drag_state.previous_selection.contains(&entity)
                        {
                            // Toggle off if previously selected
                            selection_state.selected.remove(&entity);
                            commands.entity(entity).remove::<Selected>();
                            info!("  -> Command to remove Selected component from entity {:?} queued (drag toggle)", entity);
                        } else {
                            selection_state.selected.insert(entity);
                            commands.entity(entity).insert(Selected);
                            info!("  -> Command to add Selected component to entity {:?} queued (drag select)", entity);
                        }
                    }
                }
            }
        }
    }

    // Handle mouse button release to complete selection
    if mouse_button_input.just_released(MouseButton::Left)
        && drag_state.is_dragging
    {
        drag_state.is_dragging = false;
        drag_state.start_position = None;
        drag_state.current_position = None;

        // Clean up the selection rectangle
        for entity in &selection_rect_query {
            commands.entity(entity).despawn_recursive();
        }

        // Notify about the edit if we made a selection
        if !selection_state.selected.is_empty() {
            event_writer.send(EditEvent {
                edit_type: EditType::Normal,
            });
            info!("Drag selection completed with {} entities selected", selection_state.selected.len());
        }
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
    knife_mode: Option<
        Res<crate::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    // Skip processing shortcuts if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

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
    knife_mode: Option<
        Res<crate::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
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
    knife_mode: Option<
        Res<crate::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    // Skip rendering selection indicators if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only render selection indicators in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        // Use a position with a slight Z offset to ensure it renders on top
        let position_3d =
            Vec3::new(position.x, position.y, transform.translation().z + 5.0);
        let position_2d = position_3d.truncate();

        // Different rendering based on point type
        if point_type.is_on_curve && crate::theme::USE_SQUARE_FOR_ON_CURVE {
            // Draw a square for on-curve points
            let half_size = crate::theme::SELECTION_POINT_RADIUS
                / crate::theme::ON_CURVE_SQUARE_ADJUSTMENT;

            // First draw a filled circle inside the square
            gizmos.circle_2d(
                position_2d,
                half_size * crate::theme::ON_CURVE_INNER_CIRCLE_RATIO,
                crate::theme::SELECTED_POINT_COLOR,
            );

            // Then draw the square outline
            let top_left =
                Vec2::new(position_2d.x - half_size, position_2d.y + half_size);
            let top_right =
                Vec2::new(position_2d.x + half_size, position_2d.y + half_size);
            let bottom_right =
                Vec2::new(position_2d.x + half_size, position_2d.y - half_size);
            let bottom_left =
                Vec2::new(position_2d.x - half_size, position_2d.y - half_size);

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
                position_2d,
                crate::theme::SELECTION_POINT_RADIUS
                    * crate::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER,
                crate::theme::SELECTED_POINT_COLOR,
            );

            // For off-curve points, also draw a smaller inner circle
            if !point_type.is_on_curve {
                gizmos.circle_2d(
                    position_2d,
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
            Vec2::new(position_2d.x - line_size, position_2d.y),
            Vec2::new(position_2d.x + line_size, position_2d.y),
            crate::theme::SELECTED_POINT_COLOR,
        );
        gizmos.line_2d(
            Vec2::new(position_2d.x, position_2d.y - line_size),
            Vec2::new(position_2d.x, position_2d.y + line_size),
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
    knife_mode: Option<
        Res<crate::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
) {
    // Skip processing if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Early return if no points were nudged
    if query.is_empty() {
        return;
    }

    // Only modify app_state after detaching its change detection
    let app_state = app_state.bypass_change_detection();

    // Process each nudged point
    for (transform, point_ref) in query.iter() {
        // Find the current glyph name first (before mutable borrow)
        let current_glyph = match app_state.workspace.selected.clone() {
            Some(glyph_name) => glyph_name,
            None => return, // No selected glyph
        };

        // Now get the font object with a mutable borrow
        let font_obj = app_state.workspace.font_mut();
        let Some(default_layer) = font_obj.ufo.get_default_layer_mut() else {
            return;
        };
        
        let Some(glyph) = default_layer.get_glyph_mut(&current_glyph) else {
            return;
        };

        let Some(outline) = glyph.outline.as_mut() else {
            return;
        };

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

/// System to handle Ctrl+click on line segments to upgrade them to curves
pub fn handle_line_segment_upgrade(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    ui_hover_state: Res<crate::ui_interaction::UiHoverState>,
    select_mode: Option<Res<crate::edit_mode_toolbar::select::SelectModeActive>>,
) {
    // Check if we're in Select mode
    let Some(select_mode) = select_mode else {
        // If Select mode resource doesn't exist, we can't be in select mode
        return;
    };
    
    if !select_mode.0 {
        // Not in select mode
        return;
    }

    // Don't process when hovering over UI
    if ui_hover_state.is_hovering_ui {
        info!("Line segment upgrade: Hovering over UI");
        return;
    }

    // Check for left mouse button click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Check if Ctrl key is pressed (either left or right)
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft) 
                    || keyboard_input.pressed(KeyCode::ControlRight)
                    || keyboard_input.pressed(KeyCode::SuperLeft) // For Mac
                    || keyboard_input.pressed(KeyCode::SuperRight); // For Mac

    if !ctrl_pressed {
        info!("Line segment upgrade: Left click without Ctrl key");
        return;
    }

    info!("Line segment upgrade: Ctrl+click detected");

    // Get cursor position in world coordinates
    let Ok(window) = windows.get_single() else {
        info!("Line segment upgrade: No window found");
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        info!("Line segment upgrade: No camera found");
        return;
    };

    let Some(cursor_pos) = window.cursor_position().and_then(|pos| {
        camera.viewport_to_world_2d(camera_transform, pos).ok()
    }) else {
        info!("Line segment upgrade: Could not get cursor position in world space");
        return;
    };

    info!("Line segment upgrade: Cursor position in world: ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);

    // Get the current glyph name first (before mutable borrow)
    let current_glyph = match app_state.workspace.selected.clone() {
        Some(glyph_name) => {
            info!("Line segment upgrade: Current glyph: {}", glyph_name);
            glyph_name
        }
        None => {
            info!("Line segment upgrade: No glyph selected. Please select a glyph first.");
            return;
        }
    };

    // Log the available glyphs for debugging
    {
        let font_obj = &app_state.workspace.font;
        if let Some(default_layer) = font_obj.ufo.get_default_layer() {
            // There's no direct way to iterate all glyphs in norad 0.3.1
            // Check common Latin glyphs to see what's available
            let mut glyph_names = Vec::new();
            
            // Check for A-Z and a-z glyphs
            for cp in 0x0041..=0x007A {
                if cp > 0x005A && cp < 0x0061 {
                    continue; // Skip between Z and a
                }
                let char_name = char::from_u32(cp).unwrap().to_string();
                let glyph_name = norad::GlyphName::from(char_name);
                if default_layer.get_glyph(&glyph_name).is_some() {
                    glyph_names.push(glyph_name.to_string());
                }
            }
            
            info!("Line segment upgrade: Found {} glyphs (sample): {:?}", 
                  glyph_names.len(),
                  glyph_names.iter().take(10).collect::<Vec<_>>());
        } else {
            info!("Line segment upgrade: No default layer found in font");
        }
    }

    // Now get the font object with a mutable borrow
    let font_obj = app_state.workspace.font_mut();
    let Some(default_layer) = font_obj.ufo.get_default_layer_mut() else {
        info!("Line segment upgrade: Could not get default layer");
        return;
    };
    
    let Some(glyph) = default_layer.get_glyph_mut(&current_glyph) else {
        info!("Line segment upgrade: Could not find glyph '{}' in the default layer", current_glyph);
        return;
    };

    let Some(outline) = glyph.outline.as_mut() else {
        info!("Line segment upgrade: Glyph '{}' has no outline", current_glyph);
        return;
    };

    if outline.contours.is_empty() {
        info!("Line segment upgrade: Glyph '{}' has no contours", current_glyph);
        return;
    }

    info!("Line segment upgrade: Found {} contours in the glyph '{}'", outline.contours.len(), current_glyph);

    // Check each contour for line segments that might be close to the cursor
    for (contour_idx, contour) in outline.contours.iter_mut().enumerate() {
        // Get a copy of the points for scanning
        let points_copy = contour.points.clone();
        
        // We need at least 2 points to form a line segment
        if points_copy.len() < 2 {
            info!("Line segment upgrade: Contour {} has fewer than 2 points", contour_idx);
            continue;
        }

        info!("Line segment upgrade: Checking contour {} with {} points", contour_idx, points_copy.len());

        // Log the point types in this contour for debugging
        for (i, point) in points_copy.iter().enumerate() {
            info!("Line segment upgrade: Point {}: type={:?}, coords=({:.1}, {:.1})", 
                 i, point.typ, point.x, point.y);
        }

        // Check each potential line segment
        let num_points = points_copy.len();
        for i in 0..num_points {
            let current_idx = i;
            let next_idx = (i + 1) % num_points;
            
            let current_point = &points_copy[current_idx];
            let next_point = &points_copy[next_idx];
            
            // Simplified check: we just need two consecutive on-curve points
            let current_is_on_curve = current_point.typ != norad::PointType::OffCurve;
            let next_is_on_curve = next_point.typ != norad::PointType::OffCurve;
            
            // Check if points between current and next are all off-curve
            let mut has_off_curve_between = false;
            if current_is_on_curve && next_is_on_curve {
                // Check if there are any control points between these on-curve points
                let mut idx = (current_idx + 1) % num_points;
                while idx != next_idx {
                    if points_copy[idx].typ != norad::PointType::OffCurve {
                        has_off_curve_between = true;
                        break;
                    }
                    idx = (idx + 1) % num_points;
                }
            }
            
            // Skip this segment if it doesn't look like a straight line
            if !current_is_on_curve || !next_is_on_curve || has_off_curve_between {
                info!("Line segment upgrade: Points {}-{} don't form a simple line segment", current_idx, next_idx);
                continue;
            }
            
            info!("Line segment upgrade: Found potential line segment from point {} to {}", current_idx, next_idx);
            
            // Convert points to Vec2 for distance calculation
            let p0 = Vec2::new(current_point.x, current_point.y);
            let p3 = Vec2::new(next_point.x, next_point.y);
            
            info!("Line segment upgrade: Line segment from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                  p0.x, p0.y, p3.x, p3.y);
            
            // Calculate the distance from cursor to line segment
            let cursor_v2 = Vec2::new(cursor_pos.x, cursor_pos.y);
            let line_vec = p3 - p0;
            
            // Skip if line has zero length
            if line_vec.length_squared() < 0.001 {
                info!("Line segment upgrade: Line segment too short");
                continue;
            }
            
            let line_dir = line_vec.normalize();
            let cursor_vec = cursor_v2 - p0;
            
            // Project cursor onto line
            let projection = cursor_vec.dot(line_dir);
            let closest_point_on_line = if projection <= 0.0 {
                p0
            } else if projection >= line_vec.length() {
                p3
            } else {
                p0 + line_dir * projection
            };
            
            let distance = cursor_v2.distance(closest_point_on_line);
            
            info!("Line segment upgrade: Distance to line: {:.2}, projection: {:.2}, line length: {:.2}", 
                  distance, projection, line_vec.length());
            
            // Increase the distance threshold to make it easier to hit
            // If cursor is close to the line segment, upgrade it to a curve
            if distance <= 15.0 && projection > 0.0 && projection < line_vec.length() {
                info!("Line segment upgrade: Hit detected! Converting line to curve");
                
                // Create two off-curve points at 1/3 and 2/3 positions
                let p1 = p0.lerp(p3, 1.0 / 3.0);
                let p2 = p0.lerp(p3, 2.0 / 3.0);
                
                info!("Line segment upgrade: Adding off-curve points at ({:.1}, {:.1}) and ({:.1}, {:.1})",
                      p1.x, p1.y, p2.x, p2.y);
                
                // Create new off-curve points
                let off_curve1 = norad::ContourPoint::new(
                    p1.x, 
                    p1.y, 
                    norad::PointType::OffCurve, 
                    false, // not smooth
                    None,  // no name
                    None,  // no identifier
                    None,  // no comments
                );
                
                let off_curve2 = norad::ContourPoint::new(
                    p2.x, 
                    p2.y, 
                    norad::PointType::OffCurve, 
                    false, // not smooth
                    None,  // no name
                    None,  // no identifier
                    None,  // no comments
                );
                
                // Make a copy of the points array
                let mut new_points = points_copy.clone();
                
                // Always change the end point to be a curve point
                // We're ensuring it's explicitly a Curve type, regardless of its previous type
                new_points[next_idx].typ = norad::PointType::Curve;
                
                // Insert the off-curve points correctly
                if next_idx > current_idx {
                    // Simple case: insert between current_idx and next_idx
                    info!("Line segment upgrade: Simple insertion - adding 2 off-curve points after index {}", current_idx);
                    new_points.insert(current_idx + 1, off_curve1);
                    new_points.insert(current_idx + 2, off_curve2);
                } else {
                    // Wrap-around case (end of contour connects to beginning)
                    // In this case, we append to the end of the contour
                    info!("Line segment upgrade: Wrap-around insertion - adding 2 off-curve points after the last point");
                    new_points.push(off_curve1);
                    new_points.push(off_curve2);
                }
                
                // Create a new contour with the updated points
                let new_contour = norad::Contour::new(
                    new_points.clone(), // Clone to avoid borrowing issues
                    contour.identifier().cloned(),
                    None, // No new lib needed
                );
                
                // Replace the old contour with the new one
                *contour = new_contour;
                
                // Notify that the app state has changed
                app_state_changed.send(AppStateChanged);
                
                // Print detailed debug info about the conversion with values copied before
                info!("Line segment upgrade: Start point idx={}, type={:?}", current_idx, current_point.typ);
                info!("Line segment upgrade: End point type transformed to {:?}", norad::PointType::Curve);
                
                // Log the contour points after modification for debugging
                info!("Line segment upgrade: New contour has {} points:", new_points.len());
                for (i, point) in new_points.iter().enumerate() {
                    info!("  Point {}: type={:?}, coords=({:.1}, {:.1})", 
                         i, point.typ, point.x, point.y);
                }
                
                info!("Line segment upgrade: Successfully upgraded line segment to curve in glyph {}", current_glyph);
                return; // Process only one line segment upgrade per click
            } else {
                info!("Line segment upgrade: Click not close enough to line segment");
            }
        }
    }
    
    info!("Line segment upgrade: No suitable line segment found for upgrade");
}

/// System to handle Command+U shortcut to upgrade selected line segments to curves
pub fn handle_line_segment_upgrade_shortcut(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<&GlyphPointReference, With<Selected>>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    select_mode: Option<Res<crate::edit_mode_toolbar::select::SelectModeActive>>,
) {
    // Check if we're in Select mode
    let Some(select_mode) = select_mode else {
        return;
    };
    
    if !select_mode.0 {
        return;
    }

    // Check for Command+U shortcut
    let command_pressed = keyboard_input.pressed(KeyCode::SuperLeft) 
                       || keyboard_input.pressed(KeyCode::SuperRight)
                       || keyboard_input.pressed(KeyCode::ControlLeft)  // For Windows/Linux
                       || keyboard_input.pressed(KeyCode::ControlRight);
    
    if !command_pressed || !keyboard_input.just_pressed(KeyCode::KeyU) {
        return;
    }

    info!("Line segment upgrade shortcut: Command+U detected");

    // If no points are selected, do nothing
    if selected_query.is_empty() {
        info!("Line segment upgrade shortcut: No points selected");
        return;
    }

    // Group selected points by glyph and contour
    let mut selected_points: Vec<(String, usize, usize)> = Vec::new();
    for point_ref in selected_query.iter() {
        selected_points.push((
            point_ref.glyph_name.clone(),
            point_ref.contour_index,
            point_ref.point_index,
        ));
    }
    
    // Sort to ensure points are in correct order
    selected_points.sort_by(|a, b| {
        a.0.cmp(&b.0) // First by glyph
            .then(a.1.cmp(&b.1)) // Then by contour
            .then(a.2.cmp(&b.2)) // Then by point index
    });
    
    info!("Line segment upgrade shortcut: Found {} selected points", selected_points.len());

    // Group points by glyph and contour
    let mut points_by_contour: std::collections::HashMap<(String, usize), Vec<usize>> = 
        std::collections::HashMap::new();
    
    for (glyph, contour, point) in selected_points {
        points_by_contour
            .entry((glyph, contour))
            .or_insert_with(Vec::new)
            .push(point);
    }
    
    let mut modifications_made = false;

    // Process each glyph and contour with selected points
    for ((glyph_name, contour_idx), selected_indices) in points_by_contour.iter() {
        // Get the current glyph
        let current_glyph = norad::GlyphName::from(glyph_name.as_str());
        
        // Get the font object with a mutable borrow
        let font_obj = app_state.workspace.font_mut();
        let Some(default_layer) = font_obj.ufo.get_default_layer_mut() else {
            info!("Line segment upgrade shortcut: Could not get default layer");
            continue;
        };
        
        let Some(glyph) = default_layer.get_glyph_mut(&current_glyph) else {
            info!("Line segment upgrade shortcut: Could not find glyph '{}'", current_glyph);
            continue;
        };

        let Some(outline) = glyph.outline.as_mut() else {
            info!("Line segment upgrade shortcut: Glyph '{}' has no outline", current_glyph);
            continue;
        };

        if *contour_idx >= outline.contours.len() {
            info!("Line segment upgrade shortcut: Invalid contour index {}", contour_idx);
            continue;
        }

        let contour = &mut outline.contours[*contour_idx];
        let num_points = contour.points.len();
        
        if num_points < 2 {
            info!("Line segment upgrade shortcut: Contour has fewer than 2 points");
            continue;
        }
        
        // Find adjacent selected on-curve points
        let mut adjacent_pairs = Vec::new();
        
        for i in 0..selected_indices.len() {
            let current_idx = selected_indices[i];
            if current_idx >= num_points {
                continue; // Skip invalid index
            }
            
            let current_point = &contour.points[current_idx];
            if current_point.typ == norad::PointType::OffCurve {
                continue; // Skip off-curve points
            }
            
            // Look for adjacent on-curve points by searching the selection
            for j in 0..selected_indices.len() {
                if i == j {
                    continue; // Skip self
                }
                
                let next_idx = selected_indices[j];
                if next_idx >= num_points {
                    continue; // Skip invalid index
                }
                
                let next_point = &contour.points[next_idx];
                if next_point.typ == norad::PointType::OffCurve {
                    continue; // Skip off-curve points
                }
                
                // Check if points are adjacent in the contour
                let is_adjacent = if current_idx < next_idx {
                    // Check if there are only on-curve points between them
                    let mut has_on_curve_between = false;
                    for k in (current_idx + 1)..next_idx {
                        if contour.points[k].typ != norad::PointType::OffCurve {
                            has_on_curve_between = true;
                            break;
                        }
                    }
                    !has_on_curve_between && (next_idx - current_idx <= 2)
                } else {
                    // Wrap-around case or reverse case
                    // For simplicity, skip these cases for now
                    false
                };
                
                if is_adjacent {
                    // Check if this is a straight line with no off-curves between
                    let is_straight_line = next_idx - current_idx == 1;
                    
                    if is_straight_line {
                        adjacent_pairs.push((current_idx, next_idx));
                    }
                }
            }
        }
        
        info!("Line segment upgrade shortcut: Found {} adjacent pairs in contour {}", adjacent_pairs.len(), contour_idx);
        
        // Now process each adjacent pair of on-curve points to upgrade them to curves
        // Need to process in reverse order to avoid messing up indices
        adjacent_pairs.sort_by(|a, b| b.0.cmp(&a.0));
        
        for (current_idx, next_idx) in adjacent_pairs {
            info!("Line segment upgrade shortcut: Upgrading line segment from point {} to {}", current_idx, next_idx);
            
            // Get the points
            let current_point = &contour.points[current_idx];
            let next_point = &contour.points[next_idx];
            
            // Convert points to Vec2 for calculations
            let p0 = Vec2::new(current_point.x, current_point.y);
            let p3 = Vec2::new(next_point.x, next_point.y);
            
            // Create two off-curve points at 1/3 and 2/3 positions
            let p1 = p0.lerp(p3, 1.0 / 3.0);
            let p2 = p0.lerp(p3, 2.0 / 3.0);
            
            // Create new off-curve points
            let off_curve1 = norad::ContourPoint::new(
                p1.x, 
                p1.y, 
                norad::PointType::OffCurve, 
                false, // not smooth
                None,  // no name
                None,  // no identifier
                None,  // no comments
            );
            
            let off_curve2 = norad::ContourPoint::new(
                p2.x, 
                p2.y, 
                norad::PointType::OffCurve, 
                false, // not smooth
                None,  // no name
                None,  // no identifier
                None,  // no comments
            );
            
            // Make a copy of the points to avoid borrowing issues
            let mut new_points = contour.points.clone();
            
            // Always change the end point to be a curve point
            new_points[next_idx].typ = norad::PointType::Curve;
            
            // Insert the off-curve points
            new_points.insert(current_idx + 1, off_curve1);
            new_points.insert(current_idx + 2, off_curve2);
            
            // Create a new contour with the updated points
            let new_contour = norad::Contour::new(
                new_points, 
                contour.identifier().cloned(),
                None, // No new lib needed
            );
            
            // Replace the old contour with the new one
            *contour = new_contour;
            modifications_made = true;
        }
    }
    
    // If any modifications were made, notify that the app state has changed
    if modifications_made {
        info!("Line segment upgrade shortcut: Successfully upgraded line segments to curves");
        app_state_changed.send(AppStateChanged);
    } else {
        info!("Line segment upgrade shortcut: No line segments were upgraded");
    }
}

