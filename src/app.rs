// Creates the app and adds the plugins and systems
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::PanCamPlugin;

use crate::cameras::{toggle_camera_controls, update_coordinate_display};
use crate::checkerboard::CheckerboardPlugin;
use crate::cli::CliArgs;
use crate::commands::{CodepointDirection, CycleCodepointEvent};
use crate::crypto_toolbar::CryptoToolbarPlugin;
use crate::data::AppState;
use crate::debug_hud::{spawn_debug_text, update_codepoint_not_found_text};
use crate::design_space::DesignSpacePlugin;
use crate::draw::DrawPlugin;
use crate::edit_mode_toolbar::select::SelectModePlugin;
use crate::edit_mode_toolbar::CurrentEditMode;
use crate::edit_mode_toolbar::EditModeToolbarPlugin;
use crate::selection::SelectionPlugin;
use crate::setup::setup;
use crate::text_editor::TextEditorPlugin;
use crate::theme::BACKGROUND_COLOR;
use crate::ufo::{
    initialize_font_state, print_font_info_to_terminal, LastCodepointPrinted,
};

/// System to handle keyboard shortcuts to cycle through codepoints
fn handle_codepoint_cycling(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cycle_event: EventWriter<CycleCodepointEvent>,
) {
    // Check for Shift+Plus to cycle forward through codepoints
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if shift_pressed {
        // Check for Shift+= (Plus) to move to next codepoint
        if keyboard.just_pressed(KeyCode::Equal) {
            info!(
                "Detected Shift+= key combination, cycling to next codepoint"
            );
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Next,
            });
        }

        // Check for Shift+- (Minus) to move to previous codepoint
        if keyboard.just_pressed(KeyCode::Minus) {
            info!("Detected Shift+- key combination, cycling to previous codepoint");
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Previous,
            });
        }
    }
}

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
        app.init_resource::<crate::debug_hud::WarningTextState>()
            .init_resource::<crate::ufo::LastCodepointPrinted>()
            .add_systems(Startup, spawn_debug_text)
            .add_systems(
                Update,
                (
                    // Print UFO and codepoint info to terminal
                    print_font_info_to_terminal,
                    update_codepoint_not_found_text
                        .after(crate::draw::draw_glyph_points_system),
                ),
            );
    }
}

// Plugin to organize camera-related systems
struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::commands::CycleCodepointEvent>()
            .add_systems(
                Update,
                (
                    update_coordinate_display,
                    toggle_camera_controls,
                    handle_codepoint_cycling,
                ),
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
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bezy".into(),
            resolution: (900., 900.).into(),
            // Tell wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
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
