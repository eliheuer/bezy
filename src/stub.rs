use bevy::prelude::*;
use rand::Rng;
use crate::theme::*;

// Component to mark our path points
#[derive(Component)]
pub struct PathPoint;

pub fn spawn_path_points(mut commands: Commands) {
    let mut rng = rand::thread_rng();

    // Generate random points within a reasonable area
    let points: Vec<(Entity, Vec2)> = (0..NUM_POINTS)
        .map(|_| {
            let x = rng.gen_range(-300.0..300.0);
            let y = rng.gen_range(-200.0..200.0);
            let position = Vec2::new(x, y);

            let entity = commands
                .spawn((
                    PathPoint,
                    Sprite {
                        color: PATH_COLOR,
                        custom_size: Some(Vec2::new(POINT_RADIUS * 2.0, POINT_RADIUS * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 1.0),
                    GlobalTransform::default(),
                ))
                .id();

            (entity, position)
        })
        .collect();

    // Create connections between points
    for i in 0..points.len() {
        let next_index = (i + 1) % points.len();

        // Spawn line connecting to next point
        let start = points[i].1;
        let end = points[next_index].1;
        let mid = (start + end) / 2.0;
        let distance = (end - start).length();
        let rotation = (end - start).y.atan2((end - start).x);

        commands.spawn((
            Sprite {
                color: PATH_COLOR,
                custom_size: Some(Vec2::new(distance, 2.0)),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, 0.0).with_rotation(Quat::from_rotation_z(rotation)),
            GlobalTransform::default(),
        ));
    }
}
