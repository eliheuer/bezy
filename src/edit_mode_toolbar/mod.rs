use bevy::prelude::*;

mod hyper;
mod knife;
mod measure;
mod pan;
mod pen;
mod primitives;
mod select;
mod text;
mod ui;

pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use measure::MeasureMode;
pub use pan::PanMode;
pub use pen::PenMode;
pub use primitives::PrimitivesMode;
pub use select::SelectMode;
pub use text::TextMode;
pub use ui::{
    handle_toolbar_mode_selection, spawn_edit_mode_toolbar,
    update_current_edit_mode, CurrentEditMode,
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
        app.add_systems(
            Update,
            (
                // UI systems
                handle_toolbar_mode_selection,
                update_current_edit_mode,
            ),
        );
    }
}
