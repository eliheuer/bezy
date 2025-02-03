// A font editor made with the Bevy game engine.

mod theme;
mod components;

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use components::*;

/// Main entry point for the Bezy font editor application.
/// Sets up the window and initializes the Bevy app with required plugins and systems.
fn main() {
    App::new()
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bezy".into(),
                    resolution: (1024., 768.).into(),
                    ..default()
                }),
                ..default()
            }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Darker gray background
        .add_systems(Startup, (setup, spawn_grid, spawn_path_points, spawn_animated_sprite))
        .add_systems(Update, (button_system, animate_sprite))
        .run();
}