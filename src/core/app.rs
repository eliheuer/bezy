//! Application initialization and configuration
//!
//! This module creates and configures the main Bevy application for the Bezy font editor.
//! The main entry point is `create_app()` which takes CLI arguments and returns a configured app.

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

// Internal imports
use crate::core::cli::CliArgs;
use crate::core::data::AppState;
use crate::ui::theme::BACKGROUND_COLOR;

// Editor plugins
use crate::editing::{EditSessionPlugin, SelectionPlugin, UndoPlugin};
use crate::rendering::{CheckerboardPlugin, DrawPlugin};
use crate::systems::plugins::*;
use crate::systems::{CommandsPlugin, UiInteractionPlugin};
use crate::ui::panes::DesignSpacePlugin;
use crate::ui::TextEditorPlugin;
use crate::ui::toolbars::edit_mode_toolbar::{
    select::SelectModePlugin, CurrentEditMode, EditModeToolbarPlugin,
};

/// Creates a fully configured Bevy application ready to run
pub fn create_app(cli_args: CliArgs) -> App {
    crate::utils::logger::init_custom_logger();
    let mut app = App::new();
    configure_app_settings(&mut app, cli_args);
    add_all_plugins(&mut app);
    app
}

/// Sets up application resources and configuration
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Initialize glyph navigation with the CLI codepoint value
    let glyph_navigation = crate::core::data::GlyphNavigation::new(cli_args.load_unicode.clone());
    
    app.init_resource::<AppState>()
        .insert_resource(cli_args)
        .insert_resource(glyph_navigation)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CurrentEditMode::default());
}

/// Adds all plugins to the application in logical groups
fn add_all_plugins(app: &mut App) {
    app.add_plugins(crate::systems::plugins::configure_default_plugins());
    add_rendering_plugins(app);
    add_editor_plugins(app);
    add_core_plugins(app);
}

/// Adds plugins for rendering and visual display
fn add_rendering_plugins(app: &mut App) {
    app.add_plugins((
        PanCamPlugin,        // Camera controls
        CheckerboardPlugin,  // Background grid
        DrawPlugin,          // Glyph rendering
    ));
}

/// Adds plugins for editor UI and interaction
fn add_editor_plugins(app: &mut App) {
    app.add_plugins((
        DesignSpacePlugin,      // Main design area
        EditModeToolbarPlugin,  // Mode switching toolbar
        SelectModePlugin,       // Selection tool
        EditSessionPlugin,      // Edit session management
        SelectionPlugin,        // Selection handling
        TextEditorPlugin,       // Text input
        UiInteractionPlugin,    // UI event handling
    ));
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        BezySystems,     // Main app systems
        CommandsPlugin,  // Command handling
        UndoPlugin,      // Undo/redo system
    ));
}
