use crate::stub::load_ufo;
use crate::theme::*;
use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    load_ufo();

    // Load toolbar spritesheet
    let texture = asset_server.load("raster/icons/main-toolbar.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Spawn UI camera
    commands.spawn(Camera2d);

    /*     // Temporary text for debugging
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
    */
    // Spawn a container for the main toolbar buttons in the upper left corner.
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
            for index in 0..8 {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            padding: UiRect::all(Val::Px(0.0)),
                            margin: UiRect::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_child((
                        Sprite::from_atlas_image(
                            texture.clone(),
                            TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index,
                            },
                        ),
                        Transform::from_scale(Vec3::splat(1.0)),
                    ));
            }
        });

    commands.spawn((
        Text::new("\u{E000}"),
        TextFont {
            font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
            font_size: 768.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(-256.0),
            right: Val::Px(16.0),
            ..default()
        },
    ));
}
