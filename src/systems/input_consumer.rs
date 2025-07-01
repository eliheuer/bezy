//! Input Consumer System
//!
//! This module provides the input consumer system that routes input events
//! to the appropriate handlers based on priority and current input mode.
//! It ensures that input is handled consistently and predictably across
//! the application.

use bevy::prelude::*;
use crate::core::input::{InputEvent, InputState, InputMode, InputConsumer, helpers};
use crate::core::cursor::CursorInfo;
use crate::ui::panes::design_space::DPoint;

/// Plugin for the input consumer system
pub struct InputConsumerPlugin;

impl Plugin for InputConsumerPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Update, route_input_events) // TODO: Refactor this system for Bevy compatibility
            .add_systems(Update, update_input_mode);
    }
}

/// System to route input events to appropriate consumers based on priority and mode
fn route_input_events(
    mut input_events: EventReader<InputEvent>,
    input_state: Res<InputState>,
    mut selection_consumer: SelectionInputConsumer,
    mut pen_consumer: PenInputConsumer,
    mut knife_consumer: KnifeInputConsumer,
    mut shape_consumer: ShapeInputConsumer,
    mut hyper_consumer: HyperInputConsumer,
    mut text_consumer: TextInputConsumer,
    mut camera_consumer: CameraInputConsumer,
    mut ui_consumer: UiInputConsumer,
) {
    for event in input_events.read() {
        // Skip if UI is consuming input
        if helpers::is_ui_consuming(&input_state) {
            if ui_consumer.should_handle_input(event, &input_state) {
                ui_consumer.handle_input(event, &input_state);
            }
            continue;
        }

        // Route based on input mode and priority
        let mut handled = false;

        // High priority consumers (UI, text editor)
        if text_consumer.should_handle_input(event, &input_state) {
            text_consumer.handle_input(event, &input_state);
            handled = true;
        }

        // Mode-specific consumers
        if !handled {
            match input_state.mode {
                InputMode::Select => {
                    if selection_consumer.should_handle_input(event, &input_state) {
                        selection_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Pen => {
                    if pen_consumer.should_handle_input(event, &input_state) {
                        pen_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Knife => {
                    if knife_consumer.should_handle_input(event, &input_state) {
                        knife_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Shape => {
                    if shape_consumer.should_handle_input(event, &input_state) {
                        shape_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Hyper => {
                    if hyper_consumer.should_handle_input(event, &input_state) {
                        hyper_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Text => {
                    if text_consumer.should_handle_input(event, &input_state) {
                        text_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
                InputMode::Normal | InputMode::Temporary => {
                    // In normal mode, try all consumers in priority order
                    if selection_consumer.should_handle_input(event, &input_state) {
                        selection_consumer.handle_input(event, &input_state);
                        handled = true;
                    } else if pen_consumer.should_handle_input(event, &input_state) {
                        pen_consumer.handle_input(event, &input_state);
                        handled = true;
                    } else if knife_consumer.should_handle_input(event, &input_state) {
                        knife_consumer.handle_input(event, &input_state);
                        handled = true;
                    } else if shape_consumer.should_handle_input(event, &input_state) {
                        shape_consumer.handle_input(event, &input_state);
                        handled = true;
                    } else if hyper_consumer.should_handle_input(event, &input_state) {
                        hyper_consumer.handle_input(event, &input_state);
                        handled = true;
                    }
                }
            }
        }

        // Low priority consumers (camera control, default actions)
        if !handled {
            if camera_consumer.should_handle_input(event, &input_state) {
                camera_consumer.handle_input(event, &input_state);
            }
        }
    }
}

/// System to update input mode based on current tool state
fn update_input_mode(
    mut input_state: ResMut<InputState>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
    text_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::text::TextModeActive>>,
) {
    // Determine current mode based on active tools
    if text_mode.as_ref().map_or(false, |m| m.0) {
        input_state.mode = InputMode::Text;
    } else if select_mode.as_ref().map_or(false, |m| m.0) {
        input_state.mode = InputMode::Select;
    } else if knife_mode.as_ref().map_or(false, |m| m.0) {
        input_state.mode = InputMode::Knife;
    } else {
        input_state.mode = InputMode::Normal;
    }
}

// Input Consumer Implementations

/// Selection tool input consumer
#[derive(Resource)]
pub struct SelectionInputConsumer;

impl InputConsumer for SelectionInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        match event {
            InputEvent::MouseClick { button, .. } => {
                *button == MouseButton::Left && !helpers::is_ui_consuming(input_state)
            }
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Left && !helpers::is_ui_consuming(input_state)
            }
            InputEvent::MouseRelease { button, .. } => {
                *button == MouseButton::Left && !helpers::is_ui_consuming(input_state)
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::KeyA | KeyCode::Escape) && !helpers::is_ui_consuming(input_state)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                debug!("Selection: Mouse click at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement selection click handling
            }
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                debug!("Selection: Mouse drag from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       start_position, current_position, delta, modifiers);
                // TODO: Implement selection drag handling
            }
            InputEvent::MouseRelease { position, modifiers, .. } => {
                debug!("Selection: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement selection release handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Selection: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement selection keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Pen tool input consumer
#[derive(Resource)]
pub struct PenInputConsumer;

impl InputConsumer for PenInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        match event {
            InputEvent::MouseClick { button, .. } => {
                *button == MouseButton::Left || *button == MouseButton::Right
            }
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseRelease { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::Escape | KeyCode::Enter | KeyCode::Tab)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers, .. } => {
                debug!("Pen: Mouse click {:?} at {:?} with modifiers {:?}", button, position, modifiers);
                // TODO: Implement pen click handling
            }
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                debug!("Pen: Mouse drag from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       start_position, current_position, delta, modifiers);
                // TODO: Implement pen drag handling
            }
            InputEvent::MouseRelease { position, modifiers, .. } => {
                debug!("Pen: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement pen release handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Pen: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement pen keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Knife tool input consumer
#[derive(Resource)]
pub struct KnifeInputConsumer;

impl InputConsumer for KnifeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        match event {
            InputEvent::MouseClick { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseRelease { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::Escape | KeyCode::Enter)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                debug!("Knife: Mouse click at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement knife click handling
            }
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                debug!("Knife: Mouse drag from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       start_position, current_position, delta, modifiers);
                // TODO: Implement knife drag handling
            }
            InputEvent::MouseRelease { position, modifiers, .. } => {
                debug!("Knife: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement knife release handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Knife: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement knife keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Shape tool input consumer
#[derive(Resource)]
pub struct ShapeInputConsumer;

impl InputConsumer for ShapeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        match event {
            InputEvent::MouseClick { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseRelease { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::Escape | KeyCode::Enter)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                debug!("Shape: Mouse click at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement shape click handling
            }
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                debug!("Shape: Mouse drag from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       start_position, current_position, delta, modifiers);
                // TODO: Implement shape drag handling
            }
            InputEvent::MouseRelease { position, modifiers, .. } => {
                debug!("Shape: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement shape release handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Shape: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement shape keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Hyper tool input consumer
#[derive(Resource)]
pub struct HyperInputConsumer;

impl InputConsumer for HyperInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        match event {
            InputEvent::MouseClick { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::MouseRelease { button, .. } => {
                *button == MouseButton::Left
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::Escape | KeyCode::Enter)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                debug!("Hyper: Mouse click at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement hyper click handling
            }
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                debug!("Hyper: Mouse drag from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       start_position, current_position, delta, modifiers);
                // TODO: Implement hyper drag handling
            }
            InputEvent::MouseRelease { position, modifiers, .. } => {
                debug!("Hyper: Mouse release at {:?} with modifiers {:?}", position, modifiers);
                // TODO: Implement hyper release handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Hyper: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement hyper keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Text editor input consumer
#[derive(Resource)]
pub struct TextInputConsumer;

impl InputConsumer for TextInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Text consumer handles all text input and text mode events
        matches!(event, InputEvent::TextInput { .. }) || 
        helpers::is_input_mode(input_state, InputMode::Text)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::TextInput { text, modifiers } => {
                debug!("Text: Text input '{}' with modifiers {:?}", text, modifiers);
                // TODO: Implement text input handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Text: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement text keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// Camera control input consumer
#[derive(Resource)]
pub struct CameraInputConsumer;

impl InputConsumer for CameraInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        match event {
            InputEvent::MouseWheel { .. } => true,
            InputEvent::MouseDrag { button, .. } => {
                *button == MouseButton::Middle || *button == MouseButton::Right
            }
            InputEvent::KeyPress { key, .. } => {
                matches!(key, KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD | 
                               KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight)
            }
            _ => false,
        }
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseWheel { delta, position, modifiers } => {
                debug!("Camera: Mouse wheel delta {:?} at {:?} with modifiers {:?}", delta, position, modifiers);
                // TODO: Implement camera zoom handling
            }
            InputEvent::MouseDrag { button, start_position, current_position, delta, modifiers } => {
                debug!("Camera: Mouse drag {:?} from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                       button, start_position, current_position, delta, modifiers);
                // TODO: Implement camera pan handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Camera: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement camera keyboard shortcuts
            }
            _ => {}
        }
    }
}

/// UI input consumer
#[derive(Resource)]
pub struct UiInputConsumer;

impl InputConsumer for UiInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // UI consumer handles all input when UI is consuming
        helpers::is_ui_consuming(input_state)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { button, position, modifiers, .. } => {
                debug!("UI: Mouse click {:?} at {:?} with modifiers {:?}", button, position, modifiers);
                // TODO: Implement UI click handling
            }
            InputEvent::KeyPress { key, modifiers } => {
                debug!("UI: Key press {:?} with modifiers {:?}", key, modifiers);
                // TODO: Implement UI keyboard shortcuts
            }
            _ => {}
        }
    }
}

// Example: Custom Tool Input Consumer
// This shows how to implement a custom input consumer for a new tool

/// Example: Custom measurement tool input consumer
#[derive(Resource)]
pub struct MeasurementToolInputConsumer {
    /// Track if we're currently measuring
    is_measuring: bool,
    /// Start position of the measurement
    start_position: Option<DPoint>,
    /// Current measurement distance
    current_distance: f32,
}

impl Default for MeasurementToolInputConsumer {
    fn default() -> Self {
        Self {
            is_measuring: false,
            start_position: None,
            current_distance: 0.0,
        }
    }
}

impl InputConsumer for MeasurementToolInputConsumer {
    fn should_handle_input(&self, _event: &InputEvent, input_state: &InputState) -> bool {
        // Only handle input when not in UI and in measurement mode
        if helpers::is_ui_consuming(input_state) {
            return false;
        }

        // Check if we're in measurement mode (you would add this to InputMode enum)
        // helpers::is_input_mode(input_state, InputMode::Measurement)

        // For now, always return false since we don't have a measurement mode
        false
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick { position, modifiers, .. } => {
                debug!("Measurement: Starting measurement at {:?} with modifiers {:?}", position, modifiers);
                
                // Start a new measurement
                self.is_measuring = true;
                self.start_position = Some(*position);
                self.current_distance = 0.0;
                
                // You could emit a custom event here to notify other systems
                // event_writer.send(MeasurementStarted { position: *position });
            }
            
            InputEvent::MouseDrag { start_position, current_position, delta, modifiers, .. } => {
                if self.is_measuring {
                    debug!("Measurement: Dragging from {:?} to {:?} (delta: {:?}) with modifiers {:?}", 
                           start_position, current_position, delta, modifiers);
                    
                    // Calculate current distance using Vec2 distance
                    if let Some(start) = self.start_position {
                        let start_vec = Vec2::new(start.x, start.y);
                        let current_vec = Vec2::new(current_position.x, current_position.y);
                        self.current_distance = start_vec.distance(current_vec);
                    }
                    
                    // You could emit a custom event here to update UI
                    // event_writer.send(MeasurementUpdated { distance: self.current_distance });
                }
            }
            
            InputEvent::MouseRelease { position, modifiers, .. } => {
                if self.is_measuring {
                    debug!("Measurement: Completed measurement at {:?} with modifiers {:?}", position, modifiers);
                    
                    // Finalize the measurement
                    if let Some(start) = self.start_position {
                        let start_vec = Vec2::new(start.x, start.y);
                        let end_vec = Vec2::new(position.x, position.y);
                        let final_distance = start_vec.distance(end_vec);
                        info!("Measurement completed: {:.2} units", final_distance);
                        
                        // You could emit a custom event here to save the measurement
                        // event_writer.send(MeasurementCompleted { 
                        //     start_position: start,
                        //     end_position: *position,
                        //     distance: final_distance 
                        // });
                    }
                    
                    // Reset state
                    self.is_measuring = false;
                    self.start_position = None;
                    self.current_distance = 0.0;
                }
            }
            
            InputEvent::KeyPress { key, modifiers } => {
                debug!("Measurement: Key press {:?} with modifiers {:?}", key, modifiers);
                
                match key {
                    KeyCode::Escape => {
                        // Cancel current measurement
                        if self.is_measuring {
                            debug!("Measurement: Cancelled by Escape key");
                            self.is_measuring = false;
                            self.start_position = None;
                            self.current_distance = 0.0;
                        }
                    }
                    KeyCode::Enter => {
                        // Complete current measurement
                        if self.is_measuring {
                            debug!("Measurement: Completed by Enter key");
                            self.is_measuring = false;
                            // You could emit a completion event here
                        }
                    }
                    _ => {}
                }
            }
            
            _ => {}
        }
    }
}

// Example: How to register a custom input consumer
// 
// In your plugin's build function:
//
// ```rust
// impl Plugin for MyToolPlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .init_resource::<MeasurementToolInputConsumer>()
//             .add_systems(Update, handle_measurement_events);
//     }
// }
//
// fn handle_measurement_events(
//     mut input_events: EventReader<InputEvent>,
//     input_state: Res<InputState>,
//     mut measurement_consumer: ResMut<MeasurementToolInputConsumer>,
// ) {
//     for event in input_events.read() {
//         if measurement_consumer.should_handle_input(event, &input_state) {
//             measurement_consumer.handle_input(event, &input_state);
//         }
//     }
// }
// ``` 