use bevy::gizmos::{config::DefaultGizmoConfigGroup, config::GizmoConfigStore};
use bevy::prelude::*;

use crate::editing::sort_plugin::SortPlugin;
use crate::rendering::cameras::toggle_camera_controls;
use crate::rendering::draw::{
    draw_origin_cross, draw_metrics_system,
    detect_app_state_changes, AppStateChanged,
};
use crate::data::ufo::initialize_font_state;
use crate::ui::panes::coord_pane::CoordinatePanePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::{
    GIZMO_LINE_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH,
};
use crate::ui::toolbars::access_toolbar::AccessToolbarPlugin;
use crate::ui::toolbars::edit_mode_toolbar::CurrentEditMode;
use crate::utils::setup::setup;

/// Configure the default Bevy plugins with custom settings
pub fn configure_default_plugins() -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                // Tell wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        // Disable Bevy's default LogPlugin since we're using our own custom logger
        .build()
        .disable::<bevy::log::LogPlugin>()
}

/// System to configure gizmo appearance
fn configure_gizmos(mut gizmo_store: ResMut<GizmoConfigStore>) {
    let (config, _) = gizmo_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line_width = GIZMO_LINE_WIDTH;
    info!("Configured gizmo line width to {}px", GIZMO_LINE_WIDTH);
}

/// Plugin to organize camera-related systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_camera_controls);
    }
}

/// Plugin to organize drawing-related systems
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AppStateChanged>()
            .add_systems(
                Update,
                (
                    draw_metrics_system,
                    detect_app_state_changes,
                ),
            )
            .add_systems(
                Update,
                draw_origin_cross.after(crate::editing::sort_plugin::SortSystemSet::Rendering),
            );
    }
}

/// Plugin to organize toolbar-related plugins
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentEditMode>()
            .add_plugins(AccessToolbarPlugin);
    }
}

/// Plugin to organize setup systems
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (initialize_font_state, setup, configure_gizmos),
        );
    }
}

/// Main application plugin that bundles all internal plugins
pub struct BezySystems;

impl Plugin for BezySystems {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SetupPlugin,
            CameraPlugin,
            DrawPlugin,
            ToolbarPlugin,
            CoordinatePanePlugin,
            GlyphPanePlugin,
            SortPlugin,
        ));
    }
}
