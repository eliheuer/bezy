//! A font editor made with Rust and the Bevy game engine.

mod app;
mod cameras;
mod cli;
mod data;
mod debug;
mod debug_hud;
mod design_space;
mod draw;
mod grid;
mod hud;
mod main_toolbar;
mod setup;
mod tests;
mod text_editor;
mod theme;
mod toolbar;
mod ufo;
mod virtual_font;
mod world_space;

fn main() {
    // Parse command line arguments
    let cli_args = cli::CliArgs::parse_args();

    // Create and run the app with the CLI arguments
    app::create_app(cli_args).run();
}
