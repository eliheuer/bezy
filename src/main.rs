//! A font editor made with Rust and the Bevy game engine.

mod app;
mod camera;
mod debug_hud;
mod design_space;
mod draw;
mod hud;
mod main_toolbar;
mod setup;
mod theme;
mod toolbar;
mod ufo;
mod world_space;

fn main() {
    app::create_app().run();
}
