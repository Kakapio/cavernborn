use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;

mod camera;
mod chunk;
mod debug;
mod particle;
mod player;
mod world;

use camera::{CameraPlugin, GameCamera};
use debug::DebugPlugin;
use player::PlayerPlugin;
use world::{setup_world, update_chunks_around_player};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cavernborn".into(),
                resolution: (1600.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DebugPlugin)
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

// Debug system to display camera information when I key is pressed in debug mode
fn debug_camera_info(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<player::DebugMode>,
    camera_query: Query<(&Transform, &OrthographicProjection, &GameCamera)>,
) {
    if debug_mode.enabled && keyboard.just_pressed(KeyCode::KeyI) {
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
    info!("Space: Toggle camera follow mode");
    info!("WASD: Move camera (when camera follow is disabled)");
    info!("Q/E or Mouse Wheel: Zoom in/out");
    info!("Shift + WASD: Move camera faster");

    info!("--- Debug Controls ---");
    info!("F3: Toggle debug mode");
    info!("F4: Toggle chunk visualization (when in debug mode)");
    info!("F5: Toggle chunk coordinates (when in debug mode)");
    info!("I: Show camera information (when in debug mode)");

    info!("--- System Controls ---");
    info!("Escape: Exit game");
    info!("=====================");
}
