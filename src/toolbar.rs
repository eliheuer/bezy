use crate::theme::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainToolbarButton;

#[derive(Component)]
pub struct ButtonName(pub String);

pub fn spawn_main_toolbar(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    // Load toolbar spritesheet
    let texture = asset_server.load("raster/icons/main-toolbar.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Spawn a container for the main toolbar buttons
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            for index in 0..8 {
                let button_name = match index {
                    0 => "Select",
                    1 => "Pen",
                    2 => "Hyper",
                    3 => "Knife",
                    4 => "Pan",
                    5 => "Measure",
                    6 => "Square",
                    7 => "Circle",
                    _ => "Unknown",
                };

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(4.0)),
                        ..default()
                    })
                    .with_children(|button_container| {
                        // Spawn the button
                        button_container
                            .spawn((
                                Button,
                                MainToolbarButton,
                                ButtonName(button_name.to_string()),
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    padding: UiRect::all(Val::Px(0.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor(Color::WHITE),
                                BorderRadius::all(Val::Px(BUTTON_BORDER_RADIUS)),
                                BackgroundColor(NORMAL_BUTTON),
                            ))
                            .with_child(Sprite::from_atlas_image(
                                texture.clone(),
                                TextureAtlas {
                                    layout: texture_atlas_layout.clone(),
                                    index,
                                },
                            ));

                        // Spawn the label text
                        button_container.spawn((
                            Text::new(button_name.to_string()),
                            TextFont {
                                font: asset_server.load("fonts/bezy-grotesk-regular.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn main_toolbar_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<MainToolbarButton>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }
    }
}
