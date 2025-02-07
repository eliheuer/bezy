use crate::theme::*;
use crate::ufo::get_basic_font_info;
use bevy::prelude::*;

#[derive(Component)]
pub struct PathPoint;

fn _green_text(text: String) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}

pub fn spawn_path_points(mut commands: Commands) {
    // Define a simple path with fixed points
    let points = vec![
        Vec2::new(-8.0, -8.0),
        Vec2::new(-8.0, 8.0),
        Vec2::new(8.0, 8.0),
        Vec2::new(8.0, -8.0),
    ];

    // Spawn points
    let point_entities: Vec<(Entity, Vec2)> = points
        .into_iter()
        .map(|position| {
            let entity = commands
                .spawn((
                    PathPoint,
                    Sprite {
                        color: PATH_COLOR,
                        custom_size: Some(Vec2::new(POINT_RADIUS * 1.0, POINT_RADIUS * 1.0)),
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
    for i in 0..point_entities.len() {
        let next_index = (i + 1) % point_entities.len();
        let start = point_entities[i].1;
        let end = point_entities[next_index].1;
        let mid = (start + end) / 2.0;
        let distance = (end - start).length();
        let rotation = (end - start).y.atan2((end - start).x);

        commands.spawn((
            Sprite {
                color: PATH_COLOR,
                custom_size: Some(Vec2::new(distance, 1.0)),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, 0.0).with_rotation(Quat::from_rotation_z(rotation)),
            GlobalTransform::default(),
        ));
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
    commands.spawn((
        Text::new("أشهد يا إلهي"),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 64.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            right: Val::Px(32.0),
            ..default()
        },
    ));
}
