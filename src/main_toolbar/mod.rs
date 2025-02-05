use bevy::prelude::*;

mod circle;
mod hyper;
mod knife;
mod measure;
mod pan;
mod pen;
mod select;
mod square;

pub use circle::CircleMode;
pub use hyper::HyperMode;
pub use knife::KnifeMode;
pub use measure::MeasureMode;
pub use pan::PanMode;
pub use pen::PenMode;
pub use select::SelectMode;
pub use square::SquareMode;

// Trait that all edit modes must implement
pub trait EditModeSystem: Send + Sync {
    fn update(&self, commands: &mut Commands);

    // Default implementations for lifecycle methods
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}
