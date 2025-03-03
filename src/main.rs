use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;

mod camera;
mod chunk;
mod particle;
mod player;
mod world;

use camera::{CameraPlugin, GameCamera};
use player::PlayerPlugin;
use world::{setup_world, update_chunks_around_player};

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
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, (setup_world, show_controls))
        .add_systems(
            Update,
            (check_escape, debug_camera_info, update_chunks_around_player),
        )
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
    camera_query: Query<(&Transform, &OrthographicProjection, &GameCamera)>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        if let Ok((transform, projection, _)) = camera_query.get_single() {
            info!(
                "Camera Position: ({:.1}, {:.1}), Zoom: {:.2}x",
                transform.translation.x, transform.translation.y, projection.scale
            );
        }
    }
}

// Display control information when the game starts
fn show_controls() {
    info!("=== Controls ===");

    info!("--- Player Controls ---");
    info!("A/D: Move player left/right");

    info!("--- Camera Controls ---");
    info!("WASD: Move camera (in debug mode)");
    info!("SHIFT + WASD: Move camera faster");
    info!("Q/E or Mouse Wheel: Zoom in/out");

    info!("--- Debug Controls ---");
    info!("F1: Toggle debug mode (separate camera from player)");
    info!("F3: Show camera position and zoom level");

    info!("--- System ---");
    info!("ESC: Exit game");
    info!("=====================");
}
