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
                    handle_codepoint_cycling,
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
    app_state: ResMut<AppState>,
) {
    for event in events.read() {
        if let Err(e) = app_state.workspace.font.ufo.save(&event.path) {
            error!("failed to save file to {:?}: '{:?}'", event.path, e);
        }
    }
}

fn handle_new_glyph(
    mut events: EventReader<NewGlyphEvent>,
    _app_state: ResMut<AppState>,
) {
    for _ in events.read() {
        // TODO: Implement new glyph creation logic
        info!("New glyph creation requested");
    }
}

fn handle_delete_glyph(
    mut events: EventReader<DeleteGlyphEvent>,
    _app_state: ResMut<AppState>,
) {
    for event in events.read() {
        // TODO: Implement glyph deletion logic
        info!("Delete glyph requested for {:?}", event.glyph_name);
    }
}

fn handle_rename_glyph(
    mut events: EventReader<RenameGlyphEvent>,
    _app_state: ResMut<AppState>,
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
    _app_state: ResMut<AppState>,
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
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<crate::cameras::DesignCamera>,
    >,
    window_query: Query<&Window>,
) {
    for event in events.read() {
        // Log cycling event info
        info!("Received codepoint cycling event: {:?}", event.direction);

        // Check for a debug environment variable to minimize log output in normal use
        if std::env::var("BEZY_DEBUG").ok().is_some() {
            // Dump all glyph names in the font to help identify naming conventions
            let _glyph_names =
                crate::ufo::dump_all_glyph_names(&app_state.workspace.font.ufo);

            // Dump all available codepoints in the font (only for debugging)
            let all_codepoints =
                crate::ufo::get_all_codepoints(&app_state.workspace.font.ufo);
            info!("Found {} codepoints in the font", all_codepoints.len());

            if !all_codepoints.is_empty() {
                // Show a sample of codepoints for debugging
                let sample_size = std::cmp::min(all_codepoints.len(), 20);
                let mut sample = String::new();
                for i in 0..sample_size {
                    sample.push_str(&format!("U+{} ", all_codepoints[i]));
                    if i % 5 == 4 {
                        sample.push('\n');
                    }
                }
                if all_codepoints.len() > sample_size {
                    sample.push_str("\n...");
                }
                info!("Codepoint sample:\n{}", sample);
            }
        }

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

            // Get the glyph for the new codepoint
            if let Some(default_layer) =
                app_state.workspace.font.ufo.get_default_layer()
            {
                let codepoint_string = cli_args.get_codepoint_string();

                // Try first to find by Unicode
                if let Some(glyph_name) = crate::ufo::find_glyph_by_unicode(
                    &app_state.workspace.font.ufo,
                    &codepoint_string,
                ) {
                    let name = norad::GlyphName::from(glyph_name);
                    if let Some(glyph) = default_layer.get_glyph(&name) {
                        // Center the camera on the glyph
                        crate::cameras::center_camera_on_glyph(
                            glyph,
                            &app_state.workspace.info.metrics,
                            &mut camera_query,
                            &window_query,
                        );
                    }
                } else {
                    // Try by conventional name
                    let test_glyph = cli_args.get_test_glyph();
                    let glyph_name = norad::GlyphName::from(test_glyph);

                    if let Some(glyph) = default_layer.get_glyph(&glyph_name) {
                        // Center the camera on the glyph
                        crate::cameras::center_camera_on_glyph(
                            glyph,
                            &app_state.workspace.info.metrics,
                            &mut camera_query,
                            &window_query,
                        );
                    }
                }
            }
        } else {
            warn!("No codepoints found in the font");
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
            info!(
                "Detected Shift+= key combination, cycling to next codepoint"
            );
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Next,
            });
        }

        // Check for Shift+- (Minus) to move to previous codepoint
        if keyboard.just_pressed(KeyCode::Minus) {
            info!("Detected Shift+- key combination, cycling to previous codepoint");
            cycle_event.send(CycleCodepointEvent {
                direction: CodepointDirection::Previous,
            });
        }
    }
}
