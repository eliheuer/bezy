//! Application initialization and configuration module
//!
//! This module is responsible for creating and configuring the Bevy application.
//! It sets up all resources, plugins, and initial state for the Bezy font editor.
//! For customization and styling, see theme.rs, settings.rs, and plugins.rs.
//!
//! The main function to use from this module is `create_app()`, which takes command line
//! arguments and returns a configured Bevy application ready to be run.
//!
//! # Architecture
//! This file follows a modular design pattern where:
//! - Application creation is handled by `create_app()`
//! - Resource configuration is handled by `configure_app_settings()`
//! - Plugin registration is handled by `add_plugins()`

// External crates
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

// Internal modules - grouped by functionality

// Core app functionality
use crate::cli::CliArgs;
use crate::data::AppState;
use crate::theme::BACKGROUND_COLOR;

// Plugins
use crate::checkerboard::CheckerboardPlugin;
use crate::commands::CommandsPlugin;
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::edit_session::EditSessionPlugin;
use crate::plugins::*;
use crate::selection::SelectionPlugin;
use crate::text_editor::TextEditorPlugin;
use crate::ui_interaction::UiInteractionPlugin;
use crate::undo_plugin::UndoPlugin;

// Toolbars, widgets, and gizmos
use crate::edit_mode_toolbar::{
    select::SelectModePlugin, CurrentEditMode, EditModeToolbarPlugin,
};

/// Creates and configures a new Bevy application with all required plugins and settings.
/// This is the main entry point for the Bezy font editor application.
/// Returns a configured Bevy `App` ready to be run.
pub fn create_app(cli_args: CliArgs) -> App {
    // Initialize a custom logger, see logger.rs for details
    crate::logger::init_custom_logger();
    // Create a new Bevy app instance
    let mut app = App::new();
    // Configure app with default settings and CLI arguments
    configure_app_settings(&mut app, cli_args);
    // Adds all plugins to the application
    add_plugins(&mut app);
    // Return the fully configured app from the create_app function
    app
}

/// Helper function to configure app settings and resources
/// This includes setting up:
/// - Application state (see data.rs for AppState implementation)
/// - CLI arguments for command-line control
/// - Window settings for interaction behavior
/// - Default clear color for the background
/// - Edit mode for the current editing context
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Initialize the application state - this contains the workspace, selection state, etc.
    app.init_resource::<AppState>()
        // Make CLI arguments available to all systems
        .insert_resource(cli_args)
        // Configure window system to work well for desktop app usage
        .insert_resource(WinitSettings::desktop_app())
        // Set the background color from our theme settings
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // Initialize with default edit mode (usually "select" mode)
        .insert_resource(CurrentEditMode::default());
}

/// Add all necessary plugins to the application
/// Bevy uses a plugin-based architecture where each plugin adds specific functionality.
/// Plugins are grouped logically to make it clear what each section does.
fn add_plugins(app: &mut App) {
    app.add_plugins(crate::plugins::configure_default_plugins())
        // ---- Rendering and View Plugins ----
        .add_plugins((
            // Camera panning and zooming functionality
            PanCamPlugin,
            // Background grid pattern for the design view
            CheckerboardPlugin,
            // Glyph drawing and visualization
            DrawPlugin,
        ))
        // ---- Editor UI Plugins ----
        .add_plugins((
            // Design space visualization and manipulation
            DesignSpacePlugin,
            // Toolbar for switching between editing modes
            EditModeToolbarPlugin,
            // Selection mode implementation
            SelectModePlugin,
            // Edit session management
            EditSessionPlugin,
            // Selection management (handling selected points, paths, etc.)
            SelectionPlugin,
            // Text input and editing functionality
            TextEditorPlugin,
            // UI interaction detection
            UiInteractionPlugin,
        ))
        // ---- Core Application Logic ----
        .add_plugins((
            // Main application systems bundle (defined in plugins.rs)
            BezySystems,
            // Command handling for user actions
            CommandsPlugin,
            // Undo/redo system
            UndoPlugin,
        ));
}
