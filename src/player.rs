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
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.2, 0.8), // Blue color
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0), // Start at origin, above terrain
            ..default()
        },
        // Adding a simple collider component (we'll just use this for identification)
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
    debug_mode: Res<DebugMode>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    // Don't move the player if debug mode is enabled
    if debug_mode.enabled {
        return;
    }

    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = 0.0;

        // AD movement (horizontal only)
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += 1.0;
        }

        // Move player horizontally
        if direction != 0.0 {
            let delta = direction * PLAYER_SPEED * time.delta_seconds();
            transform.translation.x += delta;

            // Log player movement
            debug!("Player moved: x={:.1}", transform.translation.x);
        }
    }
}

// Toggle debug mode system
fn toggle_debug_mode(keyboard: Res<ButtonInput<KeyCode>>, mut debug_mode: ResMut<DebugMode>) {
    if keyboard.just_pressed(KeyCode::F1) {
        debug_mode.enabled = !debug_mode.enabled;
        if debug_mode.enabled {
            info!("Debug mode ENABLED: Camera detached from player, player movement disabled");
        } else {
            info!("Debug mode DISABLED: Camera follows player, player movement enabled");
        }
    }
}
