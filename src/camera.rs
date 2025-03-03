use crate::player::Player;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// Plugin to handle camera systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollowMode>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    camera_movement,
                    camera_zoom,
                    camera_follow_player,
                    toggle_camera_follow,
                ),
            );
    }
}

// Resource to track whether the camera follows the player
#[derive(Resource)]
pub struct CameraFollowMode {
    pub following: bool,
}

impl Default for CameraFollowMode {
    fn default() -> Self {
        Self { following: true } // Follow by default
    }
}

// Component to track camera state
#[derive(Component)]
pub struct GameCamera {
    pub speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

// Setup the camera with initial position and settings
fn setup_camera(mut commands: Commands) {
    info!("Setting up game camera with WASD controls and zoom");

    let default_zoom = 1.0;

    // Using OrthographicProjection directly allows us to modify it for zoom
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 999.9), // Start at origin where player will spawn
            projection: OrthographicProjection {
                scale: default_zoom,
                ..default()
            },
            ..default()
        },
        GameCamera {
            speed: 300.0, // Units per second
            zoom_speed: 1.0,
            min_zoom: 0.1, // Allow zooming out quite far
            max_zoom: 5.0, // Allow zooming in quite close
        },
    ));

    info!(
        "Camera initialized at position (0.0, 0.0) with default zoom {}",
        default_zoom
    );
}

// System to toggle camera follow mode with the Space key
fn toggle_camera_follow(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_follow: ResMut<CameraFollowMode>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        camera_follow.following = !camera_follow.following;
        info!(
            "Camera follow mode: {}",
            if camera_follow.following {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
    }
}

// System to handle camera movement with WASD keys (when camera isn't following player)
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_follow: Res<CameraFollowMode>,
    mut camera_query: Query<(&mut Transform, &GameCamera, &OrthographicProjection)>,
) {
    // Only allow manual camera movement when not following player
    if camera_follow.following {
        return;
    }

    if let Ok((mut transform, camera, projection)) = camera_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        // WASD movement controls
        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Normalize direction vector if it's not zero
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        // Adjust speed based on zoom level (faster when zoomed out)
        let adjusted_speed = camera.speed * projection.scale;

        // Adjust speed if shift is held (faster movement)
        let speed_multiplier = if keyboard.pressed(KeyCode::ShiftLeft) {
            3.0 // Faster movement with Shift
        } else {
            1.0
        };

        // Calculate the movement delta
        let movement = direction * adjusted_speed * speed_multiplier * time.delta_seconds();

        // Apply movement
        transform.translation += movement;

        // Debug log when moving
        if direction != Vec3::ZERO {
            debug!(
                "Camera moving: {:?}, Position: ({:.1}, {:.1}), Speed: {:.1}, Zoom: {:.2}",
                direction,
                transform.translation.x,
                transform.translation.y,
                if keyboard.pressed(KeyCode::ShiftLeft) {
                    "FAST"
                } else {
                    "normal"
                },
                projection.scale
            );
        }
    }
}

// System to make the camera follow the player (based on CameraFollowMode)
fn camera_follow_player(
    camera_follow: Res<CameraFollowMode>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    // Only follow player when in follow mode
    if !camera_follow.following {
        return;
    }

    if let (Ok(player_transform), Ok(mut camera_transform)) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        // Smoothly follow the player (just using the player's x position)
        camera_transform.translation.x = player_transform.translation.x;

        // Keep the camera's y position to allow for the terrain view
        // We could implement smooth following with lerp if desired
    }
}

// System to handle camera zoom with Q and E keys and mouse wheel
fn camera_zoom(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut OrthographicProjection, &GameCamera)>,
) {
    if let Ok((mut projection, camera)) = camera_query.get_single_mut() {
        let mut zoom_delta = 0.0;

        // Q to zoom out, E to zoom in - make these more responsive
        if keyboard.pressed(KeyCode::KeyQ) {
            zoom_delta += 2.0 * camera.zoom_speed * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::KeyE) {
            zoom_delta -= 2.0 * camera.zoom_speed * time.delta_seconds();
        }

        // Mouse wheel zoom - make this more responsive
        for event in mouse_wheel_events.read() {
            zoom_delta -= event.y * 0.2; // Increased sensitivity
        }

        // Apply zoom if there's any change
        if zoom_delta != 0.0 {
            // Calculate new scale with exponential zooming for smoother feel
            let zoom_factor = (-zoom_delta * 0.5).exp(); // Increased zoom speed
            let new_scale =
                (projection.scale * zoom_factor).clamp(camera.min_zoom, camera.max_zoom);

            // Update the projection scale (this is the proper way to zoom an orthographic camera)
            projection.scale = new_scale;

            // Log zoom changes
            if zoom_delta.abs() > 0.01 {
                debug!("Camera zoom: {:.2}x", projection.scale);
            }
        }
    }
}
