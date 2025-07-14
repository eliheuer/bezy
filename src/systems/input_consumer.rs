//! Input Consumer System
//!
//! This module provides the input consumer system that routes input events
//! to the appropriate handlers based on priority and current input mode.
//! It ensures that input is handled consistently and predictably across
//! the application.

use crate::core::input::{helpers, InputEvent, InputMode, InputState};
use crate::core::pointer::PointerInfo;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionRect,
};
use crate::editing::selection::{
    DragPointState, DragSelectionState, SelectionState,
};
use crate::editing::sort::ActiveSortState;
use crate::geometry::design_space::DPoint;
use crate::systems::sort_manager::SortPointEntity;
use crate::systems::ui_interaction::UiHoverState;
use bevy::prelude::*;

/// Trait for input consumers that handle specific types of input events
pub trait InputConsumer {
    /// Determine if this consumer should handle the given input event
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool;

    /// Handle the input event
    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState);
}

/// Input consumer for selection functionality
#[derive(Resource, Default)]
pub struct SelectionInputConsumer;

impl InputConsumer for SelectionInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle mouse events for selection
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Normal)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                debug!(
                    "[SELECTION] Mouse click: {:?} at {:?} with {:?}",
                    button, position, modifiers
                );
                // Selection logic would go here
            }
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                modifiers,
                delta: _,
            } => {
                debug!(
                    "[SELECTION] Mouse drag: {:?} from {:?} to {:?} with {:?}",
                    button, start_position, current_position, modifiers
                );
                // Drag selection logic would go here
            }
            _ => {}
        }
    }
}

/// Input consumer for pen tool functionality
#[derive(Resource, Default)]
pub struct PenInputConsumer;

impl InputConsumer for PenInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle pen tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Pen)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event {
            debug!(
                "[PEN] Mouse click: {:?} at {:?} with {:?}",
                button, position, modifiers
            );
            // Pen tool logic would go here
        }
    }
}

/// Input consumer for knife tool functionality
#[derive(Resource, Default)]
pub struct KnifeInputConsumer;

impl InputConsumer for KnifeInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle knife tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Knife)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event {
            debug!(
                "[KNIFE] Mouse click: {:?} at {:?} with {:?}",
                button, position, modifiers
            );
            // Knife tool logic would go here
        }
    }
}

/// Input consumer for shape tool functionality
#[derive(Resource, Default)]
pub struct ShapeInputConsumer;

impl InputConsumer for ShapeInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle shape tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Shape)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event {
            debug!(
                "[SHAPE] Mouse click: {:?} at {:?} with {:?}",
                button, position, modifiers
            );
            // Shape tool logic would go here
        }
    }
}

/// Input consumer for hyper tool functionality
#[derive(Resource, Default)]
pub struct HyperInputConsumer;

impl InputConsumer for HyperInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle hyper tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Hyper)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event {
            debug!(
                "[HYPER] Mouse click: {:?} at {:?} with {:?}",
                button, position, modifiers
            );
            // Hyper tool logic would go here
        }
    }
}

/// Input consumer for text editing functionality
#[derive(Resource, Default)]
pub struct TextInputConsumer;

impl InputConsumer for TextInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle text input events
        matches!(
            event,
            InputEvent::KeyPress { .. } | InputEvent::TextInput { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Text)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::KeyPress { key, modifiers } => {
                debug!("[TEXT] Key press: {:?} with {:?}", key, modifiers);
                // Text editing logic would go here
            }
            InputEvent::TextInput { text } => {
                debug!("[TEXT] Text input: '{}'", text);
                // Text input logic would go here
            }
            _ => {}
        }
    }
}

/// Input consumer for camera control functionality
#[derive(Resource, Default)]
pub struct CameraInputConsumer;

impl InputConsumer for CameraInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle camera control events (low priority)
        matches!(
            event,
            InputEvent::MouseDrag { .. } | InputEvent::MouseWheel { .. }
        ) && !helpers::is_input_mode(input_state, InputMode::Text)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                modifiers,
                delta: _,
            } => {
                if *button == MouseButton::Middle {
                    debug!("[CAMERA] Middle mouse drag: from {:?} to {:?} with {:?}", 
                           start_position, current_position, modifiers);
                    // Camera pan logic would go here
                }
            }
            InputEvent::MouseWheel { delta } => {
                debug!("[CAMERA] Mouse wheel: {:?}", delta);
                // Camera zoom logic would go here
            }
            _ => {}
        }
    }
}

/// Input consumer for measurement tool functionality
#[derive(Resource, Default)]
pub struct MeasurementToolInputConsumer;

impl InputConsumer for MeasurementToolInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Handle measurement tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Temporary)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event {
            debug!(
                "[MEASURE] Mouse click: {:?} at {:?} with {:?}",
                button, position, modifiers
            );
            // Measurement tool logic would go here
        }
    }
}

/// System to process input events and route them to appropriate consumers
#[allow(clippy::too_many_arguments)]
fn process_input_events(
    mut input_events: EventReader<InputEvent>,
    input_state: Res<InputState>,
    mut selection_consumer: ResMut<SelectionInputConsumer>,
    mut pen_consumer: ResMut<PenInputConsumer>,
    mut knife_consumer: ResMut<KnifeInputConsumer>,
    mut shape_consumer: ResMut<ShapeInputConsumer>,
    mut hyper_consumer: ResMut<HyperInputConsumer>,
    mut text_consumer: ResMut<TextInputConsumer>,
    mut camera_consumer: ResMut<CameraInputConsumer>,
    mut measurement_consumer: ResMut<MeasurementToolInputConsumer>,
) {
    let events: Vec<_> = input_events.read().collect();
    debug!("[INPUT CONSUMER] Processing {} input events", events.len());

    for event in events {
        debug!("[INPUT CONSUMER] Processing event: {:?}", event);

        // Route events to consumers based on priority
        // High priority: Text input
        if text_consumer.should_handle_input(event, &input_state) {
            text_consumer.handle_input(event, &input_state);
            continue;
        }

        // Mode-specific consumers
        if pen_consumer.should_handle_input(event, &input_state) {
            pen_consumer.handle_input(event, &input_state);
            continue;
        }

        if knife_consumer.should_handle_input(event, &input_state) {
            knife_consumer.handle_input(event, &input_state);
            continue;
        }

        if shape_consumer.should_handle_input(event, &input_state) {
            shape_consumer.handle_input(event, &input_state);
            continue;
        }

        if hyper_consumer.should_handle_input(event, &input_state) {
            hyper_consumer.handle_input(event, &input_state);
            continue;
        }

        if measurement_consumer.should_handle_input(event, &input_state) {
            measurement_consumer.handle_input(event, &input_state);
            continue;
        }

        // Normal mode consumers
        if selection_consumer.should_handle_input(event, &input_state) {
            selection_consumer.handle_input(event, &input_state);
            continue;
        }

        // Low priority: Camera control
        if camera_consumer.should_handle_input(event, &input_state) {
            camera_consumer.handle_input(event, &input_state);
            continue;
        }

        debug!("[INPUT CONSUMER] No consumer handled event: {:?}", event);
    }
}

/// Plugin for the input consumer system
pub struct InputConsumerPlugin;

impl Plugin for InputConsumerPlugin {
    fn build(&self, app: &mut App) {
        info!("[INPUT CONSUMER] Registering InputConsumerPlugin");

        // Register all input consumers as resources
        app.init_resource::<SelectionInputConsumer>()
            .init_resource::<PenInputConsumer>()
            .init_resource::<KnifeInputConsumer>()
            .init_resource::<ShapeInputConsumer>()
            .init_resource::<HyperInputConsumer>()
            .init_resource::<TextInputConsumer>()
            .init_resource::<CameraInputConsumer>()
            .init_resource::<MeasurementToolInputConsumer>()
            .add_systems(Update, process_input_events);

        info!("[INPUT CONSUMER] InputConsumerPlugin registration complete");
    }
}
