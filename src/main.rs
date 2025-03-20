//! A font editor made with Rust, the Bevy game engine, and some Linebender crates.

// “The enjoyment of one's tools is an essential ingredient of successful work."
//
// — Donald Knuth

mod access_toolbar;
mod app;
mod cameras;
mod checkerboard;
mod coord_pane;
mod cli;
mod commands;
mod data;
mod debug;
mod design_space;
mod draw;
mod edit_mode_toolbar;
mod edit_session;
mod edit_type;
mod glyph_pane;
mod hud;
mod logger;
mod plugins;
mod quadrant;
mod selection;
mod settings;
mod setup;
mod tests;
mod text_editor;
mod theme;
mod ufo;
mod ui_interaction;
mod undo;
mod undo_plugin;
mod virtual_font;

fn main() {
    // Parse command line arguments
    let cli_args = cli::CliArgs::parse_args();
    // Create and run the app with the CLI arguments
    app::create_app(cli_args).run();
}
