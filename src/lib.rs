use bevy::app::AppExit;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use utils::debug;
use world::camera;

pub mod particle;
pub mod player;
pub mod render;
pub mod simulation;
pub mod utils;
pub mod world;

pub mod testing;

use crate::world::MapPlugin;
use camera::{CameraPlugin, GameCamera};
use debug::DebugPlugin;
use player::PlayerPlugin;
use render::map_renderer::MapRendererPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cavernborn".into(),
                resolution: (1600.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MapPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(MapRendererPlugin)
        .add_systems(Startup, show_controls)
        .add_systems(Update, (check_escape, debug_camera_info))
        .run();
}

fn check_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::default());
    }
}

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

fn show_controls(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Relative,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(Text::from("Controls:\n"));
            parent.spawn(Text::from("Space: Toggle camera follow mode\n"));
            parent.spawn(Text::from("WASD: Move player/camera\n"));
            parent.spawn(Text::from("Shift: Speed up camera when disconnected\n"));
            parent.spawn(Text::from("\nDebug Controls:\n"));
            parent.spawn(Text::from("F3: Toggle debug visualization\n"));
            parent.spawn(Text::from(
                "F4: Toggle chunk visualization (highlights and coordinates)\n",
            ));
            parent.spawn(Text::from("F5: Toggle chunk outlines\n"));
        });
}
