use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;

mod camera;
mod particle;
mod world;

use camera::{CameraPlugin, GameCamera};
use world::setup_world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cavernborn".into(),
                resolution: (1000., 1000.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CameraPlugin)
        .add_systems(Startup, (setup_world, show_controls))
        .add_systems(Update, (check_escape, debug_camera_info))
        .run();
}

fn check_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::default());
    }
}

// Debug system to display camera information when F3 is pressed
fn debug_camera_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Transform, &GameCamera)>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        if let Ok((transform, _)) = camera_query.get_single() {
            info!(
                "Camera Position: ({:.1}, {:.1}), Zoom: {:.2}",
                transform.translation.x,
                transform.translation.y,
                1.0 / transform.scale.x
            );
        }
    }
}

// Display control information when the game starts
fn show_controls() {
    info!("=== Camera Controls ===");
    info!("WASD: Move camera");
    info!("Q/E or Mouse Wheel: Zoom in/out");
    info!("F3: Show camera position and zoom level");
    info!("ESC: Exit game");
    info!("=====================");
}
