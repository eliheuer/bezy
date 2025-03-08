// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::checkerboard::CheckerboardPlugin;
use crate::cli::CliArgs;
use crate::data::AppState;
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::edit_mode_toolbar::select::SelectModePlugin;
use crate::edit_mode_toolbar::CurrentEditMode;
use crate::edit_mode_toolbar::EditModeToolbarPlugin;
use crate::plugins::BezySystems; // Import BezySystems from the plugins module
use crate::selection::SelectionPlugin;
use crate::text_editor::TextEditorPlugin;
use crate::theme::BACKGROUND_COLOR;

// Create the app and add the plugins and systems
pub fn create_app(cli_args: CliArgs) -> App {
    // Initialize a custom logger that excludes timestamps but keeps colors
    crate::logger::init_custom_logger();
    let mut app = App::new();
    // Configure app with default settings
    configure_app_settings(&mut app, cli_args);
    // Add all plugins
    add_plugins(&mut app);
    app
}

// Helper function to configure app settings
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Set resource for CLI arguments
    app.init_resource::<AppState>()
        .insert_resource(cli_args)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CurrentEditMode::default());
}

// Add all necessary plugins
fn add_plugins(app: &mut App) {
    app.add_plugins(crate::plugins::configure_default_plugins())
        .add_plugins(PanCamPlugin)
        .add_plugins(CheckerboardPlugin)
        .add_plugins(DrawPlugin)
        .add_plugins(DesignSpacePlugin)
        .add_plugins(EditModeToolbarPlugin)
        .add_plugins(SelectModePlugin)
        .add_plugins(SelectionPlugin)
        .add_plugins(TextEditorPlugin)
        .add_plugins(BezySystems)
        .add_plugins(crate::commands::CommandsPlugin);
}
