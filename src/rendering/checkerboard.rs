//! Checkerboard background system for the Bezy font editor

use bevy::prelude::*;
use bevy_pancam::PanCam;

const CHECKERBOARD_SIZE: f32 = 100.0;

#[derive(Component)]
pub struct CheckerboardSquare;

pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_camera_move);
    }
}

fn on_camera_move(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<PanCam>>,
    square_query: Query<Entity, With<CheckerboardSquare>>,
) {
    if let Ok((camera, camera_transform)) = camera_query.single() {
        if let Some(viewport_rect) = camera.logical_viewport_rect() {
            let bottom_left = camera
                .viewport_to_world_2d(camera_transform, viewport_rect.min)
                .unwrap_or_default();
            let top_right = camera
                .viewport_to_world_2d(camera_transform, viewport_rect.max)
                .unwrap_or_default();

            for entity in square_query.iter() {
                commands.entity(entity).despawn();
            }

            let start_x = (bottom_left.x / CHECKERBOARD_SIZE).floor() as i32;
            let end_x = (top_right.x / CHECKERBOARD_SIZE).ceil() as i32;
            let start_y = (bottom_left.y / CHECKERBOARD_SIZE).floor() as i32;
            let end_y = (top_right.y / CHECKERBOARD_SIZE).ceil() as i32;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let color = if (x + y) % 2 != 0 {
                        Color::srgb(0.128, 0.128, 0.128)
                    } else {
                        Color::srgb(0.05, 0.05, 0.05)
                    };
                    let position =
                        Vec3::new(x as f32 * CHECKERBOARD_SIZE, y as f32 * CHECKERBOARD_SIZE, -10.0);
                    commands.spawn((
                        (
                            Sprite {
                                color,
                                custom_size: Some(Vec2::splat(CHECKERBOARD_SIZE)),
                                ..default()
                            },
                            Transform::from_translation(position),
                        ),
                        CheckerboardSquare,
                    ));
                }
            }
        }
    }
} 