//! Application initialization and configuration

use crate::core::cli::CliArgs;
use crate::core::settings::BezySettings;
use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::{SelectionPlugin, TextEditorPlugin, UndoPlugin};
use crate::rendering::{
    cameras::CameraPlugin, checkerboard::CheckerboardPlugin,
};
use crate::systems::{BezySystems, CommandsPlugin, UiInteractionPlugin};
use crate::ui::hud::HudPlugin;
use crate::ui::panes::coord_pane::CoordinatePanePlugin;
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::BACKGROUND_COLOR;
use crate::ui::toolbars::EditModeToolbarPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;

/// Creates a fully configured Bevy GUI application ready to run
pub fn create_app(cli_args: CliArgs) -> Result<App, String> {
    cli_args.validate()?;
    let mut app = App::new();
    configure_app_settings(&mut app, cli_args);
    add_all_plugins(&mut app);
    Ok(app)
}

/// Sets up application resources and configuration
fn configure_app_settings(app: &mut App, cli_args: CliArgs) {
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
    app.add_plugins((CameraPlugin, CheckerboardPlugin));
}

/// Adds plugins for editor UI and interaction
fn add_editor_plugins(app: &mut App) {
    app.add_plugins((
        DesignSpacePlugin,
        GlyphPanePlugin,
        CoordinatePanePlugin,
        EditModeToolbarPlugin,
        HudPlugin,
    ));
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        SelectionPlugin,
        TextEditorPlugin,
        UndoPlugin,
        UiInteractionPlugin,
        CommandsPlugin,
        BezySystems,
    ));
}

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
                error!("Failed to load UFO font: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");
            }
        }
    } else {
        warn!("No UFO font path specified, running without a font loaded.");
    }
}
