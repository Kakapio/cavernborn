use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;

mod camera;
mod chunk;
mod debug;
mod map;
mod particle;
mod player;
mod utils;

use camera::{CameraPlugin, GameCamera};
use debug::DebugPlugin;
use map::{setup_map, update_chunks_around_player};
use player::PlayerPlugin;

// Component to mark UI controls text
#[derive(Component)]
struct ControlsText;

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
        .add_systems(Startup, (setup_map, show_controls))
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
fn show_controls(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ControlsText,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(Text::from("Controls:\n"));

            // Controls
            parent.spawn(Text::from(
                "WASD: Move camera (when camera follow is disabled)\n",
            ));
            parent.spawn(Text::from("Space: Toggle camera follow mode\n"));
            parent.spawn(Text::from("A/D: Move player\n"));
            parent.spawn(Text::from("Shift: Sprint\n"));

            // Debug section title
            parent.spawn(Text::from("\nDebug Controls:\n"));

            // Debug controls
            parent.spawn(Text::from("F3: Toggle debug visualization\n"));
            parent.spawn(Text::from(
                "F4: Toggle chunk visualization (outlines and coordinates)\n",
            ));
        });
}
