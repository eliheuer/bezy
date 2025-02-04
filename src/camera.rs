use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState { zoom: 1.0 }
    }
}

pub fn camera_zoom(
    mut camera_state: ResMut<CameraState>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for ev in scroll_evr.read() {
        // Adjust zoom speed/sensitivity here
        let zoom_delta = ev.y * 0.1;
        let new_zoom = camera_state.zoom + zoom_delta;
        camera_state.zoom = new_zoom.clamp(0.1, 10.0);

        for mut transform in &mut query {
            transform.scale = Vec3::splat(camera_state.zoom);
        }
    }
}
