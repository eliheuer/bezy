//! commands.rs - Event-based command system for the font editor
//!
//! This file defines the application's command system using Bevy's event system:
//! 1. Event structs define different actions (file operations, glyph management)
//! 2. Handler functions process these events and update application state
//! 3. CommandsPlugin registers all events and connects handlers to the application
//!
//! To add new functionality, define a new event struct and corresponding handler,
//! then register both in the CommandsPlugin::build method.

use crate::core::state::{AppState, GlyphNavigation};
// BezyResult not used in current implementation
use bevy::prelude::*;
// Using String for glyph names in current norad version
use std::path::PathBuf;

#[derive(Event)]
pub struct OpenFileEvent {
    pub path: PathBuf,
}

#[derive(Event)]
pub struct SaveFileEvent;

#[derive(Event)]
pub struct SaveFileAsEvent {
    pub path: PathBuf,
}

#[derive(Event)]
pub struct NewGlyphEvent;

#[derive(Event)]
pub struct DeleteGlyphEvent {
    pub glyph_name: String,
}

#[derive(Event)]
pub struct RenameGlyphEvent {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Event)]
pub struct OpenGlyphEditorEvent {
    pub glyph_name: String,
}

#[derive(Event, Debug)]
pub struct CycleCodepointEvent {
    pub direction: CodepointDirection,
}

#[derive(Debug)]
pub enum CodepointDirection {
    Next,
    Previous,
}

#[derive(Event)]
pub struct CreateContourEvent {
    pub contour: norad::Contour,
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        register_event_handlers(app);
    }
}

fn register_event_handlers(app: &mut App) {
    debug!("Registering command events, including CycleCodepointEvent");
    app.add_event::<OpenFileEvent>()
        .add_event::<SaveFileEvent>()
        .add_event::<SaveFileAsEvent>()
        .add_event::<NewGlyphEvent>()
        .add_event::<DeleteGlyphEvent>()
        .add_event::<RenameGlyphEvent>()
        .add_event::<OpenGlyphEditorEvent>()
        .add_event::<CycleCodepointEvent>()
        .add_event::<CreateContourEvent>()
        .add_systems(
            Update,
            (
                handle_open_file,
                handle_save_file,
                handle_save_file_as,
                handle_new_glyph,
                handle_delete_glyph,
                handle_rename_glyph,
                handle_open_glyph_editor,
                handle_cycle_codepoint,
                handle_create_contour,
                handle_codepoint_cycling,
                handle_save_shortcuts,
            ),
        );
}

fn handle_open_file(
    mut events: EventReader<OpenFileEvent>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        match app_state.load_font_from_path(event.path.clone()) {
            Ok(_) => {
                info!("Successfully loaded font from {:?}", event.path);
            }
            Err(e) => {
                error!("Failed to open file {:?}: {}", event.path, e);
            }
        }
    }
}

fn handle_save_file(
    mut events: EventReader<SaveFileEvent>,
    mut app_state: ResMut<AppState>,
) {
    for _ in events.read() {
        match app_state.save_font() {
            Ok(_) => {
                info!("Font saved successfully");
            }
            Err(e) => {
                error!("Saving failed: {}", e);
            }
        }
    }
}

fn handle_save_file_as(
    mut events: EventReader<SaveFileAsEvent>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        match app_state.save_font_as(event.path.clone()) {
            Ok(_) => {
                info!("Font saved to {:?}", event.path);
            }
            Err(e) => {
                error!("Failed to save file to {:?}: {}", event.path, e);
            }
        }
    }
}

fn handle_new_glyph(
    mut event_reader: EventReader<NewGlyphEvent>,
    _app_state: ResMut<AppState>,
) {
    for _event in event_reader.read() {
        debug!("New glyph creation requested");
        // Implementation here when needed
    }
}

fn handle_delete_glyph(
    mut event_reader: EventReader<DeleteGlyphEvent>,
    _app_state: ResMut<AppState>,
) {
    for event in event_reader.read() {
        debug!("Delete glyph requested for {:?}", event.glyph_name);
        // Implementation here when needed
    }
}

fn handle_rename_glyph(
    mut event_reader: EventReader<RenameGlyphEvent>,
    _app_state: ResMut<AppState>,
) {
    for event in event_reader.read() {
        debug!(
            "Rename glyph requested: {:?} -> {:?}",
            event.old_name, event.new_name
        );
        // Implementation here when needed
    }
}

fn handle_open_glyph_editor(
    mut event_reader: EventReader<OpenGlyphEditorEvent>,
    _app_state: ResMut<AppState>,
) {
    for event in event_reader.read() {
        debug!("Open glyph editor requested for {:?}", event.glyph_name);
        // Implementation here when needed
    }
}

fn handle_cycle_codepoint(
    mut event_reader: EventReader<CycleCodepointEvent>,
    mut glyph_navigation: ResMut<GlyphNavigation>,
    app_state: Res<AppState>,
) {
    for event in event_reader.read() {
        debug!("Received codepoint cycling event: {:?}", event.direction);

        // Check if we have any codepoints available
        let available_codepoints = crate::core::state::get_all_codepoints(&app_state);
        if available_codepoints.is_empty() {
            debug!("No codepoints found in font");
            return;
        }

        let current_codepoint = glyph_navigation.get_codepoint_string();

        // Calculate next codepoint using cycle function
        let next_codepoint = match event.direction {
            CodepointDirection::Next => {
                crate::core::state::cycle_codepoint_in_list(
                    Some(current_codepoint),
                    &app_state,
                    crate::core::state::CycleDirection::Next,
                )
            }
            CodepointDirection::Previous => {
                crate::core::state::cycle_codepoint_in_list(
                    Some(current_codepoint),
                    &app_state,
                    crate::core::state::CycleDirection::Previous,
                )
            }
        };

        // Set the new codepoint if found
        if let Some(new_codepoint) = next_codepoint {
            glyph_navigation.set_codepoint(new_codepoint);
            debug!(
                "Switched to codepoint: {}",
                glyph_navigation.get_codepoint_string()
            );
        } else {
            debug!("No next/previous codepoint found");
        }
    }
}

/// System to handle keyboard shortcuts to cycle through codepoints
pub fn handle_codepoint_cycling(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cycle_event: EventWriter<CycleCodepointEvent>,
) {
    // Check for Shift+Plus to cycle forward through codepoints
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if shift_pressed {
        // Check for Shift+= (Plus) to move to next codepoint
        if keyboard.just_pressed(KeyCode::Equal) {
            debug!(
                "Detected Shift+= key combination, cycling to next codepoint"
            );
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Next,
            });
        }

        // Check for Shift+- (Minus) to move to previous codepoint
        if keyboard.just_pressed(KeyCode::Minus) {
            debug!("Detected Shift+- key combination, cycling to previous codepoint");
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Previous,
            });
        }
    }
}

/// System to handle keyboard shortcuts for saving the font
///
/// This system watches for Command+S (macOS) or Ctrl+S (Windows/Linux)
/// and triggers a save operation when detected
pub fn handle_save_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut save_event: EventWriter<SaveFileEvent>,
) {
    // Check for Command (macOS) or Control (Windows/Linux)
    let modifier_pressed = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    // If modifier is pressed and S is just pressed, trigger save
    if modifier_pressed && keyboard.just_pressed(KeyCode::KeyS) {
        debug!("Detected Command+S / Ctrl+S key combination, saving font");
        save_event.send(SaveFileEvent);
    }
}

/// Handler for adding a new contour to the current glyph
fn handle_create_contour(
    mut event_reader: EventReader<CreateContourEvent>,
    mut app_state: ResMut<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
) {
    for event in event_reader.read() {
        debug!("Handling CreateContourEvent");

        // Get the glyph name first
        if let Some(glyph_name) = glyph_navigation.find_glyph(&app_state) {
            // Try to add the contour to the glyph
            // Note: This will need to be implemented when we have the full glyph editing system
            debug!("Would add contour to glyph: {}", glyph_name);
            // TODO: Implement contour creation when glyph editing is ready
        } else {
            warn!("No current glyph selected for contour creation");
        }
    }
} 