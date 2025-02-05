use crate::theme::*;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct PathPoint;

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

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

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

pub fn spawn_animated_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let texture = asset_server.load("raster/bevy/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    // Calculate position based on window size and margins
    let margin = 16.0;
    let sprite_size = 24.0 * 8.0;
    let x_pos = window.width() / 2.0 - margin - sprite_size / 2.0;
    let y_pos = -window.height() / 2.0 + margin + sprite_size / 2.0;

    println!("Window dimensions: {}x{}", window.width(), window.height());
    println!("Sprite position: ({}, {})", x_pos, y_pos);
    println!("Sprite size: {}", sprite_size);
    println!("Margin: {}", margin);

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_xyz(x_pos, y_pos, 2.0).with_scale(Vec3::splat(8.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

pub fn update_sprite_position(
    mut sprite_query: Query<&mut Transform, With<AnimationTimer>>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let margin = 32.0;
    let sprite_size = 24.0 * 6.0;

    for mut transform in &mut sprite_query {
        transform.translation.x = window.width() / 2.0 - margin - sprite_size / 2.0;
        transform.translation.y = -window.height() / 2.0 + margin + sprite_size / 2.0;
    }
}
