use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub struct DesignCamera;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct GridCamera;

#[derive(Component)]
pub struct CoordinateDisplay;

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub position: Vec2,
    pub bounds: Vec2, // Represents the maximum distance from origin in both directions
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            zoom: 1.0,
            position: Vec2::ZERO,
            bounds: Vec2::new(1024.0, 1024.0), // Typical font design space bounds
        }
    }
}

// Main camera for the design space (bezier curves, points, etc.)
pub fn spawn_design_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(0),
        DesignCamera, // Custom component to identify this camera
    ));
}

// Grid camera that renders underneath the design space
pub fn spawn_grid_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        RenderLayers::layer(1),
        GridCamera,
    ));
}

// UI camera for toolbars and overlays
pub fn spawn_ui_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        RenderLayers::layer(2),
        UiCamera,
    ));
}

pub fn handle_camera_zoom(
    mut camera_state: ResMut<CameraState>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for scroll_event in scroll_events.read() {
        // Adjust zoom speed/sensitivity here
        let zoom_delta = scroll_event.y * 0.1;
        let new_zoom = camera_state.zoom + zoom_delta;
        // Clamp sets the upper and lower bounds of the zoom level
        camera_state.zoom = new_zoom.clamp(0.01, 50.0);

        for mut transform in &mut query {
            transform.scale = Vec3::splat(camera_state.zoom);
        }
    }
}

pub fn handle_camera_pan(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let speed = 500.0;
    let delta = time.delta_secs();
    let mut direction = Vec2::ZERO;

    // Handle keyboard input
    if keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::ArrowUp)
    {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::ArrowDown)
    {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::ArrowLeft)
    {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD)
        || keyboard_input.pressed(KeyCode::ArrowRight)
    {
        direction.x += 1.0;
    }

    // Update camera position with bounds checking
    if direction != Vec2::ZERO {
        let movement = direction.normalize() * speed * delta;
        let new_position = camera_state.position + movement;

        // Clamp the new position within bounds
        camera_state.position = Vec2::new(
            new_position
                .x
                .clamp(-camera_state.bounds.x, camera_state.bounds.x),
            new_position
                .y
                .clamp(-camera_state.bounds.y, camera_state.bounds.y),
        );
    }

    // Apply transformation to all cameras
    for mut transform in &mut query {
        transform.translation =
            Vec3::new(camera_state.position.x, camera_state.position.y, 0.0);
    }
}

// Define a constant for the grid render layer
pub const GRID_LAYER: usize = 1;

pub fn update_coordinate_display(
    camera_state: Res<CameraState>,
    mut query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    for mut text in &mut query {
        text.0 = format!(
            "X: {:.1}, Y: {:.1}",
            camera_state.position.x, camera_state.position.y
        );
    }
}
