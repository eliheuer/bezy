//! Application initialization and configuration

use crate::core::cli::CliArgs;
use crate::core::settings::BezySettings;
use crate::core::state::{AppState, GlyphNavigation};
use crate::core::pointer::PointerPlugin;
use crate::core::input::InputPlugin;
use crate::systems::{BezySystems, CommandsPlugin, UiInteractionPlugin, InputConsumerPlugin};
use crate::editing::{SelectionPlugin, TextEditorPlugin, UndoPlugin};
use crate::rendering::{
    cameras::CameraPlugin, checkerboard::CheckerboardPlugin,
};
use crate::ui::hud::HudPlugin;
use crate::ui::panes::coord_pane::CoordinatePanePlugin;
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::BACKGROUND_COLOR;
use crate::ui::toolbars::EditModeToolbarPlugin;
use crate::ui::GlyphGridPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;

/// Creates a fully configured Bevy GUI application ready to run
pub fn create_app(cli_args: CliArgs) -> Result<App, String> {
    #[cfg(not(target_arch = "wasm32"))]
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
        .insert_resource(ClearColor(BACKGROUND_COLOR));
    
    // Configure window settings based on target platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.insert_resource(WinitSettings::desktop_app());
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        app.insert_resource(WinitSettings::game());
    }
}

/// Adds all plugins to the application in logical groups
fn add_all_plugins(app: &mut App) {
    // Configure plugins based on target platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy Font Editor".to_string(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }));
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy Font Editor".to_string(),
                canvas: None,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }).set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings {
                    backends: Some(bevy::render::settings::Backends::GL),
                    power_preference: bevy::render::settings::PowerPreference::LowPower,
                    ..default()
                }
            ),
            ..default()
        }));
    }
    
    add_rendering_plugins(app);
    add_editor_plugins(app);
    add_core_plugins(app);
    // Register GlyphGridPlugin after all core plugins
    app.add_plugins(GlyphGridPlugin);
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
    info!("Adding editor plugins...");
    app.add_plugins((
        DesignSpacePlugin,
        GlyphPanePlugin,
        CoordinatePanePlugin,
        EditModeToolbarPlugin,
        HudPlugin,
        // GlyphGridPlugin removed from here
    ));
    info!("Editor plugins added successfully");
}

/// Adds core application logic plugins
fn add_core_plugins(app: &mut App) {
    app.add_plugins((
        // Core infrastructure first
        PointerPlugin,
        InputPlugin,
        InputConsumerPlugin, // ENABLED: Centralized input system
        // Then editing systems (TextEditor first so it can set up active sort state)
        TextEditorPlugin,
        SelectionPlugin,
        UndoPlugin,
        // Then UI and interaction
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
