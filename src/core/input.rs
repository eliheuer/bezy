//! Centralized Input System for Bezy
//!
//! This module provides a unified input handling system that consolidates all
//! mouse, keyboard, and gamepad input processing. It ensures consistent
//! coordinate transformations and provides clear priority ordering for input
//! consumers.
//!
//! Key Features:
//! - Centralized input state management
//! - Consistent coordinate transformations using CursorInfo
//! - Clear input priority system
//! - Support for multiple input devices (mouse, keyboard, gamepad)
//! - Event-driven architecture for input consumers

use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::window::PrimaryWindow;
use crate::core::cursor::CursorInfo;
use crate::ui::panes::design_space::DPoint;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;

/// Plugin for the centralized input system
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputState>()
            .init_resource::<InputPriority>()
            .add_event::<InputEvent>()
            .add_systems(PreUpdate, (
                update_input_state,
                process_input_events,
            ).chain())
            .add_systems(Update, clear_input_events);
    }
}

/// Centralized input state that tracks all input devices
#[derive(Resource, Default, Debug)]
pub struct InputState {
    /// Mouse state
    pub mouse: MouseState,
    /// Keyboard state
    pub keyboard: KeyboardState,
    /// Gamepad state (future expansion)
    pub gamepad: GamepadState,
    /// Current input mode (affects how input is processed)
    pub mode: InputMode,
    /// Whether input is currently being consumed by UI
    pub ui_consuming: bool,
}

/// Mouse input state
#[derive(Debug)]
pub struct MouseState {
    /// Current position in screen coordinates
    pub screen_position: Option<Vec2>,
    /// Current position in design space coordinates
    pub design_position: Option<DPoint>,
    /// Mouse button states
    pub buttons: ButtonState<MouseButton>,
    /// Mouse wheel delta
    pub wheel_delta: Vec2,
    /// Whether mouse is moving
    pub is_moving: bool,
    /// Mouse movement delta
    pub movement_delta: Vec2,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            screen_position: None,
            design_position: None,
            buttons: ButtonState::default(),
            wheel_delta: Vec2::ZERO,
            is_moving: false,
            movement_delta: Vec2::ZERO,
        }
    }
}

/// Keyboard input state
#[derive(Debug)]
pub struct KeyboardState {
    /// Key states
    pub keys: ButtonState<KeyCode>,
    /// Modifier key states
    pub modifiers: ModifierState,
    /// Text input buffer (for text editing)
    pub text_buffer: String,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            keys: ButtonState::default(),
            modifiers: ModifierState::default(),
            text_buffer: String::new(),
        }
    }
}

/// Gamepad input state (placeholder for future expansion)
#[derive(Debug)]
pub struct GamepadState {
    /// Gamepad button states
    pub buttons: ButtonState<GamepadButton>,
    /// Analog stick positions
    pub sticks: Vec2,
    /// Trigger values
    pub triggers: Vec2,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: ButtonState::default(),
            sticks: Vec2::ZERO,
            triggers: Vec2::ZERO,
        }
    }
}

/// Generic button state tracking
#[derive(Debug)]
pub struct ButtonState<T: Copy + Eq + std::hash::Hash> {
    /// Currently pressed buttons
    pub pressed: std::collections::HashSet<T>,
    /// Buttons that were just pressed this frame
    pub just_pressed: std::collections::HashSet<T>,
    /// Buttons that were just released this frame
    pub just_released: std::collections::HashSet<T>,
}

impl<T: Copy + Eq + std::hash::Hash> Default for ButtonState<T> {
    fn default() -> Self {
        Self {
            pressed: std::collections::HashSet::new(),
            just_pressed: std::collections::HashSet::new(),
            just_released: std::collections::HashSet::new(),
        }
    }
}

/// Modifier key state
#[derive(Debug, Default, Clone)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Current input processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal editing mode
    Normal,
    /// Text editing mode
    Text,
    /// Selection mode
    Select,
    /// Pen tool mode
    Pen,
    /// Knife tool mode
    Knife,
    /// Shape tool mode
    Shape,
    /// Hyper tool mode
    Hyper,
    /// Temporary mode
    Temporary,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

/// Input priority levels for determining which system handles input
#[derive(Resource, Debug)]
pub struct InputPriority {
    /// Systems that handle input at the highest priority (UI, modals, etc.)
    pub high_priority: Vec<String>,
    /// Systems that handle input at normal priority (tools, editing)
    pub normal_priority: Vec<String>,
    /// Systems that handle input at low priority (fallbacks, defaults)
    pub low_priority: Vec<String>,
}

impl Default for InputPriority {
    fn default() -> Self {
        Self {
            high_priority: vec![
                "ui_interaction".to_string(),
                "modal_dialog".to_string(),
                "text_editor".to_string(),
            ],
            normal_priority: vec![
                "selection".to_string(),
                "pen_tool".to_string(),
                "knife_tool".to_string(),
                "shape_tool".to_string(),
                "hyper_tool".to_string(),
                "sort_interaction".to_string(),
            ],
            low_priority: vec![
                "camera_control".to_string(),
                "default_actions".to_string(),
            ],
        }
    }
}

/// Input events that can be consumed by systems
#[derive(Event, Debug, Clone)]
pub enum InputEvent {
    /// Mouse click event
    MouseClick {
        button: MouseButton,
        position: DPoint,
        modifiers: ModifierState,
    },
    /// Mouse release event
    MouseRelease {
        button: MouseButton,
        position: DPoint,
        modifiers: ModifierState,
    },
    /// Mouse drag event
    MouseDrag {
        button: MouseButton,
        start_position: DPoint,
        current_position: DPoint,
        delta: Vec2,
        modifiers: ModifierState,
    },
    /// Mouse move event
    MouseMove {
        position: DPoint,
        delta: Vec2,
    },
    /// Mouse wheel event
    MouseWheel {
        delta: Vec2,
        position: DPoint,
        modifiers: ModifierState,
    },
    /// Key press event
    KeyPress {
        key: KeyCode,
        modifiers: ModifierState,
    },
    /// Key release event
    KeyRelease {
        key: KeyCode,
        modifiers: ModifierState,
    },
    /// Text input event
    TextInput {
        text: String,
        modifiers: ModifierState,
    },
    /// Gamepad button press event (simplified for now)
    GamepadButtonPress {
        button: GamepadButton,
    },
    /// Gamepad button release event (simplified for now)
    GamepadButtonRelease {
        button: GamepadButton,
    },
    /// Gamepad analog input event (simplified for now)
    GamepadAnalog {
        stick: Vec2,
        triggers: Vec2,
    },
}

/// System to update the centralized input state
fn update_input_state(
    mut input_state: ResMut<InputState>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // Update UI consuming state
    input_state.ui_consuming = ui_hover_state.is_hovering_ui;

    // Update mouse state
    update_mouse_state(
        &mut input_state.mouse,
        &cursor,
        &mouse_button_input,
        &mut mouse_motion,
        &mut mouse_wheel,
    );

    // Update keyboard state
    update_keyboard_state(
        &mut input_state.keyboard,
        &keyboard_input,
    );

    // Update gamepad state (placeholder for future implementation)
    update_gamepad_state(&mut input_state.gamepad);
}

/// Update mouse state from Bevy input resources
fn update_mouse_state(
    mouse_state: &mut MouseState,
    cursor: &CursorInfo,
    mouse_button_input: &ButtonInput<MouseButton>,
    mouse_motion: &mut EventReader<MouseMotion>,
    mouse_wheel: &mut EventReader<MouseWheel>,
) {
    // Update position from cursor resource
    mouse_state.screen_position = cursor.screen_position;
    mouse_state.design_position = cursor.design_position;

    // Update button states
    mouse_state.buttons.pressed.clear();
    mouse_state.buttons.just_pressed.clear();
    mouse_state.buttons.just_released.clear();

    for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
        if mouse_button_input.pressed(button) {
            mouse_state.buttons.pressed.insert(button);
        }
        if mouse_button_input.just_pressed(button) {
            mouse_state.buttons.just_pressed.insert(button);
        }
        if mouse_button_input.just_released(button) {
            mouse_state.buttons.just_released.insert(button);
        }
    }

    // Update motion
    mouse_state.is_moving = false;
    mouse_state.movement_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_state.is_moving = true;
        mouse_state.movement_delta += motion.delta;
    }

    // Update wheel
    mouse_state.wheel_delta = Vec2::ZERO;
    for wheel in mouse_wheel.read() {
        mouse_state.wheel_delta += Vec2::new(wheel.x, wheel.y);
    }
}

/// Update keyboard state from Bevy input resources
fn update_keyboard_state(
    keyboard_state: &mut KeyboardState,
    keyboard_input: &ButtonInput<KeyCode>,
) {
    // Update key states
    keyboard_state.keys.pressed.clear();
    keyboard_state.keys.just_pressed.clear();
    keyboard_state.keys.just_released.clear();

    for key in [
        KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE,
        KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
        KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO,
        KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
        KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY,
        KeyCode::KeyZ, KeyCode::Digit0, KeyCode::Digit1, KeyCode::Digit2,
        KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Space,
        KeyCode::Enter, KeyCode::Tab, KeyCode::Backspace, KeyCode::Delete,
        KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight,
        KeyCode::Escape, KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4,
        KeyCode::F5, KeyCode::F6, KeyCode::F7, KeyCode::F8, KeyCode::F9,
        KeyCode::F10, KeyCode::F11, KeyCode::F12,
    ] {
        if keyboard_input.pressed(key) {
            keyboard_state.keys.pressed.insert(key);
        }
        if keyboard_input.just_pressed(key) {
            keyboard_state.keys.just_pressed.insert(key);
        }
        if keyboard_input.just_released(key) {
            keyboard_state.keys.just_released.insert(key);
        }
    }

    // Update modifier states
    keyboard_state.modifiers.shift = keyboard_input.pressed(KeyCode::ShiftLeft) 
        || keyboard_input.pressed(KeyCode::ShiftRight);
    keyboard_state.modifiers.ctrl = keyboard_input.pressed(KeyCode::ControlLeft) 
        || keyboard_input.pressed(KeyCode::ControlRight);
    keyboard_state.modifiers.alt = keyboard_input.pressed(KeyCode::AltLeft) 
        || keyboard_input.pressed(KeyCode::AltRight);
    keyboard_state.modifiers.super_key = keyboard_input.pressed(KeyCode::SuperLeft) 
        || keyboard_input.pressed(KeyCode::SuperRight);

    // Update text buffer
    keyboard_state.text_buffer.clear();
}

/// Update gamepad state (placeholder for future implementation)
fn update_gamepad_state(_gamepad_state: &mut GamepadState) {
    // TODO: Implement gamepad support
}

/// System to process input events and send them to consumers
fn process_input_events(
    input_state: Res<InputState>,
    mut input_events: EventWriter<InputEvent>,
) {
    // Process mouse events
    process_mouse_events(&input_state, &mut input_events);
    
    // Process keyboard events
    process_keyboard_events(&input_state, &mut input_events);
    
    // Process gamepad events
    process_gamepad_events(&input_state, &mut input_events);
}

/// Process mouse events and create InputEvent instances
fn process_mouse_events(
    input_state: &InputState,
    input_events: &mut EventWriter<InputEvent>,
) {
    let mouse = &input_state.mouse;
    let modifiers = &input_state.keyboard.modifiers;

    // Mouse click events
    for button in &mouse.buttons.just_pressed {
        if let Some(position) = mouse.design_position {
            input_events.write(InputEvent::MouseClick {
                button: *button,
                position,
                modifiers: modifiers.clone(),
            });
        }
    }

    // Mouse release events
    for button in &mouse.buttons.just_released {
        if let Some(position) = mouse.design_position {
            input_events.write(InputEvent::MouseRelease {
                button: *button,
                position,
                modifiers: modifiers.clone(),
            });
        }
    }

    // Mouse move events
    if mouse.is_moving {
        if let Some(position) = mouse.design_position {
            input_events.write(InputEvent::MouseMove {
                position,
                delta: mouse.movement_delta,
            });
        }
    }

    // Mouse wheel events
    if mouse.wheel_delta != Vec2::ZERO {
        if let Some(position) = mouse.design_position {
            input_events.write(InputEvent::MouseWheel {
                delta: mouse.wheel_delta,
                position,
                modifiers: modifiers.clone(),
            });
        }
    }
}

/// Process keyboard events and create InputEvent instances
fn process_keyboard_events(
    input_state: &InputState,
    input_events: &mut EventWriter<InputEvent>,
) {
    let keyboard = &input_state.keyboard;
    let modifiers = &keyboard.modifiers;

    // Key press events
    for key in &keyboard.keys.just_pressed {
        input_events.write(InputEvent::KeyPress {
            key: *key,
            modifiers: modifiers.clone(),
        });
    }

    // Key release events
    for key in &keyboard.keys.just_released {
        input_events.write(InputEvent::KeyRelease {
            key: *key,
            modifiers: modifiers.clone(),
        });
    }

    // Text input events
    if !keyboard.text_buffer.is_empty() {
        input_events.write(InputEvent::TextInput {
            text: keyboard.text_buffer.clone(),
            modifiers: modifiers.clone(),
        });
    }
}

/// Process gamepad events and create InputEvent instances
fn process_gamepad_events(
    _input_state: &InputState,
    _input_events: &mut EventWriter<InputEvent>,
) {
    // TODO: Implement gamepad event processing
}

/// System to clear input events at the end of each frame
fn clear_input_events(_input_events: EventWriter<InputEvent>) {
    // Events are automatically cleared by Bevy at the end of each frame
}

/// Helper trait for input consumers to check if they should handle input
pub trait InputConsumer {
    /// Check if this consumer should handle the given input event
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool;
    
    /// Handle the input event
    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState);
}

/// Helper functions for common input checks
pub mod helpers {
    use super::*;

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_pressed(input_state: &InputState, button: MouseButton) -> bool {
        input_state.mouse.buttons.pressed.contains(&button)
    }

    /// Check if a mouse button was just pressed
    pub fn is_mouse_just_pressed(input_state: &InputState, button: MouseButton) -> bool {
        input_state.mouse.buttons.just_pressed.contains(&button)
    }

    /// Check if a mouse button was just released
    pub fn is_mouse_just_released(input_state: &InputState, button: MouseButton) -> bool {
        input_state.mouse.buttons.just_released.contains(&button)
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(input_state: &InputState, key: KeyCode) -> bool {
        input_state.keyboard.keys.pressed.contains(&key)
    }

    /// Check if a key was just pressed
    pub fn is_key_just_pressed(input_state: &InputState, key: KeyCode) -> bool {
        input_state.keyboard.keys.just_pressed.contains(&key)
    }

    /// Check if a key was just released
    pub fn is_key_just_released(input_state: &InputState, key: KeyCode) -> bool {
        input_state.keyboard.keys.just_released.contains(&key)
    }

    /// Check if any modifier key is pressed
    pub fn has_modifier(input_state: &InputState) -> bool {
        let mods = &input_state.keyboard.modifiers;
        mods.shift || mods.ctrl || mods.alt || mods.super_key
    }

    /// Check if shift is pressed
    pub fn is_shift_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.shift
    }

    /// Check if ctrl is pressed
    pub fn is_ctrl_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.ctrl
    }

    /// Check if alt is pressed
    pub fn is_alt_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.alt
    }

    /// Get the current mouse position in design space
    pub fn get_mouse_design_position(input_state: &InputState) -> Option<DPoint> {
        input_state.mouse.design_position
    }

    /// Get the current mouse position in screen space
    pub fn get_mouse_screen_position(input_state: &InputState) -> Option<Vec2> {
        input_state.mouse.screen_position
    }

    /// Check if input is being consumed by UI
    pub fn is_ui_consuming(input_state: &InputState) -> bool {
        input_state.ui_consuming
    }

    /// Check if the current input mode matches the given mode
    pub fn is_input_mode(input_state: &InputState, mode: InputMode) -> bool {
        input_state.mode == mode
    }
} 