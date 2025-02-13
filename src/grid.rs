use crate::cameras::{DesignCamera, GRID_LAYER};
// use crate::debug::green_text;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

// Component to mark grid lines
#[derive(Component)]
pub struct GridLine;

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

// System to update grid based on camera state
pub fn update_grid(
    mut commands: Commands,
    grid_settings: Res<GridSettings>,
    grid_query: Query<Entity, With<GridLine>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    // Remove existing grid lines
    for entity in grid_query.iter() {
        commands.entity(entity).despawn();
    }

    if !grid_settings.visible {
        return;
    }

    let (_camera, camera_transform) = camera_query.single();
    let window = windows.single();

    const BASE_GRID_SPACING: f32 = 1.0;
    let camera_scale = camera_transform.compute_transform().scale.x;

    // Match the reference implementation's grid spacing calculation exactly
    let grid_spacing = if camera_scale < 1.0 {
        // When zoomed out, increase the spacing
        BASE_GRID_SPACING * (1.0 / camera_scale).ceil()
    } else {
        BASE_GRID_SPACING
    };

    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    // Calculate visible area in world space
    let visible_width = window_width / camera_scale;
    let visible_height = window_height / camera_scale;

    // Get camera position in world space
    let camera_pos = camera_transform.translation().truncate();

    // Calculate grid boundaries in world space
    let x_start = (camera_pos.x - visible_width / 2.0).floor();
    let x_end = (camera_pos.x + visible_width / 2.0).ceil();
    let y_start = (camera_pos.y - visible_height / 2.0).floor();
    let y_end = (camera_pos.y + visible_height / 2.0).ceil();

    // Fixed opacity for debugging
    let grid_fade = 0.3;

    // Draw vertical lines
    let mut x = x_start;
    while x <= x_end {
        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, grid_fade),
                custom_size: Some(Vec2::new(0.5, visible_height)),
                ..default()
            },
            Transform::from_xyz(x, camera_pos.y, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        x += grid_spacing;
    }

    // Draw horizontal lines
    let mut y = y_start;
    while y <= y_end {
        commands.spawn((
            GridLine,
            Sprite {
                color: Color::srgba(0.5, 0.5, 0.5, grid_fade),
                custom_size: Some(Vec2::new(visible_width, 0.5)),
                ..default()
            },
            Transform::from_xyz(camera_pos.x, y, 0.0),
            RenderLayers::layer(GRID_LAYER),
        ));

        y += grid_spacing;
    }
}
