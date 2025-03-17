//! A font editor made with Rust and the Bevy game engine.
mod app;
mod cameras;
mod checkerboard;
mod cli;
mod commands;
mod access_toolbar;
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
mod selection;
mod settings;
mod setup;
mod tests;
mod text_editor;
mod theme;
mod ufo;
mod undo;
mod undo_plugin;
mod virtual_font;

fn main() {
    // Parse command line arguments
    let cli_args = cli::CliArgs::parse_args();
    // Create and run the app with the CLI arguments
    app::create_app(cli_args).run();
}
