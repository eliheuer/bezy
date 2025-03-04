// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::checkerboard::CheckerboardPlugin;
use crate::cli::CliArgs;
use crate::crypto_toolbar::CryptoToolbarPlugin;
use crate::data::AppState;
use crate::debug_hud::{spawn_debug_text, update_font_info_text};
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::edit_mode_toolbar::CurrentEditMode;
use crate::edit_mode_toolbar::EditModeToolbarPlugin;
use crate::setup::setup;
use crate::text_editor::TextEditorPlugin;
use crate::theme::BACKGROUND_COLOR;
use crate::ufo::initialize_font_state;

// Create the app and add the plugins and systems
pub fn create_app(cli_args: CliArgs) -> App {
    // Initialize a custom logger that excludes timestamps but keeps colors
    init_custom_logger();
    let mut app = App::new();
    // Configure app with default settings
    configure_app_settings(&mut app, cli_args);
    // Add all plugins
    add_plugins(&mut app);
    app
}

// Plugin to organize debug-related systems
struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_debug_text,
        )
        .add_systems(
            Update,
            update_font_info_text,
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

// Plugin to organize toolbar-related plugins
struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentEditMode>()
            .add_plugins(CryptoToolbarPlugin);
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
    // but disable Bevy's built-in LogPlugin since we're using our own custom logger
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(create_window_plugin())
            .build()
            .disable::<bevy::log::LogPlugin>(),
    );

    // Add camera plugin
    app.add_plugins(PanCamPlugin::default());

    // Add application-specific plugins
    app.add_plugins((
        TextEditorPlugin,
        DesignSpacePlugin,
        DrawPlugin,
        EditModeToolbarPlugin,
        BezySystems, // Bundle of our internal system plugins
        CheckerboardPlugin,
    ));
}

// Custom logger initialization to exclude timestamps.
// This is AI generated code used to make the logs cleaner,
// don't worry if you dont understand it, I don't either. --Eli H
fn init_custom_logger() {
    use tracing_subscriber::fmt::format;
    use tracing_subscriber::fmt::time::FormatTime;
    use tracing_subscriber::prelude::*;

    // Empty time formatter that doesn't print anything
    struct EmptyTime;
    impl FormatTime for EmptyTime {
        fn format_time(
            &self,
            _: &mut tracing_subscriber::fmt::format::Writer<'_>,
        ) -> std::fmt::Result {
            // Do nothing, effectively removing timestamps
            Ok(())
        }
    }

    // Set up a custom tracing subscriber with our configuration
    let format = format()
        .with_timer(EmptyTime)
        .with_level(true)
        .with_target(true)
        .with_ansi(true); // Keep colors

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(format)
                .with_filter(
                    tracing_subscriber::filter::EnvFilter::from_default_env()
                        .add_directive("info".parse().unwrap())
                        .add_directive("wgpu_core=warn".parse().unwrap())
                        .add_directive("wgpu_hal=warn".parse().unwrap()),
                ),
        )
        .init();
}
