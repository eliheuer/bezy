use super::components::*;
use super::coordinate_system::SelectionCoordinateSystem;
use super::DragPointState;
use super::DragSelectionState;
use crate::core::settings::{SNAP_TO_GRID_ENABLED, SNAP_TO_GRID_VALUE};
use crate::core::state::AppState;
use crate::core::pointer::PointerInfo;
use crate::core::input::{InputEvent, InputState, helpers};
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

// Legacy handle_mouse_input system removed - replaced by handle_selection_input_events

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
    let rect_count = selection_rect_query.iter().len();
    info!("render_selection_rect called: {} rects", rect_count);
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
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
) {
    let selected_count = selected_query.iter().count();
    if selected_count > 0 {
        info!("Selection: Rendering {} selected entities", selected_count);
    }
    
    // Skip rendering if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
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
        let position_3d = Vec3::new(position.x, position.y, transform.translation().z + 100.0);
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
        let line_size = if point_type.is_on_curve && crate::ui::theme::USE_SQUARE_FOR_ON_CURVE {
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

        // Draw crosshair
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
    pointer_info: Res<PointerInfo>,
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

    let cursor_pos = pointer_info.design.to_raw();
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

/// System to clean up the click resource
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<ClickWorldPosition>();
}

/// System to process selection input events from the new input system
pub fn process_selection_input_events(
    mut commands: Commands,
    mut input_events: EventReader<crate::core::input::InputEvent>,
    input_state: Res<crate::core::input::InputState>,
    mut drag_state: ResMut<DragSelectionState>,
    mut drag_point_state: ResMut<DragPointState>,
    mut event_writer: EventWriter<EditEvent>,
    selectable_query: Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>, Option<&PointType>), With<Selectable>>,
    selected_query: Query<(Entity, &Transform), With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_point_entities: Query<&crate::systems::sort_manager::SortPointEntity>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
) {
    // Only process if in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    for event in input_events.read() {
        info!("Selection: Processing event: {:?}", event);
        
        // Skip if UI is consuming input
        if crate::core::input::helpers::is_ui_consuming(&input_state) {
            info!("Selection: Skipping event - UI is consuming input");
            continue;
        }

        // Only handle events that are relevant to selection
        match event {
            crate::core::input::InputEvent::MouseClick { button, position, modifiers } => {
                if *button == bevy::input::mouse::MouseButton::Left {
                    info!("Selection: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // Call the selection click handler
                    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
                        handle_selection_click(
                            &mut commands,
                            position,
                            modifiers,
                            &mut drag_state,
                            &mut drag_point_state,
                            &mut event_writer,
                            &selectable_query,
                            &selected_query,
                            &mut selection_state,
                            active_sort_entity,
                            &sort_point_entities,
                        );
                    } else {
                        info!("Selection: No active sort, skipping click handling");
                    }
                }
            }
            crate::core::input::InputEvent::MouseDrag { button, start_position, current_position, delta, modifiers } => {
                if *button == bevy::input::mouse::MouseButton::Left {
                    info!("Selection: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    info!("Selection: active_sort_entity={:?}", active_sort_state.active_sort_entity);
                    // Call the selection drag handler
                    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
                        info!("Selection: Calling handle_selection_drag...");
                        handle_selection_drag(
                            &mut commands,
                            &start_position,
                            &current_position,
                            &delta,
                            modifiers,
                            &mut drag_state,
                            &mut drag_point_state,
                            &mut event_writer,
                            &selectable_query,
                            &mut selection_state,
                            active_sort_entity,
                            &sort_point_entities,
                            &selection_rect_query,
                        );
                        info!("Selection: handle_selection_drag completed");
                    } else {
                        info!("Selection: No active sort entity, skipping drag handling");
                    }
                }
            }
            crate::core::input::InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == bevy::input::mouse::MouseButton::Left {
                    info!("Selection: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // Call the selection release handler
                    if let Some(_active_sort_entity) = active_sort_state.active_sort_entity {
                        handle_selection_release(
                            &mut commands,
                            position,
                            modifiers,
                            &mut drag_state,
                            &mut drag_point_state,
                            &mut event_writer,
                            &mut selection_state,
                            &selection_rect_query,
                        );
                    }
                }
            }
            crate::core::input::InputEvent::KeyPress { key, modifiers } => {
                if matches!(key, bevy::input::keyboard::KeyCode::KeyA | bevy::input::keyboard::KeyCode::Escape) {
                    info!("Selection: Processing key press {:?} with modifiers {:?}", key, modifiers);
                    // Call the selection key press handler
                    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
                        handle_selection_key_press(
                            &mut commands,
                            key,
                            modifiers,
                            &selectable_query,
                            &selected_query,
                            &mut selection_state,
                            &mut event_writer,
                            active_sort_entity,
                            &sort_point_entities,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Handle mouse click for selection
pub fn handle_selection_click(
    commands: &mut Commands,
    position: &DPoint,
    modifiers: &crate::core::input::ModifierState,
    _drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>, Option<&PointType>), With<Selectable>>,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    debug!("=== HANDLE SELECTION CLICK ===");
    let cursor_pos = position.to_raw();
    debug!("Click position: {:?} (raw: {:?})", position, cursor_pos);
    debug!("Modifiers: {:?}", modifiers);
    debug!("Active sort entity: {:?}", active_sort_entity);
    debug!("Current selection count: {}", selection_state.selected.len());
    
    // Count selectable points in active sort
    let mut total_selectable = 0;
    let mut active_sort_selectable = 0;
    for (entity, _, _, _) in selectable_query.iter() {
        total_selectable += 1;
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            if sort_point_entity.sort_entity == active_sort_entity {
                active_sort_selectable += 1;
            }
        }
    }
    debug!("Total selectable entities: {}, in active sort: {}", total_selectable, active_sort_selectable);
    
    debug!("Selection click at ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);

    let mut best_hit = None;
    let mut min_dist_sq = SELECTION_MARGIN * SELECTION_MARGIN;
    debug!("Using selection margin: {}", SELECTION_MARGIN);

    // Find the closest selectable entity that belongs to the active sort
    let mut checked_points = 0;
    let mut points_in_active_sort = 0;
    
    for (entity, transform, glyph_ref, point_type) in selectable_query.iter() {
        checked_points += 1;
        
        // Check if this entity belongs to the active sort
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            if sort_point_entity.sort_entity != active_sort_entity {
                continue; // Skip points that don't belong to the active sort
            }
            points_in_active_sort += 1;
        } else {
            debug!("Entity {:?} has no SortPointEntity component", entity);
            continue; // Skip entities that aren't sort points
        }

        let pos = transform.translation().truncate();
        let dist_sq = cursor_pos.distance_squared(pos);
        let distance = dist_sq.sqrt();
        
        debug!("Point {:?} at {:?}, distance: {:.1} (squared: {:.1})", entity, pos, distance, dist_sq);

        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            best_hit = Some((entity, pos, glyph_ref, point_type));
            debug!("New best hit: {:?} at distance {:.1}", entity, distance);
        }
    }
    
    debug!("Checked {} total points, {} in active sort", checked_points, points_in_active_sort);
    debug!("Best hit found: {:?}", best_hit.map(|(e, p, _, _)| (e, p)));
    
    if let Some((entity, pos, glyph_ref, point_type)) = best_hit {
        // Clicked on a selectable entity in the active sort
        commands.insert_resource(ClickWorldPosition);

        let shift_held = modifiers.shift;

        if !shift_held && selection_state.selected.contains(&entity) {
            // Clicked on already selected entity - start drag
            debug!("Clicked on already-selected entity {:?} - starting drag", entity);
        } else {
            // Handle selection
            if !shift_held {
                // Clear previous selection
                debug!("Clearing previous selection (count: {})", selection_state.selected.len());
                for (e, _) in selected_query.iter() {
                    commands.entity(e).remove::<Selected>();
                }
                selection_state.selected.clear();
            }
            
            // Add to selection
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);
            
            // Log point type for debugging
            if let Some(pt) = point_type {
                let point_type_str = if pt.is_on_curve { "on-curve" } else { "off-curve" };
                debug!("Selected {} point at ({:.1}, {:.1})", point_type_str, pos.x, pos.y);
            }
            
            if let Some(glyph_ref) = glyph_ref {
                debug!("Selected point in glyph '{}', contour {}, point {}", 
                       glyph_ref.glyph_name, glyph_ref.contour_index, glyph_ref.point_index);
            }
        }

        // Start drag operation
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

        // Save original positions
        drag_point_state.original_positions.clear();
        for (entity, transform) in selected_query.iter() {
            if selection_state.selected.contains(&entity) {
                let pos = Vec2::new(transform.translation.x, transform.translation.y);
                drag_point_state.original_positions.insert(entity, pos);
            }
        }
        
        // Also store position of newly clicked entity
        if !drag_point_state.original_positions.contains_key(&entity) {
            drag_point_state.original_positions.insert(entity, pos);
        }

        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });

        debug!("Selection updated and drag started. Current selection count: {}", selection_state.selected.len());
    } else {
        // Clicked on empty space
        debug!("Clicked on empty space - clearing selection");
        commands.insert_resource(ClickWorldPosition);
        
        // Clear selection if not multi-selecting
        if !modifiers.shift {
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            debug!("Cleared all selections");
        }
    }
}

/// Handle mouse drag for selection
pub fn handle_selection_drag(
    commands: &mut Commands,
    start_position: &DPoint,
    current_position: &DPoint,
    delta: &Vec2,
    modifiers: &crate::core::input::ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>, Option<&PointType>), With<Selectable>>,
    selection_state: &mut ResMut<SelectionState>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
    selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    info!("=== HANDLE_SELECTION_DRAG START ===");
    info!("handle_selection_drag called: start={:?}, current={:?}, delta={:?}, is_dragging={}", start_position, current_position, delta, drag_state.is_dragging);
    info!("drag_state: is_dragging={}, start_position={:?}, current_position={:?}, selection_rect_entity={:?}", 
          drag_state.is_dragging, drag_state.start_position, drag_state.current_position, drag_state.selection_rect_entity);
    // Only set start_position and create entity at the very start of a drag
    if !drag_state.is_dragging {
        info!("Starting new drag selection...");
        drag_state.is_dragging = true;
        drag_state.start_position = Some(*start_position);
        drag_state.current_position = Some(*current_position);
        drag_state.is_multi_select = modifiers.shift;
        
        // Store previous selection for multi-select mode
        if modifiers.shift {
            drag_state.previous_selection = selection_state.selected.iter().cloned().collect();
            info!("Multi-select mode: stored {} previous selections", drag_state.previous_selection.len());
        }
        
        info!("Drag selection started at {:?}", start_position);
        let rect_entity = commands.spawn(SelectionRect {
            start: start_position.to_raw(),
            end: current_position.to_raw(),
        }).id();
        drag_state.selection_rect_entity = Some(rect_entity);
        info!("SelectionRect entity created with ID {:?}: start={:?}, end={:?}", rect_entity, start_position.to_raw(), current_position.to_raw());
    } else {
        // Only update current_position during drag
        info!("Continuing existing drag selection...");
        drag_state.current_position = Some(*current_position);
        // Only update the entity if it exists
        if let Some(rect_entity) = drag_state.selection_rect_entity {
            info!("Updating SelectionRect entity {:?}", rect_entity);
            if let Ok(mut entity_commands) = commands.get_entity(rect_entity) {
                entity_commands.insert(SelectionRect {
                    start: drag_state.start_position.unwrap_or(*start_position).to_raw(),
                    end: current_position.to_raw(),
                });
                info!("SelectionRect entity updated: start={:?}, end={:?}", drag_state.start_position.unwrap_or(*start_position).to_raw(), current_position.to_raw());
            } else {
                info!("ERROR: Could not get entity commands for SelectionRect entity {:?}", rect_entity);
            }
        } else {
            info!("ERROR: No SelectionRect entity found in drag_state!");
        }
    }
    
    // Update selection based on what's inside the rectangle
    if let (Some(start_pos), Some(current_pos)) = (drag_state.start_position, drag_state.current_position) {
        info!("Selection: Marquee rect coordinates - start: {:?}, current: {:?}", start_pos, current_pos);
        
        // In multi-select mode, start with previous selection
        if drag_state.is_multi_select {
            // Reset to previous selection
            for (entity, _, _, _) in selectable_query.iter() {
                if !drag_state.previous_selection.contains(&entity) {
                    commands.entity(entity).remove::<Selected>();
                    selection_state.selected.remove(&entity);
                }
            }
            
            for &entity in &drag_state.previous_selection {
                if !selection_state.selected.contains(&entity) {
                    commands.entity(entity).insert(Selected);
                    selection_state.selected.insert(entity);
                }
            }
        } else {
            // Clear selection for non-multi-select
            for (entity, _, _, _) in selectable_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
        }
        
        // Add entities in the rectangle to selection
        let mut points_in_rect = 0;
        let mut points_selected = 0;
        
        info!("Selection: Checking {} selectable entities for marquee selection", selectable_query.iter().count());
        
        // Collect entity positions for debugging and coordinate system analysis
        let entity_positions: Vec<(Entity, Vec2)> = selectable_query
            .iter()
            .map(|(entity, transform, _, _)| (entity, transform.translation().truncate()))
            .collect();
        
        // Use centralized coordinate system for debugging
        let debug_info = SelectionCoordinateSystem::debug_coordinate_ranges(&entity_positions, &start_pos, &current_pos);
        info!("Selection: {}", debug_info);
        
        for (entity, entity_pos) in &entity_positions {
            // Use centralized coordinate system to check if entity is inside the marquee rectangle
            if SelectionCoordinateSystem::is_point_in_rectangle(entity_pos, &start_pos, &current_pos) {
                points_in_rect += 1;
                info!("Selection: Entity {:?} is inside marquee rect! Position: {:?}", entity, entity_pos);
                if drag_state.is_multi_select && drag_state.previous_selection.contains(entity) {
                    // Toggle off if previously selected
                    selection_state.selected.remove(entity);
                    commands.entity(*entity).remove::<Selected>();
                    info!("Selection: Toggled off entity {:?}", entity);
                } else {
                    // Add to selection
                    selection_state.selected.insert(*entity);
                    commands.entity(*entity).insert(Selected);
                    points_selected += 1;
                    info!("Selection: Added entity {:?} to selection", entity);
                }
            } else {
                // Debug: Show why entity is not selected using centralized coordinate system
                let rect_entity_start = SelectionCoordinateSystem::design_to_entity_coordinates(&start_pos);
                let rect_entity_end = SelectionCoordinateSystem::design_to_entity_coordinates(&current_pos);
                
                let distance_x = if entity_pos.x < rect_entity_start.x.min(rect_entity_end.x) {
                    rect_entity_start.x.min(rect_entity_end.x) - entity_pos.x
                } else if entity_pos.x > rect_entity_start.x.max(rect_entity_end.x) {
                    entity_pos.x - rect_entity_start.x.max(rect_entity_end.x)
                } else {
                    0.0
                };
                
                let distance_y = if entity_pos.y < rect_entity_start.y.min(rect_entity_end.y) {
                    rect_entity_start.y.min(rect_entity_end.y) - entity_pos.y
                } else if entity_pos.y > rect_entity_start.y.max(rect_entity_end.y) {
                    entity_pos.y - rect_entity_start.y.max(rect_entity_end.y)
                } else {
                    0.0
                };
                
                if distance_x > 0.0 || distance_y > 0.0 {
                    info!("Selection: Entity {:?} outside rect - X: {:.1} units, Y: {:.1} units", 
                          entity, distance_x, distance_y);
                }
            }
        }
        
        info!("Marquee selection: {} points in rect, {} points selected", points_in_rect, points_selected);
        
        info!("Marquee selection updated: {} points selected", selection_state.selected.len());
    }
}

/// Handle mouse release for selection
pub fn handle_selection_release(
    commands: &mut Commands,
    position: &DPoint,
    _modifiers: &crate::core::input::ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    event_writer: &mut EventWriter<EditEvent>,
    selection_state: &mut ResMut<SelectionState>,
    selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    let release_pos = position.to_raw();
    debug!("Selection release at ({:.1}, {:.1})", release_pos.x, release_pos.y);

    if drag_point_state.is_dragging {
        drag_point_state.is_dragging = false;
        drag_point_state.start_position = None;
        drag_point_state.current_position = None;
        drag_point_state.dragged_entities.clear();
        drag_point_state.original_positions.clear();
    } else if drag_state.is_dragging {
        // End drag selection
        drag_state.is_dragging = false;
        let rect_entity = drag_state.selection_rect_entity;
        drag_state.selection_rect_entity = None;
        drag_state.start_position = None;
        drag_state.current_position = None;
        drag_state.is_multi_select = false;
        
        debug!("Ended drag selection. Final selection count: {}", selection_state.selected.len());

        info!("Marquee selection complete. {} points selected.", selection_state.selected.len());
        // Clean up the selection rectangle entity
        if let Some(rect_entity) = rect_entity {
            commands.entity(rect_entity).despawn();
            info!("SelectionRect entity despawned on release");
        }
    }
}

/// Handle key press for selection shortcuts
pub fn handle_selection_key_press(
    commands: &mut Commands,
    key: &KeyCode,
    modifiers: &crate::core::input::ModifierState,
    selectable_query: &Query<(Entity, &GlobalTransform, Option<&GlyphPointReference>, Option<&PointType>), With<Selectable>>,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    event_writer: &mut EventWriter<EditEvent>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    match key {
        KeyCode::KeyA => {
            if modifiers.ctrl {
                // Ctrl+A: Select all points in the active sort
                debug!("Select all shortcut triggered for active sort");
                let mut selected_count = 0;
                
                for (entity, _, _, _) in selectable_query.iter() {
                    // Check if this entity belongs to the active sort
                    if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
                        if sort_point_entity.sort_entity != active_sort_entity {
                            continue; // Skip points that don't belong to the active sort
                        }
                    } else {
                        continue; // Skip entities that aren't sort points
                    }

                    if !selection_state.selected.contains(&entity) {
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                        selected_count += 1;
                    }
                }
                
                event_writer.write(EditEvent {
                    edit_type: EditType::Normal,
                });
                debug!("Selected all {} points in active sort", selected_count);
            }
        }
        KeyCode::Escape => {
            // Escape: Clear selection
            debug!("Escape key pressed - clearing selection");
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            event_writer.write(EditEvent {
                edit_type: EditType::Normal,
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::editing::selection::coordinate_system::SelectionCoordinateSystem;
    use crate::ui::panes::design_space::DPoint;

    #[test]
    fn test_point_selection_matches_render_position() {
        let point_pos = Vec2::new(100.0, 200.0);
        let marquee_start = DPoint::from_raw(point_pos);
        let marquee_end = DPoint::from_raw(point_pos + Vec2::splat(1.0));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &point_pos,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_point_selection_matches_render_position] point={:?}, marquee=({:?}, {:?}), in_rect={}", point_pos, marquee_start, marquee_end, in_rect);
        assert!(in_rect, "Point should be inside the marquee rectangle");
    }

    #[test]
    fn test_parented_transform_selection() {
        // Simulate a point parented to a group at (50, 50)
        let parent_offset = Vec2::new(50.0, 50.0);
        let local_point = Vec2::new(10.0, 10.0);
        let world_point = parent_offset + local_point;
        let marquee_start = DPoint::from_raw(world_point);
        let marquee_end = DPoint::from_raw(world_point + Vec2::splat(1.0));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &world_point,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_parented_transform_selection] world_point={:?}, marquee=({:?}, {:?}), in_rect={}", world_point, marquee_start, marquee_end, in_rect);
        assert!(in_rect, "Parented point should be inside the marquee rectangle");
    }

    #[test]
    fn test_off_curve_point_selection() {
        // Simulate an off-curve point at a known position
        let off_curve_pos = Vec2::new(-123.4, 567.8);
        let marquee_start = DPoint::from_raw(off_curve_pos - Vec2::splat(0.5));
        let marquee_end = DPoint::from_raw(off_curve_pos + Vec2::splat(0.5));
        let in_rect = SelectionCoordinateSystem::is_point_in_rectangle(
            &off_curve_pos,
            &marquee_start,
            &marquee_end,
        );
        println!("[test_off_curve_point_selection] off_curve_pos={:?}, marquee=({:?}, {:?}), in_rect={}", off_curve_pos, marquee_start, marquee_end, in_rect);
        assert!(in_rect, "Off-curve point should be inside the marquee rectangle");
    }
} 