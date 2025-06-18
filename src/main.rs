use bevy::prelude::*;
use clap::Parser;

mod core;
mod editing;
mod rendering;
mod systems;
mod ui;
use ui::theme::BACKGROUND_COLOR;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "assets/fonts/bezy-grotesk-regular.ufo")]
    load_ufo: Option<String>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_non_send_resource::<core::state::AppState>()
        .add_plugins((
            DefaultPlugins,
            rendering::cameras::CameraPlugin,
            rendering::checkerboard::CheckerboardPlugin,
            editing::undo_plugin::UndoPlugin,
            editing::sort_plugin::SortPlugin,
            ui::panes::design_space::DesignSpacePlugin,
        ))
        .add_systems(Startup, load_ufo_font)
        .add_systems(Update, exit_on_esc)
        .run();
}

// System to exit the application when the Escape key is pressed
fn exit_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

fn load_ufo_font(mut app_state: NonSendMut<core::state::AppState>) {
    let args = std::env::args().collect::<Vec<_>>();
    let args = Args::parse_from(args);
    
    // Now we always have a path (either provided or default)
    if let Some(path) = args.load_ufo {
        match norad::Font::load(&path) {
            Ok(font) => {
                info!("Successfully loaded UFO font from: {}", path);
                app_state.set_font(font);
            }
            Err(e) => {
                error!("Failed to load UFO font from {}: {}", path, e);
            }
        }
    }
} 