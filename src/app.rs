//! Application initialization and configuration module
//!
//! This module is responsible for creating and configuring the Bevy application.
//! It sets up all resources, plugins, and initial state for the Bezy font editor.
//!
//! The main function to use from this module is `create_app()`, which takes command line
//! arguments and returns a configured Bevy application ready to be run.
//!
//! # Architecture
//! This file follows a modular design pattern where:
//! - Application creation is handled by `create_app()`
//! - Resource configuration is handled by `configure_app_settings()`
//! - Plugin registration is handled by `add_plugins()`
//!
//! This separation of concerns makes the code more maintainable and easier to understand.

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
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::edit_mode_toolbar::{
    select::SelectModePlugin, CurrentEditMode, EditModeToolbarPlugin,
};
use crate::plugins::*;
use crate::selection::SelectionPlugin;
use crate::text_editor::TextEditorPlugin;

/// Creates and configures a new Bevy application with all required plugins and settings.
///
/// This is the main entry point for the Bezy font editor application.
///
/// # Arguments
/// * `cli_args` - Command line arguments parsed into a `CliArgs` struct
///
/// # Returns
/// A configured Bevy `App` ready to be run
pub fn create_app(cli_args: CliArgs) -> App {
    // Initialize a custom logger that excludes timestamps but keeps colors
    // See logger.rs for details on how this works
    crate::logger::init_custom_logger();

    // Create a new Bevy app instance
    let mut app = App::new();

    // Configure app with default settings and CLI arguments
    configure_app_settings(&mut app, cli_args);

    // Add all required plugins to the application
    add_plugins(&mut app);

    app
}

/// Helper function to configure app settings and resources
///
/// This includes setting up:
/// - Application state (see data.rs for AppState implementation)
/// - CLI arguments for command-line control
/// - Window settings for interaction behavior
/// - Default clear color for the background
/// - Edit mode for the current editing context
///
/// # Arguments
/// * `app` - The Bevy App being configured
/// * `cli_args` - Command line arguments from the user
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
///
/// Bevy uses a plugin-based architecture where each plugin adds specific functionality.
/// Plugins are grouped logically to make it clear what each section does.
///
/// # Arguments
/// * `app` - The Bevy App to add plugins to
fn add_plugins(app: &mut App) {
    app
        // Add core Bevy plugins with our custom window configuration
        // (This disables Bevy's default logger since we use our own)
        .add_plugins(crate::plugins::configure_default_plugins())
        // ---- Rendering and View Plugins ----
        // These plugins handle the visual representation and camera control
        .add_plugins((
            // Camera panning and zooming functionality
            PanCamPlugin,
            // Background grid pattern for the design view
            CheckerboardPlugin,
            // Glyph drawing and visualization
            DrawPlugin,
        ))
        // ---- Editor UI Plugins ----
        // These plugins provide the user interface for editing
        .add_plugins((
            // Design space visualization and manipulation
            DesignSpacePlugin,
            // Toolbar for switching between editing modes
            EditModeToolbarPlugin,
            // Selection mode implementation
            SelectModePlugin,
            // Selection management (handling selected points, paths, etc.)
            SelectionPlugin,
            // Text input and editing functionality
            TextEditorPlugin,
        ))
        // ---- Core Application Logic ----
        // These plugins handle the application's main systems and behavior
        .add_plugins((
            // Main application systems bundle (defined in plugins.rs)
            BezySystems,
            // Command handling for user actions
            crate::commands::CommandsPlugin,
        ));
}
