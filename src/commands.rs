use crate::data::AppState;
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

#[derive(Event)]
pub struct CycleCodepointEvent {
    pub direction: CodepointDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum CodepointDirection {
    Next,
    Previous,
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        info!("Registering command events, including CycleCodepointEvent");
        app.add_event::<OpenFileEvent>()
            .add_event::<SaveFileEvent>()
            .add_event::<SaveFileAsEvent>()
            .add_event::<NewGlyphEvent>()
            .add_event::<DeleteGlyphEvent>()
            .add_event::<RenameGlyphEvent>()
            .add_event::<OpenGlyphEditorEvent>()
            .add_event::<CycleCodepointEvent>()
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
                ),
            );
    }
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
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        if let Err(e) = app_state.workspace.font.ufo.save(&event.path) {
            error!("failed to save file to {:?}: '{:?}'", event.path, e);
        }
    }
}

fn handle_new_glyph(
    mut events: EventReader<NewGlyphEvent>,
    mut app_state: ResMut<AppState>,
) {
    for _ in events.read() {
        // TODO: Implement new glyph creation logic
        info!("New glyph creation requested");
    }
}

fn handle_delete_glyph(
    mut events: EventReader<DeleteGlyphEvent>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        // TODO: Implement glyph deletion logic
        info!("Delete glyph requested for {:?}", event.glyph_name);
    }
}

fn handle_rename_glyph(
    mut events: EventReader<RenameGlyphEvent>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        // TODO: Implement glyph renaming logic
        info!(
            "Rename glyph requested from {:?} to {:?}",
            event.old_name, event.new_name
        );
    }
}

fn handle_open_glyph_editor(
    mut events: EventReader<OpenGlyphEditorEvent>,
    mut app_state: ResMut<AppState>,
) {
    for event in events.read() {
        // TODO: Implement glyph editor opening logic
        info!("Open glyph editor requested for {:?}", event.glyph_name);
    }
}

fn handle_cycle_codepoint(
    mut events: EventReader<CycleCodepointEvent>,
    app_state: Res<AppState>,
    mut cli_args: ResMut<crate::cli::CliArgs>,
) {
    for event in events.read() {
        // Get the current codepoint
        let current_codepoint = cli_args.get_codepoint_string();
        info!(
            "Handling cycle codepoint event, current codepoint: {}",
            current_codepoint
        );

        // Get the next/previous codepoint based on the direction
        let new_codepoint = match event.direction {
            CodepointDirection::Next => {
                info!(
                    "Searching for next codepoint after {}",
                    current_codepoint
                );
                crate::ufo::find_next_codepoint(
                    &app_state.workspace.font.ufo,
                    &current_codepoint,
                )
            }
            CodepointDirection::Previous => {
                info!(
                    "Searching for previous codepoint before {}",
                    current_codepoint
                );
                crate::ufo::find_previous_codepoint(
                    &app_state.workspace.font.ufo,
                    &current_codepoint,
                )
            }
        };

        // Update the codepoint if found
        if let Some(cp) = new_codepoint {
            cli_args.set_codepoint(cp);
            info!("Switched to codepoint: {}", cli_args.get_codepoint_string());
        } else {
            warn!("No codepoints found in the font");
        }
    }
}
