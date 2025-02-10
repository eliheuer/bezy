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

    const BASE_GRID_SPACING: f32 = 1.0;

    // Match the reference implementation's grid spacing calculation exactly
    let grid_spacing = if camera_state.zoom < 1.0 {
        // When zoomed out, increase the spacing
        BASE_GRID_SPACING * (1.0 / camera_state.zoom).ceil()
    } else {
        BASE_GRID_SPACING
    };

    let window = windows.single();
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    // Calculate visible area in design space
    let visible_width = window_width * camera_state.zoom;
    let visible_height = window_height * camera_state.zoom;

    // Calculate grid boundaries in design space
    let x_start = (camera_state.position.x - visible_width / 2.0).floor();
    let x_end = (camera_state.position.x + visible_width / 2.0).ceil();
    let y_start = (camera_state.position.y - visible_height / 2.0).floor();
    let y_end = (camera_state.position.y + visible_height / 2.0).ceil();

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

    // Fixed opacity for debugging
    let grid_fade = 0.3;

    // Draw vertical lines
    let mut x = x_start;
    while x <= x_end {
        let screen_x = (x - camera_state.position.x) / camera_state.zoom + window_width / 2.0;

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, grid_fade),
                custom_size: Some(Vec2::new(0.5, window_height)),
                ..default()
            },
            Transform::from_xyz(screen_x - window_width / 2.0, 0.0, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        x += grid_spacing;
    }

    // Draw horizontal lines
    let mut y = y_start;
    while y <= y_end {
        let screen_y = (y - camera_state.position.y) / camera_state.zoom + window_height / 2.0;

        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, grid_fade),
                custom_size: Some(Vec2::new(window_width, 0.5)),
                ..default()
            },
            Transform::from_xyz(0.0, screen_y - window_height / 2.0, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        y += grid_spacing;
    }
}
