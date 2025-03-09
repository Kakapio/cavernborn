use crate::player::{CameraConnection, Player};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// Plugin to handle camera systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_movement, camera_zoom))
            .add_systems(
                PostUpdate,
                camera_follow_player.before(TransformSystem::TransformPropagate),
            );
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

    // Using Camera2d component with required components
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9), // Start at origin where player will spawn
        OrthographicProjection {
            scale: default_zoom,
            ..OrthographicProjection::default_2d()
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

// System to handle camera movement with WASD keys (when camera isn't following player)
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_connection: Res<CameraConnection>,
    mut camera_query: Query<(&mut Transform, &GameCamera, &OrthographicProjection)>,
) {
    // Only allow manual camera movement when not following player
    if camera_connection.connected_to_player {
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
        let movement = direction * adjusted_speed * speed_multiplier * time.delta_secs();

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

// System to make the camera follow the player (based on CameraConnection)
fn camera_follow_player(
    camera_connection: Res<CameraConnection>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    // Only follow player when in follow mode
    if !camera_connection.connected_to_player {
        return;
    }

    if let (Ok(player_transform), Ok(mut camera_transform)) =
        (player_query.get_single(), camera_query.get_single_mut())
    {
        // Smoothly follow the player (just using the player's x position)
        camera_transform.translation.x = player_transform.translation.x;

        // We could implement smooth following with lerp if desired
        camera_transform.translation.y = player_transform.translation.y;
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
            zoom_delta += 2.0 * camera.zoom_speed * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyE) {
            zoom_delta -= 2.0 * camera.zoom_speed * time.delta_secs();
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
