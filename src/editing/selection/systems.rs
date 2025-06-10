use super::components::*;
use super::DragPointState;
use super::DragSelectionState;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::core::state::AppState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::nudge::{EditEvent, NudgeState};
use crate::rendering::cameras::DesignCamera;
use crate::rendering::draw::AppStateChanged;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// A resource to hold the world position of a handled click.
/// This prevents multiple systems from reacting to the same click event.
#[derive(Resource)]
pub struct ClickWorldPosition;

// Constants for selection
const SELECTION_MARGIN: f32 = 16.0; // Distance in pixels for selection hit testing

/// System to handle mouse input for selection and hovering
pub fn handle_mouse_input(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    mut drag_state: ResMut<DragSelectionState>,
    mut drag_point_state: ResMut<DragPointState>,
    mut event_writer: EventWriter<EditEvent>,
    selectable_query: Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>), With<Selectable>>,
    selected_query: Query<(Entity, &Transform), With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    nudge_state: Res<NudgeState>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // Log at the beginning of each frame
    debug!(
        "Selection system running - current selected entities: {}",
        selection_state.selected.len()
    );

    // Skip if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            debug!("Selection skipped - knife mode active");
            return;
        }
    }

    // Only process when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            debug!("Selection skipped - select mode not active");
            return;
        }
    }

    // Don't process selection when hovering over UI
    if ui_hover_state.is_hovering_ui {
        debug!("Selection skipped - hovering over UI");
        return;
    }

    // Early return if no window
    let Ok(window) = windows.get_single() else {
        debug!("Selection skipped - no window");
        return;
    };

    // Early return if no camera
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        debug!("Selection skipped - no camera");
        return;
    };

    // If we're in the middle of a nudging operation, don't process mouse input
    // This prevents selection from being cleared during nudging
    if nudge_state.is_nudging {
        debug!("Selection skipped - nudging in progress");
        return;
    }

    // Update multi-select state based on shift key
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);
    selection_state.multi_select = shift_pressed;

    // Check for mouse click to start selection
    if mouse_button_input.just_pressed(MouseButton::Left) {
        debug!("Mouse button pressed - checking for selection");

        // Get cursor position in world coordinates
        if let Some(cursor_pos) = window.cursor_position().and_then(|pos| {
            camera.viewport_to_world_2d(camera_transform, pos).ok()
        }) {
            debug!(
                "Cursor position in world: ({:.1}, {:.1})",
                cursor_pos.x, cursor_pos.y
            );

            let mut best_hit = None;
            let mut min_dist_sq = SELECTION_MARGIN * SELECTION_MARGIN;

            for (entity, transform, _) in &selectable_query {
                let pos = transform.translation().truncate();
                let dist_sq = cursor_pos.distance_squared(pos);

                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    best_hit = Some((entity, pos));
                }
            }
            
            if let Some((entity, _)) = best_hit {
                // This click is on a selectable entity. Claim it.
                commands.insert_resource(ClickWorldPosition);

                let shift_held = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

                if !shift_held && selection_state.selected.contains(&entity) {
                    // Clicked on an already-selected entity without shift.
                    // This is the start of a drag, do nothing to selection state.
                    debug!("Clicked on already-selected entity {:?} - starting drag", entity);
                } else {
                    if !shift_held {
                        debug!("Clearing previous selection (count: {})", selection_state.selected.len());
                        for (e, _) in &selected_query {
                            commands.entity(e).remove::<Selected>();
                            debug!("Removing Selected from entity {:?}", e);
                        }
                        selection_state.selected.clear();
                    }
                    selection_state.selected.insert(entity);
                    commands.entity(entity).insert(Selected);
                    debug!("Added Selected to entity {:?}", entity);
                }

                // Ensure we're not already dragging before starting a new drag
                if drag_point_state.is_dragging {
                    debug!("WARNING: Starting drag while already dragging - resetting drag state");
                    drag_point_state.is_dragging = false;
                    drag_point_state.original_positions.clear();
                    drag_point_state.dragged_entities.clear();
                }

                drag_point_state.is_dragging = true;
                drag_point_state.start_position = Some(cursor_pos);
                drag_point_state.current_position = Some(cursor_pos);
                
                // Include all currently selected entities in the drag operation
                drag_point_state.dragged_entities = selection_state.selected.iter().cloned().collect();
                debug!("Starting drag with {} entities", drag_point_state.dragged_entities.len());

                // Save the original positions for all selected entities
                // We'll use the selected_query which contains the current selection with transforms
                drag_point_state.original_positions.clear();
                for (entity, transform) in &selected_query {
                    if selection_state.selected.contains(&entity) {
                        let pos = Vec2::new(transform.translation.x, transform.translation.y);
                        drag_point_state.original_positions.insert(entity, pos);
                        debug!("Stored original position for entity {:?}: ({:.1}, {:.1})", entity, pos.x, pos.y);
                    }
                }
                
                // Also store the position of the newly clicked entity if it's not already in selected_query
                // This handles the case where the entity was just selected in this frame
                if !drag_point_state.original_positions.contains_key(&entity) {
                    if let Some((_, transform, _)) = selectable_query.iter().find(|(e, _, _)| *e == entity) {
                        let pos = transform.translation().truncate();
                        drag_point_state.original_positions.insert(entity, pos);
                        debug!("Stored original position for newly clicked entity {:?}: ({:.1}, {:.1})", entity, pos.x, pos.y);
                    }
                }

                // Validate that we have original positions for all dragged entities
                let missing_positions = drag_point_state.dragged_entities.iter()
                    .filter(|e| !drag_point_state.original_positions.contains_key(e))
                    .count();
                if missing_positions > 0 {
                    debug!("WARNING: {} dragged entities are missing original positions", missing_positions);
                }

                // Notify about the edit
                event_writer.send(EditEvent {
                    edit_type: EditType::Normal,
                });

                debug!(
                    "Selection updated and drag started. Current selection count: {}, drag entities: {}",
                    selection_state.selected.len(),
                    drag_point_state.dragged_entities.len()
                );
            } else {
                debug!("No entity clicked, starting drag selection");
                // No entity clicked, start drag selection
                // BUT ONLY if we're not already dragging points
                if !drag_point_state.is_dragging {
                    drag_state.is_dragging = true;
                    drag_state.start_position = Some(cursor_pos);
                    drag_state.current_position = Some(cursor_pos);
                    drag_state.is_multi_select = selection_state.multi_select;

                    // Save previous selection for potential multi-select operations
                    drag_state.previous_selection =
                        selected_query.iter().map(|(entity, _)| entity).collect();

                    // If not multi-selecting, clear previous selection
                    if !selection_state.multi_select {
                        debug!("  Clearing previous selection for drag operation");
                        for (entity, _) in &selected_query {
                            commands.entity(entity).remove::<Selected>();
                            debug!("    -> Command to remove Selected component from entity {:?} queued", entity);
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
                } else {
                    debug!("Skipping drag selection - already dragging points");
                }
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
                        // Use get_entity to safely handle entity that might not exist
                        if let Some(mut entity_commands) = commands.get_entity(rect_entity) {
                            entity_commands.insert(SelectionRect {
                                start: start_pos,
                                end: cursor_pos,
                            });
                        }
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
                    for (entity, _) in &selected_query {
                        if !drag_state.previous_selection.contains(&entity) {
                            commands.entity(entity).remove::<Selected>();
                            selection_state.selected.remove(&entity);
                            debug!("  -> Command to remove Selected component from entity {:?} queued (drag)", entity);
                        }
                    }

                    for &entity in &drag_state.previous_selection {
                        if !selection_state.selected.contains(&entity) {
                            commands.entity(entity).insert(Selected);
                            selection_state.selected.insert(entity);
                            debug!("  -> Command to add Selected component to entity {:?} queued (drag restore)", entity);
                        }
                    }
                } else {
                    // Clear selection for non-multi-select
                    for (entity, _) in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                        debug!("  -> Command to remove Selected component from entity {:?} queued (drag clear)", entity);
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
                            debug!("  -> Command to remove Selected component from entity {:?} queued (drag toggle)", entity);
                        } else {
                            selection_state.selected.insert(entity);
                            commands.entity(entity).insert(Selected);
                            debug!("  -> Command to add Selected component to entity {:?} queued (drag select)", entity);
                        }
                    }
                }
            }
        }
    }

    // Handle mouse button release
    if mouse_button_input.just_released(MouseButton::Left) {
        debug!("Mouse released - checking for active drags");
        
        // Handle point drag end
        if drag_point_state.is_dragging {
            debug!("Ending point drag - {} entities were being dragged", drag_point_state.dragged_entities.len());
            drag_point_state.is_dragging = false;

            // Only send edit event if points were actually moved
            if drag_point_state.start_position != drag_point_state.current_position {
                event_writer.send(EditEvent {
                    edit_type: EditType::DragUp,
                });
                debug!("Point drag completed with movement");
            } else {
                debug!("Point drag completed without movement");
            }

            // Clean up drag state
            drag_point_state.start_position = None;
            drag_point_state.current_position = None;
            drag_point_state.dragged_entities.clear();
            drag_point_state.original_positions.clear();
        }

        // Handle drag selection end
        if drag_state.is_dragging {
            debug!("Ending drag selection");
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
                debug!(
                    "Drag selection completed with {} entities selected",
                    selection_state.selected.len()
                );
            }
        }
        
        if !drag_point_state.is_dragging && !drag_state.is_dragging {
            debug!("Mouse released - no active drags to clean up");
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
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
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
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
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
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
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
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<Selected>,
    >,
    drag_point_state: Res<DragPointState>,
    select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
    >,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
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

    // Determine if we're currently dragging points (for enhanced visibility)
    let is_dragging = drag_point_state.is_dragging;

    // Increase the visual size slightly during dragging for better visibility
    let size_multiplier = if is_dragging { 1.25 } else { 1.0 };

    // Select the color - make it brighter during dragging
    let selection_color = if is_dragging {
        // Brighter orange during dragging
        Color::srgb(1.0, 0.7, 0.2)
    } else {
        crate::ui::theme::SELECTED_POINT_COLOR
    };

    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        // Use a much higher Z offset to ensure selected points always render on top of normal glyph points
        let position_3d =
            Vec3::new(position.x, position.y, transform.translation().z + 100.0);
        let position_2d = position_3d.truncate();

        // Different rendering based on point type
        if point_type.is_on_curve && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE {
            // Draw a filled square for on-curve points to completely cover the underlying glyph point
            let half_size = crate::ui::theme::SELECTION_POINT_RADIUS
                / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT
                * size_multiplier;

            // Draw a filled rectangle to completely cover the underlying green square
            gizmos.rect_2d(
                position_2d,
                Vec2::new(half_size * 2.0, half_size * 2.0),
                selection_color,
            );

            // Draw a smaller inner circle for visual distinction
            gizmos.circle_2d(
                position_2d,
                half_size * crate::ui::theme::ON_CURVE_INNER_CIRCLE_RATIO,
                selection_color,
            );
        } else {
            // Draw a circle for off-curve points
            gizmos.circle_2d(
                position_2d,
                crate::ui::theme::SELECTION_POINT_RADIUS
                    * crate::ui::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER
                    * size_multiplier,
                selection_color,
            );

            // For off-curve points, also draw a smaller inner circle
            if !point_type.is_on_curve {
                gizmos.circle_2d(
                    position_2d,
                    crate::ui::theme::SELECTION_POINT_RADIUS
                        * crate::ui::theme::OFF_CURVE_INNER_CIRCLE_RATIO
                        * size_multiplier,
                    selection_color,
                );
            }
        }

        // Always draw the crosshair for all selected points
        let line_size = if point_type.is_on_curve
            && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE
        {
            // For on-curve square points, use the half_size of the square
            crate::ui::theme::SELECTION_POINT_RADIUS
                / crate::ui::theme::ON_CURVE_SQUARE_ADJUSTMENT
        } else {
            // For off-curve circle points, use the radius
            crate::ui::theme::SELECTION_POINT_RADIUS
                * crate::ui::theme::SELECTED_CIRCLE_RADIUS_MULTIPLIER
        };

        // Apply size multiplier to crosshairs as well
        let line_size = line_size * size_multiplier;

        // Draw crosshair with optional thickness
        // Note: In Bevy, we can't directly modify gizmos.config.line_width
        // We'll need to draw with the default thickness instead

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

        // If dragging, draw a second set of lines to make them appear thicker
        if is_dragging {
            // Offset slightly to create thicker appearance
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

/// System to draw visual indicators for hovered entities
#[allow(dead_code)] // Disabled per user request - hover functionality removed
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<Hovered>,
    >,
    _select_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>,
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

        debug!("Selection cleared due to app state change");
    }
}

/// System to update the actual glyph data when a point is nudged
pub fn update_glyph_data_from_selection(
    query: Query<
        (&Transform, &GlyphPointReference),
        (With<Selected>, Changed<Transform>, Without<crate::systems::sort_manager::SortPointEntity>),
    >,
    mut app_state: ResMut<AppState>,
    // Track if we're in a nudging operation
    _nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
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

    // Process each nudged point (excluding sort points)
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

                            debug!(
                                "Updated UFO glyph data for point {} in contour {} of glyph {}",
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

/// System to handle dragging of selected points
pub fn handle_point_drag(
    windows: Query<&Window, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    mut drag_point_state: ResMut<DragPointState>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut crate::editing::selection::nudge::PointCoordinates,
            Option<&GlyphPointReference>,
            Option<&crate::systems::sort_manager::SortCrosshair>,
        ),
        With<Selected>,
    >,
    mut app_state: ResMut<AppState>,
    mut event_writer: EventWriter<EditEvent>,
) {
    // Early return if not dragging
    if !drag_point_state.is_dragging {
        return;
    }

    debug!("handle_point_drag: Processing drag for {} selected entities", query.iter().count());

    // Early return if no window or camera
    let Ok(window) = windows.get_single() else {
        debug!("handle_point_drag: No window found");
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        debug!("handle_point_drag: No camera found");
        return;
    };

    if let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok())
    {
        drag_point_state.current_position = Some(cursor_pos);

        if let Some(start_pos) = drag_point_state.start_position {
            let total_movement = cursor_pos - start_pos;
            let mut movement = total_movement;

            // Handle constrained movement with Shift key
            if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
                if total_movement.x.abs() > total_movement.y.abs() {
                    movement.y = 0.0;
                } else {
                    movement.x = 0.0;
                }
            }

            let mut updated_count = 0;

            for (entity, mut transform, mut coordinates, point_ref, crosshair_ref) in &mut query {
                if let Some(original_pos) = drag_point_state.original_positions.get(&entity) {
                    let new_pos = *original_pos + movement;
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    
                    // Logic for sort crosshair drag
                    if crosshair_ref.is_some() {
                        transform.translation.z = 25.0; // Keep on top
                    } 
                    // Logic for glyph point drag
                    else if let Some(point_ref) = point_ref {
                        transform.translation.z = 5.0;

                        let snapped_pos = if SNAP_TO_GRID_ENABLED {
                            let grid_size = SNAP_TO_GRID_VALUE;
                            Vec2::new(
                                (new_pos.x / grid_size).round() * grid_size,
                                (new_pos.y / grid_size).round() * grid_size,
                            )
                        } else {
                            new_pos
                        };

                        transform.translation.x = snapped_pos.x;
                        transform.translation.y = snapped_pos.y;
                        coordinates.position = snapped_pos;

                        if let Some(point) = app_state.get_point_mut(point_ref) {
                            point.x = transform.translation.x;
                            point.y = transform.translation.y;
                            updated_count += 1;
                        }
                    }
                }
            }

            if updated_count > 0 {
                debug!(
                    "Updated {} points in the font data with movement ({:.2}, {:.2})",
                    updated_count, movement.x, movement.y
                );
            }

            event_writer.send(EditEvent {
                edit_type: EditType::Drag,
            });
        }
    }
}

/// A system to clean up the ClickWorldPosition resource at the end of the frame.
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<ClickWorldPosition>();
}
