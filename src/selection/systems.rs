use super::components::*;
use crate::cameras::DesignCamera;
use crate::draw::AppStateChanged;
use crate::selection::nudge::EditEvent;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Constants for selection
const SELECTION_MARGIN: f32 = 10.0; // Distance in pixels for selection hit testing
const SELECT_POINT_RADIUS: f32 = 5.0; // Radius for drawing selection circle

// Resource to track the drag selection state
#[derive(Resource, Default)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection
    pub start_position: Option<Vec2>,
    /// The current position of the drag selection
    pub current_position: Option<Vec2>,
    /// Whether this is a multi-select operation (shift is held)
    pub is_multi_select: bool,
    /// The previous selection before the drag started
    pub previous_selection: Vec<Entity>,
}

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
) {
    // Early return if no window
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Early return if no camera
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

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
                    edit_type: crate::selection::nudge::EditType::AddPoint,
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
            edit_type: crate::selection::nudge::EditType::AddPoint,
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
) {
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
            edit_type: crate::selection::nudge::EditType::AddPoint,
        });
    }
}

/// System to update hover state based on mouse position
pub fn update_hover_state(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
    hovered_query: Query<Entity, With<Hovered>>,
) {
    // First, clear all hovered states to avoid inconsistencies
    for entity in &hovered_query {
        commands.entity(entity).remove::<Hovered>();
    }

    // Early return if no window
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Early return if no camera
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    // Get cursor position in world coordinates
    if let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok())
    {
        // Find closest entity to the cursor
        let mut closest_entity = None;
        let mut closest_distance = SELECTION_MARGIN;

        for (entity, transform) in selectable_query.iter() {
            let entity_pos = transform.translation().truncate();
            let distance = cursor_pos.distance(entity_pos);

            if distance < closest_distance {
                closest_distance = distance;
                closest_entity = Some(entity);
            }
        }

        // Add hover state only to the closest entity
        if let Some(entity) = closest_entity {
            // Check if the entity is already hovered to avoid unnecessary component updates
            if !hovered_query.contains(entity) {
                commands.entity(entity).insert(Hovered);
            }
        }
    }
}

/// System to draw the selection rectangle
pub fn render_selection_rect(
    mut gizmos: Gizmos,
    selection_rect_query: Query<&SelectionRect>,
) {
    for rect in &selection_rect_query {
        let rect_bounds = Rect::from_corners(rect.start, rect.end);
        let points = [
            Vec3::new(rect_bounds.min.x, rect_bounds.min.y, 0.0),
            Vec3::new(rect_bounds.max.x, rect_bounds.min.y, 0.0),
            Vec3::new(rect_bounds.max.x, rect_bounds.max.y, 0.0),
            Vec3::new(rect_bounds.min.x, rect_bounds.max.y, 0.0),
            Vec3::new(rect_bounds.min.x, rect_bounds.min.y, 0.0),
        ];

        // Draw the rectangle outline
        gizmos.linestrip(points, Color::WHITE);
    }
}

/// System to draw visual indicators for selected entities
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<&GlobalTransform, With<Selected>>,
) {
    for transform in &selected_query {
        let position = transform.translation().truncate();

        // Draw a circle around the selected point
        gizmos.circle_2d(
            position,
            SELECT_POINT_RADIUS * 1.5,
            Color::srgb(1.0, 1.0, 0.0), // Yellow
        );
    }
}

/// System to draw visual indicators for hovered entities
pub fn render_hovered_entities(
    mut gizmos: Gizmos,
    hovered_query: Query<&GlobalTransform, With<Hovered>>,
) {
    for transform in &hovered_query {
        let position = transform.translation().truncate();

        // Draw a circle around the hovered point
        gizmos.circle_2d(
            position,
            SELECT_POINT_RADIUS * 1.2,
            Color::srgba(0.3, 0.8, 1.0, 0.7),
        );
    }
}

/// System to clear selection when the app state changes
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
