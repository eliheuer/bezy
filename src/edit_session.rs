use crate::selection::components::{PointType, Selectable};
use bevy::prelude::*;

/// A session for editing a glyph
#[derive(Component, Debug, Clone)]
pub struct EditSession {
    /// The selection state
    pub selection_count: usize,
}

impl Default for EditSession {
    fn default() -> Self {
        Self { selection_count: 0 }
    }
}

impl EditSession {
    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.selection_count == 0
    }

    /// Nudge the selected points by the given amount
    pub fn nudge_selection(&mut self, nudge: bevy::prelude::Vec2) {
        if self.is_empty() {
            return;
        }

        info!("Nudging selection by {:?}", nudge);
        // In a real implementation, this would modify the points
    }
}

/// Plugin to register the EditSession component and systems
pub struct EditSessionPlugin;

impl Plugin for EditSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_test_points)
            .add_systems(Update, edit_session_system);
    }
}

/// System to create test points for selection
fn create_test_points(mut commands: Commands) {
    info!("Creating test points for selection");

    // Create a grid of test points
    for x in 0..5 {
        for y in 0..5 {
            let position = Vec3::new(
                x as f32 * 50.0 + 100.0,
                y as f32 * 50.0 + 100.0,
                0.0,
            );

            commands.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Selectable,
                PointType { is_on_curve: true },
                Name::new(format!("TestPoint_{}_{}", x, y)),
            ));
        }
    }
}

/// System to update the edit session
fn edit_session_system(_query: Query<&mut EditSession>) {
    // Add system logic here
}
