//! Application initialization and configuration
//!
//! This module creates and configures the main Bevy application
//! The main entry point is `create_app()` which takes CLI arguments

// ------------------------------------------------------------

// External dependencies - libraries we use from other crates
use bevy::prelude::*;
use bevy::winit::WinitSettings;

// Core application modules - fundamental app structure
use crate::core::cli::CliArgs;
use crate::core::state::{AppState, GlyphNavigation};

// UI and theming - visual appearance and user interface
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::theme::BACKGROUND_COLOR;

// Editing functionality - tools for modifying fonts
use crate::editing::{sort_plugin::SortPlugin, undo_plugin::UndoPlugin};

// Rendering - drawing glyphs and visual elements
use crate::rendering::{
    cameras::CameraPlugin,
    checkerboard::CheckerboardPlugin,
};

// ------------------------------------------------------------

/// Creates a fully configured Bevy application ready to run
pub fn create_app(cli_args: CliArgs) -> App {
    let mut app = App::new();
    configure_app_settings(&mut app, cli_args);
    add_all_plugins(&mut app);
    app
}

// ------------------------------------------------------------

/// Sets up application resources and configuration
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Create GlyphNavigation - for now we'll use a default since
    // current CLI doesn't have load_unicode
    let glyph_navigation = GlyphNavigation::default();
    
    app.init_resource::<AppState>()
        .insert_resource(cli_args)
        .insert_resource(glyph_navigation)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR));
}

/// Adds all plugins to the application in logical groups
fn add_all_plugins(app: &mut App) {
    app.add_plugins(DefaultPlugins);
    add_rendering_plugins(app);
    add_editor_plugins(app);
    add_core_plugins(app);
    add_startup_systems(app);
}

/// Adds startup and update systems
fn add_startup_systems(app: &mut App) {
    app.add_systems(Startup, load_ufo_font)
        .add_systems(Update, exit_on_esc);
}

/// Adds plugins for rendering and visual display
fn add_rendering_plugins(app: &mut App) {
    app.add_plugins((
        CameraPlugin,           // Our camera setup (includes PanCam)
        CheckerboardPlugin,     // Background grid
    ));
}

/// Adds plugins for editor UI and interaction
fn add_editor_plugins(app: &mut App) {
    app.add_plugins((
        DesignSpacePlugin,      // Main design area
        // Note: Other UI plugins like EditModeToolbarPlugin,
        // TextEditorPlugin, etc. are not included as they don't
        // exist in the current codebase
    ));
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        SortPlugin,             // Sort functionality
        UndoPlugin,             // Undo/redo system
        // Note: BezySystems, CommandsPlugin, etc. are not included
        // as they don't exist in the current codebase
    ));
}

// ------------------------------------------------------------

/// System to exit the application when the Escape key is pressed
fn exit_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

/// System to load UFO font on startup
fn load_ufo_font(cli_args: Res<CliArgs>, mut app_state: ResMut<AppState>) {
    // clap provides the default value, so ufo_path is guaranteed to be Some
    if let Some(path) = &cli_args.ufo_path {
        match app_state.load_font_from_path(path.clone()) {
            Ok(_) => {
                info!("Successfully loaded UFO font from: {}", path.display());
            }
            Err(e) => {
                error!("Failed to load UFO font from {}: {}", path.display(), e);
            }
        }
    }
}
