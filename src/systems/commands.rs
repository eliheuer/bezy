//! commands.rs - Event-based command system for the font editor
//!
//! This file defines the application's command system using Bevy's event system:
//! 1. Event structs define different actions (file operations, glyph management)
//! 2. Handler functions process these events and update application state
//! 3. CommandsPlugin registers all events and connects handlers to the application
//!
//! To add new functionality, define a new event struct and corresponding handler,
//! then register both in the CommandsPlugin::build method.

use crate::core::data::AppState;
use bevy::prelude::*;
use norad::GlyphName;
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
    pub glyph_name: GlyphName,
}

#[derive(Event)]
pub struct RenameGlyphEvent {
    pub old_name: GlyphName,
    pub new_name: GlyphName,
}

#[derive(Event)]
pub struct OpenGlyphEditorEvent {
    pub glyph_name: GlyphName,
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
        match norad::Ufo::load(&event.path) {
            Ok(ufo) => app_state.set_font(ufo, Some(event.path.clone())),
            Err(e) => error!("failed to open file {:?}: '{:?}'", event.path, e),
        }
    }
}

fn handle_save_file(
    mut events: EventReader<SaveFileEvent>,
    mut app_state: ResMut<AppState>,
) {
    for _ in events.read() {
        if let Err(e) = app_state.workspace.save() {
            error!("saving failed: '{}'", e);
        }
    }
}

fn handle_save_file_as(
    mut events: EventReader<SaveFileAsEvent>,
    app_state: ResMut<AppState>,
) {
    for event in events.read() {
        if let Err(e) = app_state.workspace.font.ufo.save(&event.path) {
            error!("failed to save file to {:?}: '{:?}'", event.path, e);
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
    mut glyph_navigation: ResMut<crate::core::data::GlyphNavigation>,
    app_state: Res<AppState>,
) {
    for event in event_reader.read() {
        debug!("Received codepoint cycling event: {:?}", event.direction);

        // Get available codepoints using the io::ufo module functions
        let available_codepoints = crate::io::ufo::get_all_codepoints(&app_state.workspace.font.ufo);
        let current_codepoint = glyph_navigation.get_codepoint_string();

        if available_codepoints.is_empty() {
            debug!("No codepoints found in font");
            return;
        }

        // Calculate next codepoint based on direction
        let next_codepoint = match event.direction {
            CodepointDirection::Next => {
                crate::io::ufo::find_next_codepoint(&app_state.workspace.font.ufo, &current_codepoint)
            }
            CodepointDirection::Previous => {
                crate::io::ufo::find_previous_codepoint(&app_state.workspace.font.ufo, &current_codepoint)
            }
        };

        // Set the new codepoint if found
        if let Some(new_codepoint) = next_codepoint {
            glyph_navigation.set_codepoint(new_codepoint);
            debug!("Switched to codepoint: {}", glyph_navigation.get_codepoint_string());
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
    glyph_navigation: Res<crate::core::data::GlyphNavigation>,
) {
    for event in event_reader.read() {
        debug!("Handling CreateContourEvent");

        // Get the glyph name first
        if let Some(glyph_name) =
            glyph_navigation.find_glyph(&app_state.workspace.font.ufo)
        {
            let glyph_name = glyph_name.clone(); // Clone the glyph name

            // Get mutable access to the font
            let font_obj = app_state.workspace.font_mut();

            // Get the current glyph
            if let Some(default_layer) = font_obj.ufo.get_default_layer_mut() {
                if let Some(glyph) = default_layer.get_glyph_mut(&glyph_name) {
                    // Get or create the outline
                    let outline = glyph.outline.get_or_insert_with(|| {
                        norad::glyph::Outline {
                            contours: Vec::new(),
                            components: Vec::new(),
                        }
                    });

                    // Add the new contour
                    outline.contours.push(event.contour.clone());
                    debug!("Added new contour to glyph {}", glyph_name);
                } else {
                    warn!("Could not find glyph for contour creation");
                }
            } else {
                warn!("No default layer found for contour creation");
            }
        } else {
            warn!("No current glyph selected for contour creation");
        }
    }
}
