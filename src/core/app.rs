//! Application initialization and configuration
//!
//! This module creates and configures the main Bevy application
//! The main entry point is `create_app()` which takes CLI arguments

// ------------------------------------------------------------

// External dependencies - libraries we use from other crates
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

// Core application modules - fundamental app structure
use crate::core::cli::CliArgs;
use crate::core::state::{AppState, GlyphNavigation};

// UI and theming - visual appearance and user interface
use crate::ui::panes::DesignSpacePlugin;
use crate::ui::theme::BACKGROUND_COLOR;
use crate::ui::toolbars::edit_mode_toolbar::{
    select::SelectModePlugin, CurrentEditMode, EditModeToolbarPlugin,
};
use crate::ui::TextEditorPlugin;

// Editing functionality - tools for modifying fonts
use crate::editing::{EditSessionPlugin, SelectionPlugin, UndoPlugin};

// Rendering - drawing glyphs and visual elements
use crate::rendering::CheckerboardPlugin;

// System plugins - core app behavior and event handling
use crate::systems::plugins::*;
use crate::systems::{CommandsPlugin, UiInteractionPlugin};

// ------------------------------------------------------------

/// Creates a fully configured Bevy application ready to run
pub fn create_app(cli_args: CliArgs) -> App {
    crate::utils::logger::init_custom_logger();
    let mut app = App::new();
    configure_app_settings(&mut app, cli_args);
    add_all_plugins(&mut app);
    app
}

// ------------------------------------------------------------

/// Sets up application resources and configuration
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Create GlyphNavigation with the codepoint from CLI args
    let glyph_navigation =
        GlyphNavigation::new(Some(cli_args.load_unicode.clone()));
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
        PanCamPlugin,       // Camera controls
        CheckerboardPlugin, // Background grid
        // DrawPlugin now handled by BezySystems bundle
    ));
}

/// Adds plugins for editor UI and interaction
fn add_editor_plugins(app: &mut App) {
    app.add_plugins((
        DesignSpacePlugin,     // Main design area
        EditModeToolbarPlugin, // Mode switching toolbar
        SelectModePlugin,      // Selection tool
        EditSessionPlugin,     // Edit session management
        SelectionPlugin,       // Selection handling
        TextEditorPlugin,      // Text input
        UiInteractionPlugin,   // UI event handling
    ));
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        BezySystems,    // Main app systems
        CommandsPlugin, // Command handling
        UndoPlugin,     // Undo/redo system
    ));
}