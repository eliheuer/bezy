// A font editor made with the Bevy game engine.

mod theme;
mod components;

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use components::*;

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Darker gray background
        .add_plugins({
            let window_config = Window {
                title: "Bezy".into(),
                resolution: (1024., 768.).into(),
                ..default()
            };

            let window_plugin = WindowPlugin {
                primary_window: Some(window_config),
                ..default()
            };
    
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window_plugin)
        })
        .add_systems(Startup, (setup, spawn_grid, spawn_path_points, spawn_animated_sprite))
        .add_systems(Update, (button_system, animate_sprite))
        .run();
}