use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub position: Vec2,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            zoom: 1.0,
            position: Vec2::ZERO,
        }
    }
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
        camera_state.zoom = new_zoom.clamp(0.1, 10.0);

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
    let speed = 500.0; // Adjust this value for pan speed
    let delta = time.delta_secs();
    let mut direction = Vec2::ZERO;

    // Handle keyboard input
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Update camera position
    if direction != Vec2::ZERO {
        let movement = direction.normalize() * speed * delta;
        camera_state.position += movement;
    }

    // Apply transformation to all cameras
    for mut transform in &mut query {
        transform.translation = Vec3::new(camera_state.position.x, camera_state.position.y, 0.0);
    }
}
