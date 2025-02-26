// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::cli::CliArgs;
use crate::data::AppState;
use crate::debug_hud::{
    spawn_debug_text, spawn_main_toolbar_debug, update_main_toolbar_debug, update_font_info_text,
};
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::main_toolbar::MainToolbarPlugin;
use crate::setup::setup;
use crate::text_editor::TextEditorPlugin;
use crate::theme::BACKGROUND_COLOR;
use crate::toolbar::{
    handle_toolbar_mode_selection, update_current_edit_mode, CurrentEditMode,
};
use crate::ufo::initialize_font_state;

// Plugin to organize debug-related systems
struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_main_toolbar_debug, spawn_debug_text))
            .add_systems(
                Update,
                (update_main_toolbar_debug, update_font_info_text),
            );
    }
}

// Plugin to organize camera-related systems
struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_coordinate_display, toggle_camera_controls),
        );
    }
}

// Plugin to organize toolbar-related systems
struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_toolbar_mode_selection, update_current_edit_mode),
        );
    }
}

// Plugin to organize setup systems
struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (initialize_font_state, setup));
    }
}

// Main application plugin that bundles all internal plugins
struct BezySystems;

impl Plugin for BezySystems {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SetupPlugin,
            DebugPlugin,
            CameraPlugin,
            ToolbarPlugin,
        ));
    }
}

// Create the app and add the plugins and systems
pub fn create_app(cli_args: CliArgs) -> App {
    let mut app = App::new();

    // Configure app with default settings
    configure_app_settings(&mut app, cli_args);
    
    // Add all plugins
    add_plugins(&mut app);

    app
}

// Helper function to create window configuration
fn create_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Bezy".into(),
            resolution: (256. * 5., 256. * 3.).into(),
            ..default()
        }),
        ..default()
    }
}


// Configure basic app settings and resources
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
    app.init_resource::<AppState>()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CurrentEditMode::default())
        .insert_resource(cli_args); // Add CLI args as a resource
}

// Add all necessary plugins
fn add_plugins(app: &mut App) {
    // Add built-in plugins with our window configuration
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(create_window_plugin()),
    );
    
    // Add camera plugin
    app.add_plugins(PanCamPlugin::default());
    
    // Add application-specific plugins
    app.add_plugins((
        TextEditorPlugin, 
        DesignSpacePlugin,
        DrawPlugin, 
        MainToolbarPlugin,
        BezySystems, // Bundle of our internal system plugins
    ));
}
