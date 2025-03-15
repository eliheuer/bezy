use crate::selection::components::Selected;
use bevy::prelude::*;
use crate::edit_session::EditSession;
use crate::edit_type::EditType;
use crate::undo_plugin::UndoStateResource;
use std::sync::Arc;

/// The amount to nudge by in each direction (in design units)
const NUDGE_AMOUNT: f32 = 1.0;
/// The amount to nudge when shift is held (for larger movements)
const SHIFT_NUDGE_AMOUNT: f32 = 10.0;
/// The amount to nudge when command/ctrl is held (for even larger movements)
const CMD_NUDGE_AMOUNT: f32 = 100.0;

/// Resource to track if we're currently in a nudging operation
#[derive(Resource, Debug, Default)]
pub struct NudgeState {
    /// Whether we're currently nudging (to prevent selection loss)
    pub is_nudging: bool,
    /// Timestamp of the last nudge operation
    pub last_nudge_time: f32,
    /// The last key that was pressed for nudging
    pub last_key_pressed: Option<KeyCode>,
}

/// Component to track the last edit type for undo purposes
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct LastEditType {
    pub edit_type: Option<EditType>,
}

/// Event to signal that an edit has been made that should be added to the undo stack
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub struct EditEvent {
    pub edit_type: EditType,
}

/// Component to track point coordinates in font space
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PointCoordinates {
    pub position: Vec2,
}

/// System to handle keyboard input for nudging selected points
pub fn handle_nudge_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (Entity, &mut Transform, &mut PointCoordinates),
        With<Selected>,
    >,
    mut event_writer: EventWriter<EditEvent>,
    mut nudge_state: ResMut<NudgeState>,
    time: Res<Time>,
    mut edit_sessions: Query<&mut EditSession>,
) {
    // Store the pressed key if any
    let pressed_key = if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        Some(KeyCode::ArrowLeft)
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        Some(KeyCode::ArrowRight)
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        Some(KeyCode::ArrowUp)
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        Some(KeyCode::ArrowDown)
    } else {
        None
    };

    // Early return if no arrow key is pressed
    if pressed_key.is_none() {
        // Check if a previously pressed key was released
        if let Some(last_key) = nudge_state.last_key_pressed {
            if !keyboard_input.pressed(last_key) {
                // Reset the last key pressed but keep nudging state active
                nudge_state.last_key_pressed = None;
            }
        }
        return;
    }

    // Update the last key pressed
    nudge_state.last_key_pressed = pressed_key;

    // Calculate nudge amount based on modifiers
    let mut amount = NUDGE_AMOUNT;
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);
    let cmd_pressed = keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight)
        || keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    if cmd_pressed {
        amount = CMD_NUDGE_AMOUNT;
    } else if shift_pressed {
        amount = SHIFT_NUDGE_AMOUNT;
    }

    // Determine direction and edit type based on the pressed key
    let (direction, edit_type) = match pressed_key.unwrap() {
        KeyCode::ArrowLeft => {
            (Vec3::new(-amount, 0.0, 0.0), EditType::NudgeLeft)
        }
        KeyCode::ArrowRight => {
            (Vec3::new(amount, 0.0, 0.0), EditType::NudgeRight)
        }
        KeyCode::ArrowUp => (Vec3::new(0.0, amount, 0.0), EditType::NudgeUp),
        KeyCode::ArrowDown => {
            (Vec3::new(0.0, -amount, 0.0), EditType::NudgeDown)
        }
        _ => return, // This should never happen due to our earlier check
    };

    // Only proceed if we have selected points to nudge
    let count = query.iter().count();
    if count == 0 {
        return;
    }

    // Mark that we're in a nudging operation
    nudge_state.is_nudging = true;
    nudge_state.last_nudge_time = time.elapsed_secs();
    
    // Get the current edit session if available
    let mut session_updated = false;
    if let Ok(mut session) = edit_sessions.get_single_mut() {
        session_updated = true;
    }

    // Apply nudge to all selected entities
    for (entity, mut transform, mut coordinates) in &mut query {
        // Update the transform to move the entity
        transform.translation += direction;

        // Also update the point coordinates to keep in sync
        coordinates.position.x = transform.translation.x;
        coordinates.position.y = transform.translation.y;
        
        // Directly update the EditSession for this entity
        if session_updated {
            if let Ok(mut session) = edit_sessions.get_single_mut() {
                session.point_positions.insert(
                    entity,
                    Vec2::new(transform.translation.x, transform.translation.y)
                );
            }
        }

        debug!(
            "Nudged entity {:?} to position {:?}",
            entity, transform.translation
        );
    }

    // Log the nudge operation
    info!("Nudged {} points by {:?}", count, direction);

    // Send edit event for undo system
    event_writer.send(EditEvent { edit_type });
}

/// System to reset the nudging state after a delay
/// This ensures we don't permanently block selection changes
pub fn reset_nudge_state(mut nudge_state: ResMut<NudgeState>, time: Res<Time>) {
    // Reset nudging state after 2.0 seconds of no nudging activity
    if nudge_state.is_nudging
        && time.elapsed_secs() - nudge_state.last_nudge_time > 2.0
    {
        nudge_state.is_nudging = false;
    }
}

/// System to ensure transform and point coordinates stay in sync
pub fn sync_transforms_and_coordinates(
    mut query: Query<(&Transform, &mut PointCoordinates), Changed<Transform>>,
) {
    for (transform, mut coords) in &mut query {
        // Only update if there's a significant difference to avoid unnecessary updates
        if (coords.position.x - transform.translation.x).abs() > 0.001
            || (coords.position.y - transform.translation.y).abs() > 0.001
        {
            coords.position.x = transform.translation.x;
            coords.position.y = transform.translation.y;
            debug!(
                "Synced point coordinates to transform: {:?}",
                coords.position
            );
        }
    }
}

/// System to handle edit events for the undo stack
#[allow(clippy::type_complexity)]
pub fn handle_edit_events(
    mut events: EventReader<EditEvent>,
    mut undo_state: ResMut<UndoStateResource>,
    edit_session_entities: Query<Entity, With<EditSession>>,
    edit_session_query: Query<&EditSession>,
) {
    // Log if we have any events
    let event_count = events.len();
    if event_count > 0 {
        info!("Processing {} edit events", event_count);
    }
    
    // Create a default EditSession for undo purposes
    let default_session = EditSession::default();
    
    for event in events.read() {
        let edit_type = event.edit_type;
        
        info!("Received edit event: {:?}", edit_type);
        
        // Create a snapshot of the current state
        // Try to find an EditSession entity first
        let session = if let Some(entity) = edit_session_entities.iter().next() {
            // Try to get the EditSession component from the entity
            if let Ok(session) = edit_session_query.get(entity) {
                info!("Found EditSession entity: {:?}", entity);
                // Make sure to clone the session to get a true snapshot
                let session_clone = session.clone();
                
                // Create a snapshot right now (deferred to after all systems) 
                // We'll use this snapshot for the undo stack
                let snapshot = Arc::new(session_clone);
                
                // Check if we need a new undo group based on the edit type
                if let Some(last_edit_type) = undo_state.last_edit_type() {
                    let needs_new_group = last_edit_type.needs_new_undo_group(edit_type);
                    
                    if needs_new_group {
                        // Start a new undo group
                        info!("Creating new undo group for edit type: {:?}", edit_type);
                        let stack_size_before = undo_state.stack_size();
                        let stack_index_before = undo_state.current_index();
                        undo_state.push_undo_state(snapshot);
                        let stack_size_after = undo_state.stack_size();
                        let stack_index_after = undo_state.current_index();
                        info!("Undo stack changed: size {}→{}, index {}→{}", 
                              stack_size_before, stack_size_after,
                              stack_index_before, stack_index_after);
                    } else {
                        // Update the current undo group
                        info!("Updating current undo group for edit type: {:?}", edit_type);
                        let stack_index_before = undo_state.current_index();
                        undo_state.update_current_undo(snapshot);
                        let stack_index_after = undo_state.current_index();
                        info!("Updated undo group at index {} (now {})", 
                              stack_index_before, stack_index_after);
                    }
                } else {
                    // This is the first edit, so create a new undo group
                    info!("Creating first undo group for edit type: {:?}", edit_type);
                    let stack_size_before = undo_state.stack_size();
                    let stack_index_before = undo_state.current_index();
                    undo_state.push_undo_state(snapshot);
                    let stack_size_after = undo_state.stack_size();
                    let stack_index_after = undo_state.current_index();
                    info!("Undo stack initialized: size {}→{}, index {}→{}", 
                          stack_size_before, stack_size_after,
                          stack_index_before, stack_index_after);
                }
                
                // Update the last edit type
                undo_state.set_last_edit_type(edit_type);
                
                session.clone()
            } else {
                warn!("Found EditSession entity but couldn't get component, using default");
                default_session.clone()
            }
        } else {
            warn!("No EditSession found, using default");
            default_session.clone()
        };
    }
}

/// Plugin to set up nudging functionality
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EditEvent>()
            .register_type::<EditType>()
            .register_type::<LastEditType>()
            .register_type::<PointCoordinates>()
            .init_resource::<NudgeState>()
            .add_systems(
                Update,
                (
                    handle_nudge_shortcuts,
                    reset_nudge_state,
                    sync_transforms_and_coordinates,
                    handle_edit_events,
                ),
            );
    }
}
