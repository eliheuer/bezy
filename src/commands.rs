use bevy::prelude::*;
use norad::GlyphName;
use std::path::PathBuf;
use crate::data::AppState;

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

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OpenFileEvent>()
            .add_event::<SaveFileEvent>()
            .add_event::<SaveFileAsEvent>()
            .add_event::<NewGlyphEvent>()
            .add_event::<DeleteGlyphEvent>()
            .add_event::<RenameGlyphEvent>()
            .add_event::<OpenGlyphEditorEvent>()
            .add_systems(Update, (
                handle_open_file,
                handle_save_file,
                handle_save_file_as,
                handle_new_glyph,
                handle_delete_glyph,
                handle_rename_glyph,
                handle_open_glyph_editor,
            ));
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
        info!("Rename glyph requested from {:?} to {:?}", event.old_name, event.new_name);
    }
}

fn handle_open_glyph_editor(
    mut events: EventReader<OpenGlyphEditorEvent>,
    app_state: Res<AppState>,
) {
    for event in events.read() {
        // TODO: Implement glyph editor opening logic
        info!("Open glyph editor requested for {:?}", event.glyph_name);
    }
}
