// A debug display for the edit mode toolbar state

use crate::data::AppState;
use crate::theme::get_default_text_style;
use crate::theme::DEFAULT_FONT_PATH;
use crate::ufo::get_basic_font_info_from_state;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

#[derive(Component)]
pub struct FontInfoText;

#[derive(Component)]
pub struct CodepointNotFoundText;

// Resource to track if we've already created the warning text
#[derive(Resource, Default)]
pub struct WarningTextState {
    pub created: bool,
}

// Resource to track the last printed codepoint information
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct LastCodepointPrinted {
    pub codepoint: Option<String>,
}

pub fn update_font_info_text(
    app_state: Res<AppState>,
    mut query: Query<&mut Text, With<FontInfoText>>,
    cli_args: Res<crate::cli::CliArgs>,
) {
    for mut text in query.iter_mut() {
        let font_info = crate::ufo::get_basic_font_info_from_state(&app_state);
        let mut display_text = font_info;

        // Add codepoint info if present
        if let Some(codepoint) = &cli_args.test_unicode {
            if !codepoint.is_empty() {
                // Try to get a readable character representation
                let cp_value = match u32::from_str_radix(
                    codepoint.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(value) => value,
                    Err(_) => 0,
                };

                let char_display = match char::from_u32(cp_value) {
                    Some(c) if c.is_control() => format!("<control>"),
                    Some(c) => format!("'{}'", c),
                    None => format!("<none>"),
                };

                display_text.push_str(&format!(
                    " | Codepoint: U+{} {}",
                    codepoint, char_display
                ));
            }
        }

        text.0 = display_text;
    }
}

// New system to print font info and codepoint to terminal
pub fn print_font_info_to_terminal(
    app_state: Res<AppState>,
    cli_args: Res<crate::cli::CliArgs>,
    mut last_printed: ResMut<LastCodepointPrinted>,
) {
    let font_info = crate::ufo::get_basic_font_info_from_state(&app_state);
    let mut display_text = font_info;
    let current_codepoint = cli_args.test_unicode.clone();

    // Check if we need to print (startup or codepoint changed)
    let should_print = last_printed.codepoint != current_codepoint;

    if should_print {
        // Add codepoint info if present
        if let Some(codepoint) = &cli_args.test_unicode {
            if !codepoint.is_empty() {
                // Try to get a readable character representation
                let cp_value = match u32::from_str_radix(
                    codepoint.trim_start_matches("0x"),
                    16,
                ) {
                    Ok(value) => value,
                    Err(_) => 0,
                };

                let char_display = match char::from_u32(cp_value) {
                    Some(c) if c.is_control() => format!("<control>"),
                    Some(c) => format!("'{}'", c),
                    None => format!("<none>"),
                };

                display_text.push_str(&format!(
                    " | Codepoint: U+{} {}",
                    codepoint, char_display
                ));
            }
        }

        // Print the info to the terminal
        info!("{}", display_text);

        // Update the last printed codepoint
        last_printed.codepoint = current_codepoint;
    }
}

pub fn update_codepoint_not_found_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    text_query: Query<Entity, With<CodepointNotFoundText>>,
    cli_args: Res<crate::cli::CliArgs>,
    mut warning_state: ResMut<WarningTextState>,
) {
    // Only proceed if test_unicode flag was specified
    if let Some(test_unicode) = &cli_args.test_unicode {
        if !test_unicode.is_empty() && !cli_args.codepoint_found {
            // Only create the warning text once
            if !warning_state.created && text_query.is_empty() {
                commands.spawn((
                    CodepointNotFoundText,
                    Text::new(format!(
                        "Codepoint {} not found in UFO source",
                        test_unicode
                    )),
                    TextFont {
                        font: asset_server.load(DEFAULT_FONT_PATH),
                        font_size: 96.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.2, 0.0)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Auto,
                        bottom: Val::Auto,
                        left: Val::Auto,
                        right: Val::Auto,
                        margin: UiRect {
                            top: Val::Percent(32.0),
                            left: Val::Px(32.0),
                            ..default()
                        },
                        ..default()
                    },
                    Visibility::Visible,
                    RenderLayers::layer(1), // UI layer
                ));
                warning_state.created = true;
            }
        } else {
            // If codepoint was found, clean up any existing warning text
            for entity in text_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            warning_state.created = false;
        }
    } else if !text_query.is_empty() {
        // If no test_unicode, clean up any existing warning text
        for entity in text_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        warning_state.created = false;
    }
}

pub fn spawn_debug_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Res<AppState>,
) {
    // Initialize the warning text state
    commands.init_resource::<WarningTextState>();

    // Initialize the last printed codepoint tracking resource
    commands.init_resource::<LastCodepointPrinted>();

    // The FontInfoText component has been removed to eliminate the UFO and codepoint display
    // in the lower left hand side of the application

    // PUA icon display for debug/testing
    if std::env::var("BEZY_DEBUG_ICONS").is_ok() {
        commands.spawn((
            Text::new("\u{E000}"),
            TextFont {
                font: asset_server.load(DEFAULT_FONT_PATH),
                font_size: 256.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(32.0),
                ..default()
            },
            RenderLayers::layer(1), // UI layer
        ));
    }
}
