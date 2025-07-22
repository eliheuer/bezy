//! Application initialization and configuration

use crate::core::cli::CliArgs;
use crate::core::io::gamepad::GamepadPlugin;
use crate::core::io::input::InputPlugin;
use crate::core::io::pointer::PointerPlugin;
use crate::core::settings::{BezySettings, DEFAULT_WINDOW_SIZE, WINDOW_TITLE};
use crate::core::state::GlyphNavigation;
use crate::editing::{SelectionPlugin, TextEditorPlugin, UndoPlugin};
use crate::rendering::{
    cameras::CameraPlugin, checkerboard::CheckerboardPlugin, MeshGlyphOutlinePlugin, 
    OutlineElementsPlugin, PointRenderingPlugin,
};
use crate::systems::{
    exit_on_esc, load_fontir_font, load_ufo_font, BezySystems, CommandsPlugin,
    InputConsumerPlugin, UiInteractionPlugin,
};
use crate::ui::hud::HudPlugin;
use crate::ui::panes::coord_pane::CoordinatePanePlugin;
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::CurrentTheme;
use crate::ui::toolbars::EditModeToolbarPlugin;
use anyhow::Result;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::*;
use bevy::winit::WinitSettings;

/// Plugin group for core application functionality
#[derive(Default)]
pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PointerPlugin)
            .add(InputPlugin)
            .add(GamepadPlugin)
            .add(InputConsumerPlugin)
            .add(TextEditorPlugin)
            .add(SelectionPlugin)
            .add(UndoPlugin)
            .add(UiInteractionPlugin)
            .add(CommandsPlugin)
            .add(BezySystems)
    }
}

/// Plugin group for rendering functionality
#[derive(Default)]
pub struct RenderingPluginGroup;

impl PluginGroup for RenderingPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraPlugin)
            .add(CheckerboardPlugin)
            .add(MeshGlyphOutlinePlugin)
            .add(PointRenderingPlugin)
            .add(OutlineElementsPlugin)
    }
}

/// Plugin group for editor UI
#[derive(Default)]
pub struct EditorPluginGroup;

impl PluginGroup for EditorPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(DesignSpacePlugin)
            .add(GlyphPanePlugin)
            .add(CoordinatePanePlugin)
            .add(EditModeToolbarPlugin)
            .add(HudPlugin)
            // Add clean tools plugin and supporting plugins
            .add(crate::tools::CleanToolsPlugin)
            .add(crate::tools::SelectToolPlugin)
            .add(crate::tools::PenToolPlugin)
        // .add(crate::tools::TextToolPlugin) // Disabled - handled by legacy text tool with submenu
    }
}

/// Creates a fully configured Bevy GUI application ready to run
pub fn create_app(cli_args: CliArgs) -> Result<App> {
    #[cfg(not(target_arch = "wasm32"))]
    cli_args
        .validate()
        .map_err(|e| anyhow::anyhow!("CLI validation failed: {}", e))?;

    let mut app = App::new();
    configure_resources(&mut app, cli_args);
    configure_window_plugins(&mut app);
    add_plugin_groups(&mut app);
    add_lifecycle_systems(&mut app);
    Ok(app)
}

/// Sets up application resources and configuration
fn configure_resources(app: &mut App, cli_args: CliArgs) {
    let glyph_navigation = GlyphNavigation::default();
    let mut settings = BezySettings::default();

    // Set theme from CLI args (CLI overrides settings)
    let theme_variant = cli_args.get_theme_variant();
    settings.set_theme(theme_variant.clone());

    // Initialize current theme
    let current_theme = CurrentTheme::new(theme_variant);
    let background_color = current_theme.theme().background_color();

    // Note: FontIRAppState is initialized by load_fontir_font startup system
    // app.init_resource::<AppState>() // Old system - keeping for gradual migration
    app.insert_resource(cli_args)
        .insert_resource(glyph_navigation)
        .insert_resource(settings)
        .insert_resource(current_theme)
        .insert_resource(ClearColor(background_color));

    // Configure platform-specific window settings
    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(WinitSettings::desktop_app());

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(WinitSettings::game());
}

/// Configure window and default plugins with platform-specific settings
fn configure_window_plugins(app: &mut App) {
    let window_config = Window {
        title: WINDOW_TITLE.to_string(),
        resolution: DEFAULT_WINDOW_SIZE.into(),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window_config),
            ..default()
        }));
    }

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: None,
                prevent_default_event_handling: false,
                ..window_config
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
}

/// Add all plugin groups to the application
fn add_plugin_groups(app: &mut App) {
    info!("Adding plugin groups...");
    app.add_plugins((RenderingPluginGroup, EditorPluginGroup, CorePluginGroup));
    info!("All plugin groups added successfully");
}

/// Add lifecycle systems for startup and shutdown
fn add_lifecycle_systems(app: &mut App) {
    app.add_systems(Startup, load_fontir_font)
        .add_systems(Update, exit_on_esc);
}
