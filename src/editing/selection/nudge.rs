use bevy::prelude::*;
use crate::editing::edit_type::EditType;

/// Resource to track if we're currently in a nudging operation
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct NudgeState {
    /// Whether we're currently nudging (to prevent selection loss)
    pub is_nudging: bool,
    /// Timestamp of the last nudge operation
    pub last_nudge_time: f32,
    /// The last key that was pressed for nudging
    #[reflect(ignore)]
    pub last_key_pressed: Option<KeyCode>,
}

/// Component to track the last edit type for undo purposes
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct LastEditType {
    pub edit_type: Option<EditType>,
}

/// Event to signal that an edit has been made that should be added to the undo stack
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub struct EditEvent {
    pub edit_type: EditType,
}

/// Component to track point coordinates in font space
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PointCoordinates {
    pub position: Vec2,
}

/// Plugin to set up nudging functionality
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EditEvent>()
            .register_type::<EditType>()
            .register_type::<LastEditType>()
            .register_type::<PointCoordinates>()
            .init_resource::<NudgeState>();
    }
} 