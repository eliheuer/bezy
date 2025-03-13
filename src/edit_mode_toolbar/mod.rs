use bevy::prelude::*;

mod hyper;
mod knife;
mod measure;
mod pan;
mod pen;
mod primitives;
mod primitives_mode;
pub mod select;
mod text;
mod ui;

pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use measure::MeasureMode;
pub use pan::PanMode;
pub use pen::PenMode;
pub use primitives::base::{
    handle_primitive_mouse_events, ActivePrimitiveDrawing,
};
pub use primitives::ui::{
    handle_radius_input, spawn_primitive_controls,
    update_primitive_ui_visibility, CurrentCornerRadius, UiInteractionState,
};
pub use primitives_mode::PrimitivesMode;
pub use primitives_mode::{
    handle_active_primitive_tool, handle_primitive_selection,
    spawn_primitives_submenu, toggle_primitive_submenu_visibility,
};
pub use primitives_mode::{CurrentPrimitiveType, PrimitiveType};
pub use select::SelectMode;
pub use text::TextMode;
pub use ui::{
    handle_toolbar_mode_selection, spawn_edit_mode_toolbar,
    update_current_edit_mode, CurrentEditMode, EditMode,
};

/// Trait that defines the behavior of an edit mode in the application.
///
/// An edit mode represents a specific tool or interaction mode that the user
/// can select from the toolbar. Each mode has its own behavior and state.
///
/// # Implementation Requirements
///
/// Implementers must define the `update` method to specify how the edit mode
/// behaves during the application update cycle. They can optionally override
/// the `on_enter` and `on_exit` methods to handle state transitions when the
/// mode is activated or deactivated.
///
/// # Example
///
/// ```
/// struct MyEditMode;
///
/// impl EditModeSystem for MyEditMode {
///     fn update(&self, commands: &mut Commands) {
///         // Implement mode-specific logic here
///     }
///     
///     fn on_enter(&self) {
///         // Setup logic when mode is activated
///     }
///     
///     fn on_exit(&self) {
///         // Cleanup logic when mode is deactivated
///     }
/// }
/// ```
pub trait EditModeSystem: Send + Sync + 'static {
    fn update(&self, commands: &mut Commands);
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}

/// Plugin that adds all the toolbar functionality
pub struct EditModeToolbarPlugin;

impl Plugin for EditModeToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPrimitiveType>()
            .init_resource::<ActivePrimitiveDrawing>()
            .init_resource::<CurrentCornerRadius>()
            .init_resource::<UiInteractionState>()
            .add_systems(Startup, handle_primitive_selection)
            .add_systems(
                Update,
                (
                    // UI systems
                    handle_toolbar_mode_selection,
                    update_current_edit_mode,
                    // Primitives sub-menu systems
                    handle_primitive_selection,
                    toggle_primitive_submenu_visibility,
                    handle_active_primitive_tool,
                    // Rounded rectangle radius control - runs BEFORE mouse event handling
                    update_primitive_ui_visibility,
                    handle_radius_input,
                    // Mouse event handling for drawing shapes - runs AFTER UI systems
                    handle_primitive_mouse_events,
                    // Camera panning control based on edit mode
                    pan::toggle_pancam_on_mode_change,
                ),
            );
    }
}
