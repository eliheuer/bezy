use super::components::*;
use bevy::prelude::*;

/// A basic system that logs the number of selected and hovered entities
/// This is just a placeholder until we implement the full selection functionality
pub fn selection_visualization(
    selected_entities: Query<Entity, With<Selected>>,
    hovered_entities: Query<Entity, With<Hovered>>,
) {
    // Count the number of selected and hovered entities
    let selected_count = selected_entities.iter().count();
    let hovered_count = hovered_entities.iter().count();

    // Log the counts (just for debugging)
    if selected_count > 0 || hovered_count > 0 {
        debug!(
            "Selected entities: {}, Hovered entities: {}",
            selected_count, hovered_count
        );
    }
}

/// System to update hover state based on cursor position
pub fn update_hover_state(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    selectables: Query<(Entity, &GlobalTransform), With<Selectable>>,
    selection_state: Res<SelectionState>,
) {
    // Skip hover checks if we're doing a drag selection
    if selection_state.drag_selecting {
        return;
    }

    // Get the primary window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    // Get cursor position if available
    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Find cursor world position
    let mut cursor_world_pos = None;
    for (camera, camera_transform) in camera_q.iter() {
        if let Ok(pos) =
            camera.viewport_to_world(camera_transform, cursor_position)
        {
            cursor_world_pos = Some(pos);
            break;
        }
    }

    let cursor_world_pos = match cursor_world_pos {
        Some(pos) => pos,
        None => return,
    };

    // Ray position in world space
    let ray_pos_world = cursor_world_pos.origin.truncate();

    // Remove hover from all entities first
    for (entity, _) in selectables.iter() {
        commands.entity(entity).remove::<Hovered>();
    }

    // Check each selectable entity for hovering
    for (entity, transform) in selectables.iter() {
        let entity_pos = transform.translation().truncate();
        let distance = ray_pos_world.distance(entity_pos);

        // If close enough to the entity, mark as hovered
        if distance < 15.0 {
            // slightly larger threshold for hovering
            commands.entity(entity).insert(Hovered);
            break; // For now, we'll only hover one entity at a time
        }
    }
}

/// Marks entities as selected based on mouse input and selection state
pub fn mark_selected_entities(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    selectables: Query<(Entity, &GlobalTransform), With<Selectable>>,
    hovered: Query<Entity, With<Hovered>>,
) {
    // Only process if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Check if shift is held for multi-select
    let multi_select = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);
    selection_state.multi_select = multi_select;

    // Get the primary window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => {
            warn!("No window found for selection");
            return;
        }
    };

    // Get cursor position if available
    let cursor_position = if let Some(cursor_pos) = window.cursor_position() {
        cursor_pos
    } else {
        return;
    };

    // Find a camera that can be used for selection
    let mut cursor_world_pos = None;

    for (camera, camera_transform) in camera_q.iter() {
        if let Ok(pos) =
            camera.viewport_to_world(camera_transform, cursor_position)
        {
            cursor_world_pos = Some(pos);
            break;
        }
    }

    // If no camera could convert the cursor position, return
    let cursor_world_pos = match cursor_world_pos {
        Some(pos) => pos,
        None => {
            warn!("No camera found that can convert cursor position");
            return;
        }
    };

    // Ray position in world space
    let ray_pos_world = cursor_world_pos.origin.truncate();

    // If not in multi-select mode, clear previous selection
    if !multi_select {
        // Clear Selected component from all entities
        for entity in selectables.iter().map(|(e, _)| e) {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.clear();
    }

    // First try to select a hovered entity if any
    let mut selected_entity = false;

    if let Ok(entity) = hovered.get_single() {
        // Toggle selection if already selected in multi-select mode
        if multi_select && selection_state.selected_entities.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            // Remove from selection state
            selection_state.selected_entities.retain(|&e| e != entity);
        } else {
            commands.entity(entity).insert(Selected);
            selection_state.add_selected(entity);
        }
        selected_entity = true;
    }

    // If no hovered entity was selected, do a distance-based selection
    if !selected_entity {
        // Check each selectable entity
        for (entity, transform) in selectables.iter() {
            // Simple distance-based selection
            let entity_pos = transform.translation().truncate();
            let distance = ray_pos_world.distance(entity_pos);

            // If close enough to the entity, select it
            if distance < 10.0 {
                // 10 units threshold, adjust as needed
                // Toggle selection if already selected in multi-select mode
                if multi_select
                    && selection_state.selected_entities.contains(&entity)
                {
                    commands.entity(entity).remove::<Selected>();
                    // Remove from selection state
                    selection_state.selected_entities.retain(|&e| e != entity);
                } else {
                    commands.entity(entity).insert(Selected);
                    selection_state.add_selected(entity);
                }
                break; // For now, we'll select only one entity at a time
            }
        }
    }
}

/// System to handle the start of a drag selection
pub fn start_drag_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    select_rect_query: Query<Entity, With<SelectionRect>>,
    hovered_query: Query<Entity, With<Hovered>>,
    entities_with_selected: Query<Entity, With<Selected>>,
) {
    // Only process if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Skip if we clicked on a hovered entity
    if !hovered_query.is_empty() {
        return;
    }

    // Get the primary window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    // Get cursor position if available
    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Find cursor world position
    let mut cursor_world_pos = None;
    for (camera, camera_transform) in camera_q.iter() {
        if let Ok(pos) =
            camera.viewport_to_world(camera_transform, cursor_position)
        {
            cursor_world_pos = Some(pos);
            break;
        }
    }

    let cursor_world_pos = match cursor_world_pos {
        Some(pos) => pos,
        None => return,
    };

    // Ray position in world space
    let ray_pos_world = cursor_world_pos.origin.truncate();

    // Remove any existing selection rectangle
    for entity in select_rect_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create a new selection rectangle
    commands.spawn((SelectionRect {
        start: ray_pos_world,
        end: ray_pos_world,
    },));

    // Set drag selecting flag
    selection_state.drag_selecting = true;

    // Check if shift is held for multi-select
    let multi_select = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);
    selection_state.multi_select = multi_select;

    // If not in multi-select mode, clear previous selection
    if !multi_select {
        // Clear selected component from all entities
        for entity in entities_with_selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }

        selection_state.clear();
    }
}

/// System to update a drag selection in progress
pub fn update_drag_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    selection_state: ResMut<SelectionState>,
    mut select_rect_query: Query<&mut SelectionRect>,
) {
    // Only update if left mouse button is pressed and we're drag selecting
    if !mouse_button.pressed(MouseButton::Left)
        || !selection_state.drag_selecting
    {
        return;
    }

    // Get the primary window
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    // Get cursor position if available
    let cursor_position = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Find cursor world position
    let mut cursor_world_pos = None;
    for (camera, camera_transform) in camera_q.iter() {
        if let Ok(pos) =
            camera.viewport_to_world(camera_transform, cursor_position)
        {
            cursor_world_pos = Some(pos);
            break;
        }
    }

    let cursor_world_pos = match cursor_world_pos {
        Some(pos) => pos,
        None => return,
    };

    // Ray position in world space
    let ray_pos_world = cursor_world_pos.origin.truncate();

    // Update the selection rectangle end position
    if let Ok(mut select_rect) = select_rect_query.get_single_mut() {
        select_rect.end = ray_pos_world;
    }
}

/// System to debug selection status
pub fn debug_selection_state(
    selectables: Query<(Entity, &GlobalTransform), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
    selection_state: Res<SelectionState>,
) {
    if selection_state.is_changed() {
        let selectable_count = selectables.iter().count();
        let selected_count = selected.iter().count();

        info!(
            "Selection state changed: {} selectables, {} selected entities",
            selectable_count, selected_count
        );

        if selection_state.is_empty() {
            info!("Selection is empty");
        } else {
            info!(
                "Selection state selected_entities: {:?}",
                selection_state.selected_entities
            );
        }
    }
}

/// System to finish a drag selection
pub fn finish_drag_selection(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    select_rect_query: Query<(Entity, &SelectionRect)>,
    selectables: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    // Only process if left mouse button was just released and we're drag selecting
    if !mouse_button.just_released(MouseButton::Left)
        || !selection_state.drag_selecting
    {
        return;
    }

    // Get the selection rectangle
    let select_rect =
        if let Ok((entity, select_rect)) = select_rect_query.get_single() {
            // Cleanup the selection rectangle entity
            commands.entity(entity).despawn();
            select_rect
        } else {
            // Reset drag selecting flag
            selection_state.drag_selecting = false;
            return;
        };

    // Calculate the rectangle bounds
    let min_x = select_rect.start.x.min(select_rect.end.x);
    let max_x = select_rect.start.x.max(select_rect.end.x);
    let min_y = select_rect.start.y.min(select_rect.end.y);
    let max_y = select_rect.start.y.max(select_rect.end.y);

    // Minimum size to consider it a real selection (prevent accidental clicks)
    let min_size = 2.0; // Reduced from 5.0 to make selection easier

    // Only process if the rectangle is big enough
    if max_x - min_x > min_size || max_y - min_y > min_size {
        info!(
            "Selection rectangle: ({}, {}) to ({}, {})",
            min_x, min_y, max_x, max_y
        );
        let mut selected_count = 0;

        // Select all entities within the rectangle
        for (entity, transform) in selectables.iter() {
            let pos = transform.translation().truncate();

            // Check if the entity is within the selection rectangle
            if pos.x >= min_x
                && pos.x <= max_x
                && pos.y >= min_y
                && pos.y <= max_y
            {
                commands.entity(entity).insert(Selected);
                selection_state.add_selected(entity);
                selected_count += 1;
                info!(
                    "Selected entity {:?} at position ({}, {})",
                    entity, pos.x, pos.y
                );
            }
        }

        info!("Selected {} entities from drag selection", selected_count);
    } else {
        info!(
            "Selection rectangle too small: ({}, {}) to ({}, {})",
            min_x, min_y, max_x, max_y
        );
    }

    // Reset drag selecting flag
    selection_state.drag_selecting = false;
}

/// System to render the selection rectangle
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    select_rect_query: Query<&SelectionRect>,
) {
    if let Ok(select_rect) = select_rect_query.get_single() {
        // Calculate the rectangle bounds
        let min_x = select_rect.start.x.min(select_rect.end.x);
        let max_x = select_rect.start.x.max(select_rect.end.x);
        let min_y = select_rect.start.y.min(select_rect.end.y);
        let max_y = select_rect.start.y.max(select_rect.end.y);

        // Calculate center and size
        let center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        let size = Vec2::new(max_x - min_x, max_y - min_y);

        // Define bright orange/yellow color with full opacity for border
        let border_color = Color::srgba(1.0, 0.7, 0.0, 0.8);

        // Calculate corner positions
        let half_width = size.x / 2.0;
        let half_height = size.y / 2.0;

        let top_left = Vec2::new(center.x - half_width, center.y - half_height);
        let top_right =
            Vec2::new(center.x + half_width, center.y - half_height);
        let bottom_right =
            Vec2::new(center.x + half_width, center.y + half_height);
        let bottom_left =
            Vec2::new(center.x - half_width, center.y + half_height);

        // Draw dashed border lines
        let dash_length = 4.0;
        let gap_length = 4.0;

        // Helper function to draw dashed lines
        let mut draw_dashed_line = |start: Vec2, end: Vec2, color: Color| {
            let direction = (end - start).normalize();
            let distance = (end - start).length();
            let num_segments =
                (distance / (dash_length + gap_length)).ceil() as i32;

            for i in 0..num_segments {
                let segment_start =
                    start + direction * (i as f32 * (dash_length + gap_length));
                let segment_end = segment_start
                    + direction
                        * dash_length.min(
                            distance - (i as f32 * (dash_length + gap_length)),
                        );

                // Make sure we don't exceed the end point
                if segment_start.distance(start) <= distance {
                    gizmos.line_2d(segment_start, segment_end, color);
                }
            }
        };

        // Draw dashed borders
        draw_dashed_line(top_left, top_right, border_color);
        draw_dashed_line(top_right, bottom_right, border_color);
        draw_dashed_line(bottom_right, bottom_left, border_color);
        draw_dashed_line(bottom_left, top_left, border_color);
    }
}

/// System to handle keyboard shortcuts for selection
pub fn handle_selection_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    selectables: Query<Entity, With<Selectable>>,
) {
    // Select all (Ctrl+A)
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_pressed && keyboard.just_pressed(KeyCode::KeyA) {
        // Clear previous selection
        selection_state.clear();

        // Select all selectable entities
        for entity in selectables.iter() {
            commands.entity(entity).insert(Selected);
            selection_state.add_selected(entity);
        }

        info!("Selected all entities: {}", selection_state.count());
    }

    // Deselect all (Escape)
    if keyboard.just_pressed(KeyCode::Escape) {
        // Clear selection state
        selection_state.clear();

        // Remove Selected component from all entities
        for entity in selectables.iter() {
            commands.entity(entity).remove::<Selected>();
        }

        info!("Deselected all entities");
    }

    // Delete selected entities (Delete key)
    if keyboard.just_pressed(KeyCode::Delete)
        || keyboard.just_pressed(KeyCode::Backspace)
    {
        // Get all selected entities
        let selected = selection_state.selected_entities.clone();

        // Check if any entities are selected
        if !selected.is_empty() {
            info!("Deleting {} selected entities", selected.len());

            // For now, we'll just log the deletion
            // In a real implementation, you would actually delete the entities
            info!("Entities to delete: {:?}", selected);

            // Clear selection after deletion
            selection_state.clear();
        }
    }
}

/// System to render visual indicators for selected entities
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<
        (&GlobalTransform, Option<&PointType>),
        With<Selected>,
    >,
) {
    for (transform, point_type) in selected_query.iter() {
        let position = transform.translation().truncate();

        // Base selection color (orange-yellow)
        let selection_color = Color::srgba(1.0, 0.7, 0.2, 0.9);

        // Draw a circle to indicate selection
        gizmos.circle_2d(position, 14.0, selection_color);

        // Render differently based on point type
        match point_type {
            // Off-curve points (control points) - render as small circles
            Some(PointType::OffCurve) => {
                // Draw a hollow circle for off-curve points
                gizmos.circle_2d(position, 7.0, selection_color);

                // Small filled center
                gizmos.circle_2d(position, 2.5, selection_color);
            }

            // On-curve points or default - render as squares
            _ => {
                // Draw a square for on-curve points
                let square_size = 8.0;
                let half_size = square_size / 2.0;

                // Draw square using four lines
                let top_left =
                    Vec2::new(position.x - half_size, position.y - half_size);
                let top_right =
                    Vec2::new(position.x + half_size, position.y - half_size);
                let bottom_right =
                    Vec2::new(position.x + half_size, position.y + half_size);
                let bottom_left =
                    Vec2::new(position.x - half_size, position.y + half_size);

                gizmos.line_2d(top_left, top_right, selection_color);
                gizmos.line_2d(top_right, bottom_right, selection_color);
                gizmos.line_2d(bottom_right, bottom_left, selection_color);
                gizmos.line_2d(bottom_left, top_left, selection_color);

                // Draw a filled center
                gizmos.circle_2d(position, 3.0, selection_color);
            }
        }
    }
}

/// System to render visual indicators for hovered entities
pub fn render_hovered_entities(
    mut gizmos: Gizmos,
    hovered_query: Query<
        (&GlobalTransform, Option<&PointType>),
        (With<Hovered>, Without<Selected>),
    >,
) {
    for (transform, point_type) in hovered_query.iter() {
        let position = transform.translation().truncate();

        // Hover color (lighter orange)
        let hover_color = Color::srgba(1.0, 0.8, 0.4, 0.7);

        // Draw hover indicator
        gizmos.circle_2d(position, 16.0, hover_color);

        // Different hover indicator based on point type
        match point_type {
            // Off-curve points (control points)
            Some(PointType::OffCurve) => {
                // Draw a smaller hover indicator for off-curve points
                gizmos.circle_2d(position, 8.0, hover_color);
            }

            // On-curve points
            _ => {
                // Draw a square outline for on-curve points
                let square_size = 10.0;
                let half_size = square_size / 2.0;

                let top_left =
                    Vec2::new(position.x - half_size, position.y - half_size);
                let top_right =
                    Vec2::new(position.x + half_size, position.y - half_size);
                let bottom_right =
                    Vec2::new(position.x + half_size, position.y + half_size);
                let bottom_left =
                    Vec2::new(position.x - half_size, position.y + half_size);

                gizmos.line_2d(top_left, top_right, hover_color);
                gizmos.line_2d(top_right, bottom_right, hover_color);
                gizmos.line_2d(bottom_right, bottom_left, hover_color);
                gizmos.line_2d(bottom_left, top_left, hover_color);
            }
        }
    }
}
