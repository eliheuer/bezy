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

/// Initial setup system that runs on startup.
/// Spawns the UI camera and creates the font info text display.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    load_ufo();

    // Spawn UI camera
    commands.spawn(Camera2d);

    // Spawn your font info text (unchanged)
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

    // Spawn a container for the buttons in the upper left corner.
    // We set its flex direction to Row so its children are arranged horizontally.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(32.0),
            // Ensure horizontal layout
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            // Create 5 buttons showing the letters "A" through "E"
            // Each button is square (60x60 pixels) with a bit of margin.
            for letter in ["A", "B", "C", "D", "E"] {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            margin: UiRect::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BorderRadius::all(Val::Px(0.0)),
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_child((
                        Text::new(letter),
                        TextFont {
                            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
            }
        });
}

/// Loads and validates the UFO font file, printing status to console.
/// Currently loads a test font from the design-assets directory.
fn load_ufo() {
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

/// Attempts to load the UFO font file and returns a Result.
/// Returns Ok(Ufo) if successful, or an Error if loading fails.
fn try_load_ufo() -> Result<Ufo> {
    let path = "assets/fonts/bezy-grotesk-regular.ufo";
    let ufo = Ufo::load(path)?;
    Ok(ufo)
}

/// Retrieves basic font information as a formatted string.
/// Returns a string containing the font family name and style,
/// or an error message if the font cannot be loaded.
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
