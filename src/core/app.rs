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
use crate::core::settings::BezySettings;

// Systems and utilities
use crate::systems::{UiInteractionPlugin, CommandsPlugin, BezySystems};

// UI and theming - visual appearance and user interface
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::panes::coord_pane::CoordinatePanePlugin;
use crate::ui::toolbars::EditModeToolbarPlugin;
use crate::ui::hud::HudPlugin;
use crate::ui::theme::BACKGROUND_COLOR;

// Editing functionality - tools for modifying fonts
use crate::editing::{SelectionPlugin, TextEditorPlugin, UndoPlugin};

// Rendering - drawing glyphs and visual elements
use crate::rendering::{
    cameras::CameraPlugin,
    checkerboard::CheckerboardPlugin,
};

// ------------------------------------------------------------

/// Creates a fully configured Bevy application ready to run
/// 
/// This function sets up all resources, plugins, and systems needed for
/// the font editor. It validates CLI arguments before proceeding.
pub fn create_app(cli_args: CliArgs) -> Result<App, String> {
    // Validate CLI arguments first
    cli_args.validate()?;
    
    let mut app = App::new();
    configure_app_settings(&mut app, cli_args);
    add_all_plugins(&mut app);
    
    Ok(app)
}

// ------------------------------------------------------------

/// Sets up application resources and configuration
/// 
/// This function initializes all the resources needed by the font editor,
/// including settings, state, and Bevy configuration.
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    // Initialize core resources
    let glyph_navigation = GlyphNavigation::default();
    let settings = BezySettings::default();
    
    app.init_resource::<AppState>()
        .insert_resource(cli_args)
        .insert_resource(glyph_navigation)
        .insert_resource(settings)
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
        GlyphPanePlugin,        // Glyph information panel
        CoordinatePanePlugin,   // Coordinate display panel
        EditModeToolbarPlugin,  // Edit mode toolbar
        HudPlugin,              // HUD management
    ));
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        SelectionPlugin,        // Selection handling and events
                    TextEditorPlugin,       // Text editor-based sort functionality
        UndoPlugin,             // Undo/redo system
        UiInteractionPlugin,    // UI hover detection
        CommandsPlugin,         // Command system for file operations and shortcuts
        BezySystems,            // Core Bezy systems
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
/// 
/// This system runs once at startup to load the UFO font specified in the CLI arguments.
/// It provides detailed error messages if the font cannot be loaded.
fn load_ufo_font(cli_args: Res<CliArgs>, mut app_state: ResMut<AppState>) {
    // clap provides the default value, so ufo_path is guaranteed to be Some
    if let Some(path) = &cli_args.ufo_path {
        match app_state.load_font_from_path(path.clone()) {
            Ok(_) => {
                info!("Successfully loaded UFO font from: {}", path.display());
            }
            Err(e) => {
                error!("Failed to load UFO font: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");
            }
        }
    } else {
        warn!("No UFO font path specified, running without a font loaded.");
    }
}
