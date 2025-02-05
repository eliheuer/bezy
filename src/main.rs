// A font editor made with Rust and the Bevy game engine.
//     +y
//      ↑
// -x ←-+-> +x
//      ↓
//     -y

mod app;
mod camera;
mod draw;
mod hud;
mod setup;
mod stub;
mod theme;

fn main() {
    app::create_app().run();
}
