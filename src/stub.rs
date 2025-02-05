use crate::theme::*;
use anyhow::Result;
use bevy::prelude::*;
use norad::Font as Ufo;
use rand::Rng;
use std::path::PathBuf;

#[derive(Component)]
pub struct PathPoint;

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

fn green(text: String) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}

pub fn spawn_path_points(mut commands: Commands) {
    let mut rng = rand::thread_rng();

    // Generate random points within a reasonable area
    let points: Vec<(Entity, Vec2)> = (0..NUM_POINTS)
        .map(|_| {
            let x = rng.gen_range(-512.0..512.0);
            let y = rng.gen_range(-256.0..256.0);
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
    let x_pos = window.width() / 2.0;
    let y_pos = -window.height() / 2.0;

    println!(
        "{}",
        green(format!(
            "Window dimensions: {}x{}",
            window.width(),
            window.height()
        ))
    );
    println!(
        "{}",
        green(format!("Sprite position: ({}, {})", x_pos, y_pos))
    );

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
        ),
        Transform::from_xyz(x_pos, y_pos, 2.0).with_scale(Vec3::splat(16.0)),
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

pub fn load_ufo() {
    match try_load_ufo() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            println!(
                "Successfully loaded UFO font: {} {}",
                family_name, style_name
            );
        }
        Err(e) => eprintln!("Error loading UFO file: {:?}", e),
    }
}

fn try_load_ufo() -> Result<Ufo> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let font_path = manifest_dir.join("assets/fonts/bezy-grotesk-regular.ufo");
    let ufo = Ufo::load(font_path)?;
    Ok(ufo)
}

fn get_basic_font_info() -> String {
    match try_load_ufo() {
        Ok(ufo) => {
            let family_name = ufo.font_info.family_name.unwrap_or_default();
            let style_name = ufo.font_info.style_name.unwrap_or_default();
            format!("{} {}", family_name, style_name)
        }
        Err(e) => format!("Error loading font: {:?}", e),
    }
}

pub fn spawn_debug_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new(get_basic_font_info()),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(32.0),
            ..default()
        },
    ));
}
