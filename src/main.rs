use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;

mod particle;
mod world;

use world::generate_world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cavernborn".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, generate_world))
        .add_systems(Update, check_escape)
        .run();
}

fn setup(mut commands: Commands) {
    // Add a 2D camera
    commands.spawn(Camera2dBundle::default());
    info!("Starting game...");
}

fn check_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::default());
    }
}
