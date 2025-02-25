use super::EditModeSystem;
use bevy::prelude::*;
use std::time::Duration;
use crate::toolbar::{CurrentEditMode, EditMode};

/// Component to mark an entity as selected
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Selected;

/// Resource to store the current selection state
#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected_points: Vec<Entity>,
}

/// Resource for tracking debug timer
#[derive(Resource)]
pub struct SelectionDebugTimer(pub Timer);

impl Default for SelectionDebugTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(2), TimerMode::Repeating))
    }
}

/// Plugin that adds selection functionality to the app
pub struct SelectPlugin;

impl Plugin for SelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
           .init_resource::<SelectionDebugTimer>();
    }
}

/// System that periodically logs debug information about the selection state
pub fn debug_selection_state(
    time: Res<Time>,
    mut timer: ResMut<SelectionDebugTimer>,
    selection_state: Res<SelectionState>,
    selected_query: Query<Entity, With<Selected>>,
    current_mode: Res<CurrentEditMode>,
) {
    // Only run when Select mode is active
    if current_mode.0 != EditMode::Select {
        return;
    }

    if timer.0.tick(time.delta()).just_finished() {
        let selected_count = selected_query.iter().count();
        let state_count = selection_state.selected_points.len();
        
        if selected_count > 0 || state_count > 0 {
            info!("Selection debug: {} entities with Selected component, {} in SelectionState", 
                 selected_count, state_count);
            
            if selected_count != state_count {
                warn!("Selection count mismatch: {} Selected components vs {} in SelectionState", 
                     selected_count, state_count);
            }
            
            for entity in &selection_state.selected_points {
                if !selected_query.contains(*entity) {
                    warn!("Entity {:?} is in SelectionState but missing Selected component", entity);
                }
            }
            
            for entity in selected_query.iter() {
                if !selection_state.selected_points.contains(&entity) {
                    warn!("Entity {:?} has Selected component but not in SelectionState", entity);
                }
            }
        }
    }
}

/// System to debug camera information
pub fn debug_camera_info(
    time: Res<Time>,
    mut timer: Local<Timer>,
    cameras: Query<(Entity, &Camera, Option<&Name>, Option<&bevy::render::camera::OrthographicProjection>)>,
    pan_cams: Query<Entity, With<bevy_pancam::PanCam>>,
    current_mode: Res<CurrentEditMode>,
) {
    // Only run when Select mode is active
    if current_mode.0 != EditMode::Select {
        return;
    }

    // Initialize timer on first run
    if timer.duration() == Duration::ZERO {
        *timer = Timer::new(Duration::from_secs(5), TimerMode::Repeating);
    }
    
    // Check cameras every 5 seconds
    if timer.tick(time.delta()).just_finished() {
        info!("=== Camera Debug Information ===");
        info!("Number of cameras found: {}", cameras.iter().count());
        info!("Number of PanCam cameras: {}", pan_cams.iter().count());
        
        // Log PanCam entities
        for (i, entity) in pan_cams.iter().enumerate() {
            info!("PanCam #{}: Entity {:?}", i, entity);
        }
        
        for (i, (entity, camera, name, projection)) in cameras.iter().enumerate() {
            let camera_name = name.map_or("Unnamed".to_string(), |n| n.to_string());
            let is_orthographic = projection.is_some();
            let is_pan_cam = pan_cams.contains(entity);
            
            info!("Camera #{}: Entity {:?}, Name: '{}', Is Orthographic: {}, Is PanCam: {}", 
                  i, entity, camera_name, is_orthographic, is_pan_cam);
            info!("   Is Active: {}, Target: {:?}", 
                  camera.is_active, camera.target);
        }
        
        if cameras.iter().count() == 0 {
            error!("No cameras found in the scene! The selection system requires a camera.");
        }
    }
}

/// System to debug entity information in the scene
pub fn debug_scene_entities(
    time: Res<Time>,
    mut timer: Local<Timer>,
    transforms: Query<(Entity, &GlobalTransform, Option<&Name>)>,
    current_mode: Res<CurrentEditMode>,
) {
    // Only run when Select mode is active
    if current_mode.0 != EditMode::Select {
        return;
    }

    // Initialize timer on first run
    if timer.duration() == Duration::ZERO {
        *timer = Timer::new(Duration::from_secs(10), TimerMode::Repeating);
    }
    
    // Check entities every 10 seconds
    if timer.tick(time.delta()).just_finished() {
        info!("=== Scene Entity Debug Information ===");
        info!("Total entities with transforms: {}", transforms.iter().count());
        
        // Only log details for first 10 entities to avoid flooding the console
        let mut count = 0;
        for (entity, transform, name) in transforms.iter().take(10) {
            let position = transform.translation().truncate();
            let entity_name = name.map_or("Unnamed".to_string(), |n| n.to_string());
            
            info!("Entity #{}: {:?}, Name: '{}', Position: {:?}", 
                  count, entity, entity_name, position);
            count += 1;
        }
        
        if transforms.iter().count() > 10 {
            info!("... and {} more entities", transforms.iter().count() - 10);
        }
    }
}

pub struct SelectMode;

impl EditModeSystem for SelectMode {
    fn update(&self, commands: &mut Commands) {
        // Implementation for select mode update
        debug!("Select mode update");
        
        // This is where we would schedule systems specific to the select mode
        // But since our selection systems are already registered in the MainToolbarPlugin,
        // we don't need to add them here again
    }

    fn on_enter(&self) {
        info!("Select mode activated - click on points to select them");
    }

    fn on_exit(&self) {
        info!("Exiting select mode");
    }
}

/// System that handles mouse clicks for selecting points
pub fn select_point_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    camera_2d: Query<(&Camera, &GlobalTransform), With<bevy::render::camera::OrthographicProjection>>,
    pan_cam: Query<(&Camera, &GlobalTransform), With<bevy_pancam::PanCam>>,
    transforms: Query<(Entity, &GlobalTransform, Option<&Name>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    current_mode: Res<CurrentEditMode>,
) {
    // Only run when Select mode is active
    if current_mode.0 != EditMode::Select {
        return;
    }

    // Only handle left mouse button clicks
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    info!("Left mouse button clicked - checking for point selection");
    
    // Get the window and cursor position
    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => {
            error!("No window found");
            return;
        },
    };
    
    let cursor_position = match window.cursor_position() {
        Some(position) => {
            info!("Cursor position: {:?}", position);
            position
        },
        None => {
            info!("Cursor not in window");
            return; // Cursor not in window
        },
    };

    // Try to get the camera in order of preference:
    // 1. PanCam (most likely in your app)
    // 2. Orthographic camera
    // 3. Any camera
    let (camera, camera_transform) = match pan_cam.get_single() {
        Ok(camera) => {
            info!("Found PanCam camera");
            camera
        },
        Err(_) => match camera_2d.get_single() {
            Ok(camera) => {
                info!("Found 2D orthographic camera");
                camera
            },
            Err(_) => {
                // Fall back to any camera if we can't find a specialized one
                match cameras.get_single() {
                    Ok(camera) => {
                        info!("Found generic camera");
                        camera
                    },
                    Err(_) => {
                        error!("No camera found. Available cameras: {}", cameras.iter().count());
                        error!("Your app uses bevy_pancam, but no PanCam component was found.");
                        
                        // List available camera entities for debugging
                        for (i, (camera, _)) in cameras.iter().enumerate() {
                            info!("Camera {}: {:?}", i, camera);
                        }
                        return;
                    },
                }
            },
        },
    };

    // Convert screen coordinates to world coordinates
    let world_position = camera
        .viewport_to_world(camera_transform, cursor_position)
        .map(|ray| ray.origin.truncate())
        .unwrap_or_default();
    
    info!("World position: {:?}", world_position);

    // Define selection distance threshold (in world units)
    const SELECTION_THRESHOLD: f32 = 20.0; // Increased for better detection
    
    // Find the closest point to the click
    let mut closest_point = None;
    let mut closest_distance = SELECTION_THRESHOLD;

    info!("Total entities to check: {}", transforms.iter().count());
    
    for (entity, transform, name) in transforms.iter() {
        let point_position = transform.translation().truncate();
        let distance = world_position.distance(point_position);
        
        if distance < closest_distance {
            closest_distance = distance;
            closest_point = Some(entity);
            if let Some(name) = name {
                info!("New closest entity found: {:?} ({}) at distance {}", entity, name, distance);
            } else {
                info!("New closest entity found: {:?} at distance {}", entity, distance);
            }
        }
    }

    // Check if shift is held (for multi-select)
    let shift_held = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if shift_held {
        info!("Shift key is held - using multi-select mode");
    }

    // Handle the selection
    if let Some(entity) = closest_point {
        info!("Selected entity: {:?} at distance {}", entity, closest_distance);
        
        if shift_held {
            // Toggle selection state for this point
            if selection_state.selected_points.contains(&entity) {
                info!("Deselecting point: {:?}", entity);
                selection_state.selected_points.retain(|e| *e != entity);
                commands.entity(entity).remove::<Selected>();
            } else {
                info!("Adding point to selection: {:?}", entity);
                selection_state.selected_points.push(entity);
                commands.entity(entity).insert(Selected);
            }
        } else {
            // Single selection - clear previous selection
            if !selection_state.selected_points.is_empty() {
                info!("Clearing previous selection of {} points", selection_state.selected_points.len());
                for selected_entity in &selection_state.selected_points {
                    commands.entity(*selected_entity).remove::<Selected>();
                }
            }
            
            // Select only this point
            selection_state.selected_points.clear();
            selection_state.selected_points.push(entity);
            commands.entity(entity).insert(Selected);
            info!("Selected single point: {:?}", entity);
        }
    } else {
        info!("No point found within threshold distance {}", SELECTION_THRESHOLD);
        
        if !shift_held {
            // Clicked on empty space without shift - clear selection
            if !selection_state.selected_points.is_empty() {
                info!("Clearing selection of {} points", selection_state.selected_points.len());
                for selected_entity in &selection_state.selected_points {
                    commands.entity(*selected_entity).remove::<Selected>();
                }
                selection_state.selected_points.clear();
            }
        }
    }
    
    info!("Current selection state: {:?} points selected", selection_state.selected_points.len());
}

/// System that draws selected points with a different visual style
pub fn draw_selected_points_system(
    selected_query: Query<Entity, With<Selected>>,
    transform_query: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
    current_mode: Res<CurrentEditMode>,
) {
    // Only run when Select mode is active
    if current_mode.0 != EditMode::Select {
        return;
    }

    // Use a very distinctive color for selected points
    let selected_color = Color::rgb(1.0, 0.0, 0.0); // Bright red for maximum visibility
    let outline_color = Color::rgb(1.0, 1.0, 1.0); // White outline for contrast
    let selected_count = selected_query.iter().count();
    
    if selected_count > 0 {
        info!("Drawing {} selected points", selected_count);
        
        // Log all selected entities
        for (i, entity) in selected_query.iter().enumerate() {
            info!("Selected entity #{}: {:?}", i, entity);
        }
    }
    
    for entity in selected_query.iter() {
        match transform_query.get(entity) {
            Ok(transform) => {
                let position = transform.translation().truncate();
                info!("Drawing selected point at position: {:?}", position);
                
                // Draw multiple shapes at different Z depths to ensure visibility
                
                // Draw multiple concentric circles with alternating colors
                let sizes = [15.0, 12.0, 9.0, 6.0, 3.0];
                let colors = [outline_color, selected_color, outline_color, selected_color, outline_color];
                
                for (size, color) in sizes.iter().zip(colors.iter()) {
                    gizmos.circle_2d(position, *size, *color);
                }
                
                // Draw a cross
                let cross_size = 20.0;
                gizmos.line_2d(
                    Vec2::new(position.x - cross_size, position.y),
                    Vec2::new(position.x + cross_size, position.y),
                    outline_color
                );
                gizmos.line_2d(
                    Vec2::new(position.x, position.y - cross_size),
                    Vec2::new(position.x, position.y + cross_size),
                    outline_color
                );
            },
            Err(err) => {
                error!("Failed to get transform for selected entity {:?}: {:?}", entity, err);
                
                // Try to provide more context about the entity
                error!("This entity might be missing a GlobalTransform component");
            }
        }
    }
}
