use bevy::prelude::*;

// Constants for player
pub const PLAYER_SIZE: f32 = 20.0;
pub const PLAYER_SPEED: f32 = 150.0;

// Player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMode>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement, toggle_debug_mode));
    }
}

// Components
#[derive(Component)]
pub struct Player;

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
            custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
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
