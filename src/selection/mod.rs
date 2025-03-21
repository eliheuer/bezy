pub mod components;
pub mod nudge;
pub mod systems;

use bevy::prelude::*;
pub use components::*;
pub use nudge::*;

/// Resource to track the drag selection state
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection
    pub start_position: Option<Vec2>,
    /// The current position of the drag selection
    pub current_position: Option<Vec2>,
    /// Whether this is a multi-select operation (shift is held)
    pub is_multi_select: bool,
    /// The previous selection before the drag started
    #[reflect(ignore)]
    pub previous_selection: Vec<Entity>,
}

/// Plugin to add selection functionality to the font editor
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register components
            .register_type::<Selectable>()
            .register_type::<Selected>()
            .register_type::<Hovered>()
            .register_type::<SelectionRect>()
            .register_type::<PointType>()
            .register_type::<GlyphPointReference>()
            .register_type::<LastEditType>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<DragSelectionState>()
            .init_resource::<nudge::NudgeState>()
            .add_event::<nudge::EditEvent>()
            // Add core selection systems
            .configure_sets(Update, (
                SelectionSystemSet::Input,
                SelectionSystemSet::Processing,
                SelectionSystemSet::Render,
            ).chain())
            .add_systems(
                Update,
                (
                    systems::handle_mouse_input,
                    systems::handle_selection_shortcuts,
                    systems::handle_key_releases,
                ).in_set(SelectionSystemSet::Input)
            )
            .add_systems(
                Update,
                (
                    systems::update_glyph_data_from_selection,
                    sync_selected_components,
                ).in_set(SelectionSystemSet::Processing)
                .after(SelectionSystemSet::Input)
            )
            .add_systems(
                Update,
                (
                    systems::render_selection_rect,
                    systems::render_selected_entities,
                ).in_set(SelectionSystemSet::Render)
                .after(SelectionSystemSet::Processing)
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin)
            // Add the new system here
            .add_systems(
                Update,
                (
                    systems::handle_line_segment_upgrade,
                    systems::handle_line_segment_upgrade_shortcut,
                ).in_set(SelectionSystemSet::Processing)
                .after(SelectionSystemSet::Input)
            );
    }
}

/// System sets for Selection
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SelectionSystemSet {
    Input,
    Processing,
    Render,
}

/// System to ensure Selected components are synchronized with SelectionState
pub fn sync_selected_components(
    mut commands: Commands,
    selection_state: Res<SelectionState>,
    selected_entities: Query<Entity, With<Selected>>,
    entities: Query<Entity>,
) {
    // Always run this system to ensure components stay synchronized
    info!("Synchronizing Selected components with SelectionState (current: {})", selection_state.selected.len());
    
    // First, ensure all entities in the selection_state have the Selected component
    for &entity in &selection_state.selected {
        // Only add the component if the entity is valid
        if entities.contains(entity) && !selected_entities.contains(entity) {
            commands.entity(entity).insert(Selected);
            info!("Adding Selected component to entity {:?} from selection state", entity);
        }
    }
    
    // Then, ensure all entities with the Selected component are in the selection_state
    for entity in &selected_entities {
        if !selection_state.selected.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            info!("Removing Selected component from entity {:?} not in selection state", entity);
        }
    }
}
