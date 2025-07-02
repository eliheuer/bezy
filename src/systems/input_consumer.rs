//! Input Consumer System
//!
//! This module provides the input consumer system that routes input events
//! to the appropriate handlers based on priority and current input mode.
//! It ensures that input is handled consistently and predictably across
//! the application.

use bevy::prelude::*;
use crate::core::input::{InputEvent, InputState, InputMode, helpers};
use crate::core::pointer::PointerInfo;
use crate::ui::panes::design_space::DPoint;
use crate::editing::selection::{DragSelectionState, DragPointState, SelectionState};
use crate::editing::selection::components::{Selectable, Selected, SelectionRect, PointType, GlyphPointReference};
use crate::editing::selection::nudge::{EditEvent, NudgeState};
use crate::systems::ui_interaction::UiHoverState;
use crate::editing::sort::ActiveSortState;
use crate::systems::sort_manager::SortPointEntity;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;

/// Trait for systems that consume input events
pub trait InputConsumer: Send + Sync + 'static {
    /// Determine if this consumer should handle the given input event
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool;
    
    /// Handle the input event
    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState);
    
    /// Get the priority of this consumer (higher numbers = higher priority)
    fn priority(&self) -> u32 {
        0
    }
}

/// Plugin for the input consumer system
pub struct InputConsumerPlugin;

impl Plugin for InputConsumerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add systems
            .add_systems(Update, process_input_events)
            // Add input consumers as resources
            .init_resource::<SelectionInputConsumer>()
            .init_resource::<PenInputConsumer>()
            .init_resource::<KnifeInputConsumer>()
            .init_resource::<ShapeInputConsumer>()
            .init_resource::<HyperInputConsumer>()
            .init_resource::<TextInputConsumer>()
            .init_resource::<CameraInputConsumer>()
            .init_resource::<MeasurementToolInputConsumer>();
    }
}

/// System to process input events and route them to appropriate consumers
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
    for event in input_events.read() {
        // Process events in priority order
        // 1. Mode-specific consumers (highest priority)
        if selection_consumer.should_handle_input(event, &input_state) {
            selection_consumer.handle_input(event, &input_state);
            continue;
        }

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

        if text_consumer.should_handle_input(event, &input_state) {
            text_consumer.handle_input(event, &input_state);
            continue;
        }

        // 2. Camera consumer (lowest priority)
        if camera_consumer.should_handle_input(event, &input_state) {
            camera_consumer.handle_input(event, &input_state);
            continue;
        }

        // 3. Custom tool consumers (if any)
        if measurement_consumer.should_handle_input(event, &input_state) {
            measurement_consumer.handle_input(event, &input_state);
            continue;
        }

        // If no consumer handled the event, log it for debugging
        debug!("No input consumer handled event: {:?}", event);
    }
}

/// Input consumer for selection tool
#[derive(Resource, Default)]
pub struct SelectionInputConsumer;

impl InputConsumer for SelectionInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Only handle input if select mode is active
        if !helpers::is_input_mode(input_state, InputMode::Select) {
            return false;
        }
        
        // Skip if UI is consuming input
        if helpers::is_ui_consuming(input_state) {
            return false;
        }
        
        // Handle mouse events and keyboard shortcuts
        match event {
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. } => true,
            InputEvent::KeyPress { key, .. } => matches!(key, KeyCode::KeyA | KeyCode::Escape),
            _ => false,
        }
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        // The actual selection logic will be handled by the selection systems
        // This consumer just ensures that selection events are routed correctly
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Selection: Mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // The selection systems will handle this via the existing process_selection_input_events
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Selection: Mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // The selection systems will handle this via the existing process_selection_input_events
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Selection: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // The selection systems will handle this via the existing process_selection_input_events
                }
            }
            InputEvent::KeyPress { key, modifiers } => {
                info!("Selection: Key press {:?} with modifiers {:?}", key, modifiers);
                // The selection systems will handle this via the existing process_selection_input_events
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        100 // High priority
    }
}

/// Input consumer for pen tool
#[derive(Resource, Default)]
pub struct PenInputConsumer;

impl InputConsumer for PenInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Only handle input if pen mode is active
        if !helpers::is_input_mode(input_state, InputMode::Pen) {
            return false;
        }
        
        // Handle mouse events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Pen: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen click handling
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Pen: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement pen drag handling
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Pen: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen release handling
                }
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        50 // Medium priority
    }
}

/// Input consumer for knife tool
#[derive(Resource, Default)]
pub struct KnifeInputConsumer;

impl InputConsumer for KnifeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Only handle input if knife mode is active
        if !helpers::is_input_mode(input_state, InputMode::Knife) {
            return false;
        }
        
        // Handle mouse events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Knife: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement knife click handling
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Knife: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement knife drag handling
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Knife: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement knife release handling
                }
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        50 // Medium priority
    }
}

/// Input consumer for shape tool
#[derive(Resource, Default)]
pub struct ShapeInputConsumer;

impl InputConsumer for ShapeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Only handle input if shape mode is active
        if !helpers::is_input_mode(input_state, InputMode::Shape) {
            return false;
        }
        
        // Handle mouse events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Shape: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement shape click handling
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Shape: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement shape drag handling
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Shape: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement shape release handling
                }
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        50 // Medium priority
    }
}

/// Input consumer for hyper tool
#[derive(Resource, Default)]
pub struct HyperInputConsumer;

impl InputConsumer for HyperInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, _input_state: &InputState) -> bool {
        // Only handle input if hyper mode is active
        if !helpers::is_input_mode(_input_state, InputMode::Hyper) {
            return false;
        }
        
        // Handle mouse events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Hyper: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement hyper click handling
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Hyper: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement hyper drag handling
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Hyper: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement hyper release handling
                }
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        50 // Medium priority
    }
}

/// Input consumer for text tool
#[derive(Resource)]
#[derive(Default)]
pub struct TextInputConsumer;

impl InputConsumer for TextInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, _input_state: &InputState) -> bool {
        // Only handle input if text mode is active
        if !helpers::is_input_mode(_input_state, InputMode::Text) {
            return false;
        }
        
        // Handle mouse and keyboard events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::KeyPress { .. } | 
            InputEvent::TextInput { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Text: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement text click handling
                }
            }
            InputEvent::KeyPress { key, modifiers } => {
                info!("Text: Processing key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement text key handling
            }
            InputEvent::TextInput { text } => {
                info!("Text: Processing text input: {}", text);
                // TODO: Implement text input handling
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        75 // High priority for text
    }
}

/// Input consumer for camera control
#[derive(Resource)]
#[derive(Default)]
pub struct CameraInputConsumer;

impl InputConsumer for CameraInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, _input_state: &InputState) -> bool {
        // Camera always handles certain events regardless of mode
        match event {
            InputEvent::MouseDrag { button, .. } if *button == MouseButton::Middle => true,
            InputEvent::MouseWheel { .. } => true,
            _ => false,
        }
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Middle {
                    info!("Camera: Processing middle mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement camera pan handling
                }
            }
            InputEvent::MouseWheel { delta } => {
                info!("Camera: Processing mouse wheel delta: {:?}", delta);
                // TODO: Implement camera zoom handling
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        10 // Low priority
    }
}

/// Input consumer for measurement tool
#[derive(Resource)]
#[derive(Default)]
pub struct MeasurementToolInputConsumer;

impl InputConsumer for MeasurementToolInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, _input_state: &InputState) -> bool {
        // Only handle input if measurement mode is active
        if !helpers::is_input_mode(_input_state, InputMode::Temporary) {
            return false;
        }
        
        // Handle mouse events
        matches!(event, 
            InputEvent::MouseClick { .. } | 
            InputEvent::MouseDrag { .. } | 
            InputEvent::MouseRelease { .. }
        )
    }
    
    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Measurement: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement measurement click handling
                }
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta: _, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Measurement: Processing mouse drag from {:?} to {:?} with modifiers {:?}", 
                          start_position, current_position, modifiers);
                    // TODO: Implement measurement drag handling
                }
            }
            InputEvent::MouseRelease { button, position, modifiers } => {
                if *button == MouseButton::Left {
                    info!("Measurement: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement measurement release handling
                }
            }
            _ => {}
        }
    }
    
    fn priority(&self) -> u32 {
        25 // Low-medium priority
    }
} 