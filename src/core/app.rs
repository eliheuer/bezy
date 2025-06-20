//! Application initialization and configuration
//!
//! This module creates and configures the main Bevy application
//! The main entry point is `create_app()` which takes CLI arguments

use bevy::prelude::*;
use crate::core::cli::CliArgs;
use crate::ui::theme::BACKGROUND_COLOR;

/// Creates a fully configured Bevy application ready to run
pub fn create_app(cli_args: CliArgs) -> App {
    let mut app = App::new();
    configure_app(&mut app, cli_args);
    app
}

/// Configures the application with all necessary plugins and systems
fn configure_app(app: &mut App, cli_args: CliArgs) {
    app.insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_resource::<crate::core::state::AppState>()
        .init_resource::<crate::core::state::GlyphNavigation>()
        .add_plugins((
            DefaultPlugins,
            crate::rendering::cameras::CameraPlugin,
            crate::rendering::checkerboard::CheckerboardPlugin,
            crate::editing::undo_plugin::UndoPlugin,
            crate::editing::sort_plugin::SortPlugin,
            crate::ui::panes::design_space::DesignSpacePlugin,
        ))
        .add_systems(Startup, move |mut app_state: ResMut<crate::core::state::AppState>| {
            load_ufo_font(&cli_args, &mut app_state);
        })
        .add_systems(Update, exit_on_esc);
}

// System to exit the application when the Escape key is pressed
fn exit_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

fn load_ufo_font(cli_args: &CliArgs, app_state: &mut ResMut<crate::core::state::AppState>) {
    // Use the UFO path from CLI args or default
    let path = cli_args.ufo_path.clone()
        .unwrap_or_else(|| std::path::PathBuf::from("assets/fonts/bezy-grotesk-regular.ufo"));
    
    match app_state.load_font_from_path(path.clone()) {
        Ok(_) => {
            info!("Successfully loaded UFO font from: {}", path.display());
        }
        Err(e) => {
            error!("Failed to load UFO font from {}: {}", path.display(), e);
        }
    }
}
