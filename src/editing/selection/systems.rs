use super::components::*;
use super::DragPointState;
use super::DragSelectionState;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::core::state::AppState;
use crate::core::cursor::CursorInfo;
use crate::editing::edit_type::EditType;
use crate::editing::selection::nudge::{EditEvent, NudgeState};
use crate::rendering::cameras::DesignCamera;
use bevy::input::ButtonInput;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::window::PrimaryWindow;
#[allow(unused_imports)]
use crate::geometry::point::{EditPoint, EntityId, EntityKind};
#[allow(unused_imports)]
use crate::ui::panes::design_space::{DPoint, ViewPort};

/// Event to signal that app state has changed
#[derive(Event, Debug, Clone)]
pub struct AppStateChanged;

/// A resource to hold the world position of a handled click.
/// This prevents multiple systems from reacting to the same click event.
#[derive(Resource)]
pub struct ClickWorldPosition;

// Constants for selection
#[allow(dead_code)]
const SELECTION_MARGIN: f32 = 16.0; // Distance in pixels for selection hit testing

/// System to handle mouse input for selection and hovering
#[allow(dead_code)]
pub fn handle_mouse_input(
    mut commands: Commands,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drag_state: ResMut<DragSelectionState>,
    mut drag_point_state: ResMut<DragPointState>,
    mut event_writer: EventWriter<EditEvent>,
    selectable_query: Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>), With<Selectable>>,
    selected_query: Query<(Entity, &Transform), With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    nudge_state: Res<NudgeState>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
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

    // Only allow selection when there's an active sort in text editor
    if let Some(text_editor_state) = text_editor_state.as_ref() {
        if text_editor_state.get_active_sort().is_none() {
            debug!("Selection skipped - no active sort");
            return;
        }
    } else {
        debug!("Selection skipped - no text editor state");
        return;
    }

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

        // Get cursor position in design coordinates from the resource
        if let Some(cursor_dpos) = cursor.design_position {
            let cursor_pos = cursor_dpos.to_raw();
            debug!(
                "Cursor position in design: ({:.1}, {:.1})",
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
                event_writer.write(EditEvent {
                    edit_type: EditType::Normal,
                });

                debug!(
                    "Selection updated and drag started. Current selection count: {}, drag entities: {}",
                    selection_state.selected.len(),
                    drag_point_state.dragged_entities.len()
                );
            } else {
                // Clicked on empty space
                debug!("Clicked on empty space");
                
                // Clear selection if not multi-selecting
                if !shift_pressed {
                    debug!("Clearing selection (no shift held)");
                    
                    // Remove Selected component from all selected entities
                    for (entity, _) in &selected_query {
                        commands.entity(entity).remove::<Selected>();
                        debug!("Removing Selected from entity {:?}", entity);
                    }
                    selection_state.selected.clear();
                    
                    // Also clean up any existing selection rectangles
                    for entity in &selection_rect_query {
                        commands.entity(entity).despawn();
                    }
                }

                // Start a drag selection
                drag_state.is_dragging = true;
                drag_state.start_position = Some(cursor_pos);
                drag_state.current_position = Some(cursor_pos);
                drag_state.is_multi_select = shift_pressed;
                
                // Store the previous selection state
                drag_state.previous_selection = selection_state.selected.iter().cloned().collect();
                
                // Spawn a selection rectangle entity
                commands.spawn((
                    SelectionRect {
                        start: cursor_pos,
                        end: cursor_pos,
                    },
                    Name::new("SelectionRect"),
                ));
                
                debug!("Started drag selection at ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);
            }
        }
    }

    // Handle mouse release
    if mouse_button_input.just_released(MouseButton::Left) {
        debug!("Mouse button released");
        
        // End point dragging
        if drag_point_state.is_dragging {
            debug!("Ending point drag");
            drag_point_state.is_dragging = false;
            drag_point_state.start_position = None;
            drag_point_state.current_position = None;
            drag_point_state.dragged_entities.clear();
            drag_point_state.original_positions.clear();
        }
        
        // End drag selection
        if drag_state.is_dragging {
            debug!("Ending drag selection");
            
            // Finalize selection based on what was in the rectangle
            if let Some(start_pos) = drag_state.start_position {
                if let Some(end_pos) = drag_state.current_position {
                    // Calculate selection rectangle bounds
                    let min_x = start_pos.x.min(end_pos.x);
                    let max_x = start_pos.x.max(end_pos.x);
                    let min_y = start_pos.y.min(end_pos.y);
                    let max_y = start_pos.y.max(end_pos.y);
                    
                    // If not multi-select, start fresh
                    if !drag_state.is_multi_select {
                        // Clear existing selection
                        for (entity, _) in &selected_query {
                            commands.entity(entity).remove::<Selected>();
                        }
                        selection_state.selected.clear();
                    }
                    
                    // Select all entities within the rectangle
                    for (entity, transform, _) in &selectable_query {
                        let pos = transform.translation().truncate();
                        
                        if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
                            if !selection_state.selected.contains(&entity) {
                                selection_state.selected.insert(entity);
                                commands.entity(entity).insert(Selected);
                                debug!("Selected entity {:?} in drag rectangle", entity);
                            }
                        }
                    }
                }
            }
            
            // Clean up drag state
            drag_state.is_dragging = false;
            drag_state.start_position = None;
            drag_state.current_position = None;
            drag_state.is_multi_select = false;
            drag_state.previous_selection.clear();
            
            // Remove selection rectangle
            for entity in &selection_rect_query {
                commands.entity(entity).despawn();
            }
        }
    }

    // Handle mouse movement during drag operations
    if let Some(cursor_dpos) = cursor.design_position {
        let cursor_pos = cursor_dpos.to_raw();
        // Update drag selection
        if drag_state.is_dragging {
            drag_state.current_position = Some(cursor_pos);
            
                         // Update selection rectangle - this would need a separate query with mutable access
             // For now, we'll skip this update as it requires restructuring the system
        }
        
        // Update point drag
        if drag_point_state.is_dragging {
            drag_point_state.current_position = Some(cursor_pos);
        }
    }
}

/// System to handle selection shortcuts (Ctrl+A for select all, etc.)
pub fn handle_selection_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut selection_state: ResMut<SelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
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

    // Only allow selection shortcuts when there's an active sort in text editor
    if let Some(text_editor_state) = text_editor_state.as_ref() {
        if text_editor_state.get_active_sort().is_none() {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
        debug!("Cleared selection");
    }

    // Handle Ctrl+A (select all)
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft) 
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);
    
    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyA) {
        debug!("Select all shortcut pressed");
        
        // Clear current selection
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
        
        // Select all selectable entities
        for entity in &selectable_query {
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);
        }
        
        debug!("Selected all {} entities", selection_state.selected.len());
        
        // Send edit event
        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });
    }
}

/// System to update which entities are being hovered over by the mouse
#[allow(dead_code)]
pub fn update_hover_state(
    mut _commands: Commands,
    _windows: Query<&Window, With<PrimaryWindow>>,
    _camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    _selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
    _hovered_query: Query<Entity, With<Hovered>>,
) {
    // Hover functionality is disabled per user request
}

/// System to render the selection rectangle during drag operations
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    selection_rect_query: Query<&SelectionRect>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
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

/// System to render selected entities with visual feedback
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
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
) {
    // Skip rendering if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only render selection in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for (transform, point_type) in &selected_query {
        let pos = transform.translation().truncate();
        
        // Choose color based on point type
        let color = if point_type.is_on_curve {
            Color::srgb(1.0, 0.7, 0.2) // Orange for on-curve points
        } else {
            Color::srgb(0.2, 0.7, 1.0) // Blue for off-curve points
        };
        
        // Draw selection indicator
        let radius = if drag_point_state.is_dragging { 6.0 } else { 5.0 };
        gizmos.circle_2d(pos, radius, color);
        
        // Draw inner circle for contrast
        gizmos.circle_2d(pos, radius * 0.6, Color::WHITE);
    }
}

/// System to render hovered entities (disabled for now)
#[allow(dead_code)]
pub fn render_hovered_entities(
    mut _gizmos: Gizmos,
    _hovered_query: Query<
        (
            &GlobalTransform,
            &crate::editing::selection::components::PointType,
        ),
        With<Hovered>,
    >,
) {
    // Hover functionality is disabled per user request
}

/// System to update the actual glyph data when a point is moved
pub fn update_glyph_data_from_selection(
    query: Query<
        (&Transform, &GlyphPointReference),
        (With<Selected>, Changed<Transform>, Without<crate::systems::sort_manager::SortPointEntity>),
    >,
    mut app_state: ResMut<AppState>,
    // Track if we're in a nudging operation
    _nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
) {
    // Skip processing if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Early return if no points were moved
    if query.is_empty() {
        return;
    }

    // Only modify app_state after detaching its change detection
    let app_state = app_state.bypass_change_detection();

    // Process each moved point
    for (transform, point_ref) in query.iter() {
        // Use the correct method to update point position
        let updated = app_state.set_point_position(
            &point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            transform.translation.x as f64, // Convert f32 to f64
            transform.translation.y as f64, // Convert f32 to f64
        );

        if updated {
            debug!(
                "Updated UFO glyph data for point {} in contour {} of glyph {}",
                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
            );
        } else {
            warn!(
                "Failed to update UFO glyph data for point {} in contour {} of glyph {} - invalid indices",
                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
            );
        }
    }
}

/// System to handle key releases for nudging
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

/// System to clear selection when app state changes (e.g., when codepoint changes)
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

/// System to handle advanced point dragging with constraints and snapping
pub fn handle_point_drag(
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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
    // Only drag if the resource says we are
    if !drag_point_state.is_dragging {
        return;
    }

    if let Some(cursor_dpos) = cursor.design_position {
        let cursor_pos = cursor_dpos.to_raw();
        drag_point_state.current_position = Some(cursor_pos);

        if let Some(start_pos) = drag_point_state.start_position {
            let total_movement = cursor_pos - start_pos;
            let mut movement = total_movement;

            // Handle constrained movement with Shift key
            if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
                if total_movement.x.abs() > total_movement.y.abs() {
                    movement.y = 0.0; // Constrain to horizontal
                } else {
                    movement.x = 0.0; // Constrain to vertical
                }
            }

            let mut updated_count = 0;

            for (entity, mut transform, mut coordinates, point_ref, sort_crosshair) in &mut query {
                if let Some(original_pos) = drag_point_state.original_positions.get(&entity) {
                    let new_pos = *original_pos + movement;
                    
                    // Handle sort crosshair drag (no snapping, keep on top)
                    if sort_crosshair.is_some() {
                        transform.translation.x = new_pos.x;
                        transform.translation.y = new_pos.y;
                        transform.translation.z = 25.0; // Keep crosshairs on top
                        coordinates.position = new_pos;
                    }
                    // Handle glyph point drag (with snapping)
                    else if let Some(point_ref) = point_ref {
                        // Apply grid snapping if enabled
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
                        transform.translation.z = 5.0; // Keep glyph points above background
                        coordinates.position = snapped_pos;

                        // Update UFO data for glyph points
                        let updated = app_state.set_point_position(
                            &point_ref.glyph_name,
                            point_ref.contour_index,
                            point_ref.point_index,
                            transform.translation.x as f64, // Convert f32 to f64
                            transform.translation.y as f64, // Convert f32 to f64
                        );
                        if updated {
                            updated_count += 1;
                        }
                    }
                    // Handle other draggable entities (no snapping, normal Z layer)
                    else {
                        transform.translation.x = new_pos.x;
                        transform.translation.y = new_pos.y;
                        transform.translation.z = 10.0; // Middle layer
                        coordinates.position = new_pos;
                    }
                }
            }

            if updated_count > 0 {
                debug!("Updated {} UFO points during drag", updated_count);
                
                // Send edit event
                event_writer.write(EditEvent {
                    edit_type: EditType::Normal,
                });
            }
        }
    }
}

/// System to clean up the click resource
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<ClickWorldPosition>();
} 