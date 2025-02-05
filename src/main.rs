//! A font editor made with Rust and the Bevy game engine.

mod app;
mod camera;
mod debug_hud;
mod draw;
mod hud;
mod setup;
mod stub;
mod theme;
mod toolbar;

fn main() {
    app::create_app().run();
}
