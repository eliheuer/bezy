use super::EditModeSystem;
use bevy::prelude::*;
use bevy::input::ButtonInput as Input;

#[derive(Component, Default)]
#[allow(dead_code)]
pub struct TextModeUI {
    text: String,
}

/// Resource to hold the text mode state
#[derive(Resource)]
#[allow(dead_code)]
pub struct TextModeState {
    text_entity: Option<Entity>,
}

impl Default for TextModeState {
    fn default() -> Self {
        Self {
            text_entity: None,
        }
    }
}

#[allow(dead_code)]
pub struct TextMode;

impl EditModeSystem for TextMode {
    fn update(&self, _commands: &mut Commands) {
        info!("Text mode active");
    }

    fn on_enter(&self) {
        info!("Entering text mode");
    }

    fn on_exit(&self) {
        info!("Exiting text mode");
    }
}

// System to set up text mode UI
#[allow(dead_code)]
pub fn setup_text_mode(mut commands: Commands) {
    commands.spawn((
        TextModeUI::default(),
        Node::default(),
        BackgroundColor(Color::rgb(0.9, 0.9, 0.9)),
    ));
}

// System to handle text input
#[allow(dead_code)]
pub fn handle_text_input(
    mut text_ui: Query<&mut TextModeUI>,
    keyboard: Res<Input<KeyCode>>,
) {
    let Ok(mut editor) = text_ui.get_single_mut() else { return };

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        if !editor.text.is_empty() {
            editor.text.pop();
        }
        return;
    }

    // Handle space
    if keyboard.just_pressed(KeyCode::Space) {
        editor.text.push(' ');
        return;
    }

    // Handle letters
    for key in keyboard.get_just_pressed() {
        match key {
            KeyCode::KeyA => editor.text.push('a'),
            KeyCode::KeyB => editor.text.push('b'),
            KeyCode::KeyC => editor.text.push('c'),
            KeyCode::KeyD => editor.text.push('d'),
            KeyCode::KeyE => editor.text.push('e'),
            KeyCode::KeyF => editor.text.push('f'),
            KeyCode::KeyG => editor.text.push('g'),
            KeyCode::KeyH => editor.text.push('h'),
            KeyCode::KeyI => editor.text.push('i'),
            KeyCode::KeyJ => editor.text.push('j'),
            KeyCode::KeyK => editor.text.push('k'),
            KeyCode::KeyL => editor.text.push('l'),
            KeyCode::KeyM => editor.text.push('m'),
            KeyCode::KeyN => editor.text.push('n'),
            KeyCode::KeyO => editor.text.push('o'),
            KeyCode::KeyP => editor.text.push('p'),
            KeyCode::KeyQ => editor.text.push('q'),
            KeyCode::KeyR => editor.text.push('r'),
            KeyCode::KeyS => editor.text.push('s'),
            KeyCode::KeyT => editor.text.push('t'),
            KeyCode::KeyU => editor.text.push('u'),
            KeyCode::KeyV => editor.text.push('v'),
            KeyCode::KeyW => editor.text.push('w'),
            KeyCode::KeyX => editor.text.push('x'),
            KeyCode::KeyY => editor.text.push('y'),
            KeyCode::KeyZ => editor.text.push('z'),
            _ => {}
        }
    }
}

// System to update text display
#[allow(dead_code)]
pub fn update_text_display(text_ui: Query<&TextModeUI>) {
    if let Ok(editor) = text_ui.get_single() {
        info!("Text: {}", editor.text);
    }
}

// System to clean up text mode UI
#[allow(dead_code)]
pub fn cleanup_text_mode(
    mut commands: Commands,
    state: Option<Res<TextModeState>>,
) {
    if let Some(state) = state {
        if let Some(entity) = state.text_entity {
            commands.entity(entity).despawn_recursive();
        }
        commands.remove_resource::<TextModeState>();
    }
}

// Add these systems to your app setup
#[allow(dead_code)]
pub fn register_text_mode(app: &mut App) {
    app.add_systems(Update, (
        handle_text_input,
        update_text_display,
    ));
}
