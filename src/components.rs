use crate::theme::*;
use anyhow::Result;
use bevy::prelude::*;
use norad::Font as Ufo;
use rand::Rng;

// Component to mark our path points
#[derive(Component)]
pub struct PathPoint;

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

/// Spawns a grid centered in the window.
/// Creates both vertical and horizontal lines with semi-transparent gray color.
pub fn spawn_grid(mut commands: Commands) {
    // Get window dimensions (using a larger value to ensure coverage)
    let window_width = 2048.0; // Doubled from window width
    let window_height = 1536.0; // Doubled from window height
    let grid_position = Vec2::new(0.0, 0.0); // Center of the window

    // Create vertical lines
    for i in -512..=512 {
        // Increased range
        let x = grid_position.x + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(1.0, window_height)),
                ..default()
            },
            Transform::from_xyz(x * 32.0, grid_position.y, 0.0),
        ));
    }

    // Create horizontal lines
    for i in -512..=512 {
        // Increased range
        let y = grid_position.y + (i as f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.9, 0.9, 0.1),
                custom_size: Some(Vec2::new(window_width, 1.0)),
                ..default()
            },
            Transform::from_xyz(grid_position.x, y * 32.0, 0.0),
        ));
    }
}

pub fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "P".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::Hovered => {
                **text = "H".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "B".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
        }
    }
}

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
) {
    let texture = asset_server.load("raster/bevy/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_xyz(0.0, -200.0, 2.0).with_scale(Vec3::splat(12.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
