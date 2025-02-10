use crate::camera::CameraState;
// use crate::debug::green_text;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

// Component to mark grid lines
#[derive(Component)]
pub struct GridLine;

// Component to mark the grid camera
#[derive(Component)]
pub struct GridCamera;

// Resource to track grid visibility
#[derive(Resource)]
pub struct GridSettings {
    pub visible: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self { visible: true }
    }
}

// System to toggle grid visibility
pub fn toggle_grid(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut grid_settings: ResMut<GridSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        grid_settings.visible = !grid_settings.visible;
    }
}

// Calculate the appropriate grid spacing based on zoom level
fn calculate_grid_spacing(zoom: f32) -> f32 {
    let base_spacing = 8.0;
    let zoom_factor = 1.0 / zoom;

    // Adjust spacing based on zoom, maintaining power of 2 intervals
    if zoom_factor > 8.0 {
        base_spacing * 8.0
    } else if zoom_factor > 4.0 {
        base_spacing * 4.0
    } else if zoom_factor > 2.0 {
        base_spacing * 2.0
    } else if zoom_factor > 1.0 {
        base_spacing
    } else {
        base_spacing / 2.0
    }
}

// Define a constant for the grid render layer
const GRID_LAYER: usize = 1;

// System to update grid based on camera state
pub fn update_grid(
    mut commands: Commands,
    camera_state: Res<CameraState>,
    grid_settings: Res<GridSettings>,
    grid_query: Query<Entity, Or<(With<GridLine>, With<GridCamera>)>>,
    windows: Query<&Window>,
) {
    // Remove existing grid lines and camera
    for entity in grid_query.iter() {
        commands.entity(entity).despawn();
    }

    if !grid_settings.visible {
        return;
    }

    let window = windows.single();
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    let grid_spacing = calculate_grid_spacing(camera_state.zoom);

    // Calculate the visible range in world coordinates
    let half_width = window_width / (2.0 * camera_state.zoom);
    let half_height = window_height / (2.0 * camera_state.zoom);

    // Calculate grid boundaries in world space
    let min_x = camera_state.position.x - half_width;
    let max_x = camera_state.position.x + half_width;
    let min_y = camera_state.position.y - half_height;
    let max_y = camera_state.position.y + half_height;

    // Calculate grid line positions
    let start_x = (min_x / grid_spacing).floor() * grid_spacing;
    let end_x = (max_x / grid_spacing).ceil() * grid_spacing;
    let start_y = (min_y / grid_spacing).floor() * grid_spacing;
    let end_y = (max_y / grid_spacing).ceil() * grid_spacing;

    // Create a camera for the grid
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        RenderLayers::layer(GRID_LAYER),
        GridCamera,
    ));

    // Spawn vertical lines
    let mut x = start_x;
    while x <= end_x {
        let screen_x = (x - camera_state.position.x) * camera_state.zoom + window_width / 2.0;

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                custom_size: Some(Vec2::new(1.0, window_height)),
                ..default()
            },
            Transform::from_xyz(screen_x - window_width / 2.0, 0.0, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        x += grid_spacing;
    }

    // Spawn horizontal lines
    let mut y = start_y;
    while y <= end_y {
        let screen_y = (y - camera_state.position.y) * camera_state.zoom + window_height / 2.0;

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                custom_size: Some(Vec2::new(window_width, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, screen_y - window_height / 2.0, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        y += grid_spacing;
    }
}
