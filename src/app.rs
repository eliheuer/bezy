// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::cli::CliArgs;
use crate::data::AppState;
use crate::debug_hud::{
    spawn_debug_text, spawn_main_toolbar_debug, update_main_toolbar_debug,
};
use crate::draw::DrawPlugin;
use crate::setup::setup;
use crate::text_editor::TextEditorPlugin;
use crate::theme::BACKGROUND_COLOR;
use crate::toolbar::{
    handle_toolbar_mode_selection, update_current_edit_mode, CurrentEditMode,
};
use crate::design_space::DesignSpacePlugin;

// Create the app and add the plugins and systems
pub fn create_app(cli_args: CliArgs) -> App {
    let mut app = App::new();

    let window_config = Window {
        title: "Bezy".into(),
        resolution: (256. * 5., 256. * 3.).into(),
        ..default()
    };

    let window_plugin = WindowPlugin {
        primary_window: Some(window_config),
        ..default()
    };

    // Sequence of events to start and run the app
    // Pay attention to the order of the systems
    app.init_resource::<AppState>()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(CurrentEditMode::default())
        .insert_resource(cli_args)  // Add CLI args as a resource
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin),
        )
        .add_plugins(PanCamPlugin::default())
        .add_plugins(TextEditorPlugin)
        .add_plugins(DesignSpacePlugin)
        .add_plugins(DrawPlugin)
        // Initialize font state before setup
        .add_systems(Startup, (crate::ufo::initialize_font_state, setup))
        // When the app starts, run the setup system and spawn everything
        .add_systems(
            Startup,
            (
                spawn_main_toolbar_debug,
                spawn_debug_text,
            ),
        )
        // Update the app and get input
        .add_systems(
            Update,
            (
                handle_toolbar_mode_selection,
                update_main_toolbar_debug,
                update_current_edit_mode,
                update_coordinate_display,
                toggle_camera_controls,
                crate::debug_hud::update_font_info_text,
            ),
        );
    app
}
