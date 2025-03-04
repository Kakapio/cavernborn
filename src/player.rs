use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

// Constants for player
pub const PLAYER_SIZE: u32 = 20;
pub const PLAYER_SPEED: f32 = 150.0;

// Player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMode>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, spawn_player)
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, player_movement)
            .add_systems(Update, toggle_debug_mode)
            .add_systems(Update, update_fps_counter);
    }
}

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FpsText;

// Resources
#[derive(Resource, Default)]
pub struct DebugMode {
    pub enabled: bool,
}

// Spawn the player
fn spawn_player(mut commands: Commands) {
    info!("Spawning player");

    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8), // Blue color
            custom_size: Some(Vec2::new(PLAYER_SIZE as f32, PLAYER_SIZE as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0), // Start at origin, above terrain
        Collider,
    ));
}

// Simple collider component (for identification)
#[derive(Component)]
pub struct Collider;

// Player movement system
fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec2::ZERO;

        // AD movement (horizontal)
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // WS movement (vertical)
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }

        // Move player
        if direction != Vec2::ZERO {
            let normalized_direction = direction.normalize_or_zero();
            let delta = normalized_direction * PLAYER_SPEED * time.delta_secs();
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;

            // Log player movement
            debug!(
                "Player moved: x={:.1}, y={:.1}",
                transform.translation.x, transform.translation.y
            );
        }
    }
}

// Toggle debug mode system
fn toggle_debug_mode(keyboard: Res<ButtonInput<KeyCode>>, mut debug_mode: ResMut<DebugMode>) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_mode.enabled = !debug_mode.enabled;
        if debug_mode.enabled {
            info!("Debug visualization: ENABLED - Use F4 and F5 to toggle specific features");
        } else {
            info!("Debug visualization: DISABLED");
        }
    }
}

// Setup FPS counter
fn setup_fps_counter(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            Visibility::Hidden, // Start hidden
        ))
        .with_children(|parent| {
            parent.spawn((FpsText, Text::from("FPS: 0")));
        });
}

// Update FPS counter
fn update_fps_counter(
    debug_mode: Res<DebugMode>,
    diagnostics: Res<DiagnosticsStore>,
    mut fps_query: Query<(&mut Text, &mut Visibility), With<FpsText>>,
    mut parent_query: Query<&mut Visibility, (Without<FpsText>, With<Node>)>,
) {
    // Update parent node visibility
    for mut visibility in &mut parent_query {
        *visibility = if debug_mode.enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Only update text if debug mode is enabled
    if debug_mode.enabled {
        for (mut text, _) in &mut fps_query {
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.smoothed() {
                    // Update the FPS text with the new value
                    *text = Text::from(format!("FPS: {:.1}", value));
                }
            }
        }
    }
}
