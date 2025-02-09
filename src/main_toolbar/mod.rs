use bevy::prelude::*;

mod hyper;
mod knife;
mod measure;
mod pan;
mod pen;
mod primitives;
mod select;
mod text;

pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use measure::MeasureMode;
pub use pan::PanMode;
pub use pen::PenMode;
pub use primitives::PrimitivesMode;
pub use select::SelectMode;
pub use text::TextMode;

// Trait that all edit modes must implement
pub trait EditModeSystem: Send + Sync + 'static {
    fn update(&self, commands: &mut Commands);

    // Default implementations for lifecycle methods
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}
