//! A font editor made with Rust and the Bevy game engine.

mod app;
mod camera;
mod draw;
mod hud;
mod setup;
mod stub;
mod theme;
mod toolbar;
mod debug_hud;

fn main() {
    app::create_app().run();
}
