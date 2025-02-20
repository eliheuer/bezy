//! A font editor made with Rust and the Bevy game engine.

mod app;
mod cameras;
mod data;
mod debug;
mod debug_hud;
mod design_space;
mod draw;
mod grid;
mod hud;
mod main_toolbar;
mod setup;
mod text_editor;
mod theme;
mod toolbar;
mod ufo;
mod tests;
mod virtual_font;
mod world_space;

fn main() {
    app::create_app().run();
}
