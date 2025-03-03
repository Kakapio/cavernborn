use crate::player::{DebugMode, Player};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// Plugin to handle camera systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_movement, camera_zoom, camera_follow_player));
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

// System to handle camera movement with WASD keys (only in debug mode)
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<DebugMode>,
    mut camera_query: Query<(&mut Transform, &GameCamera, &OrthographicProjection)>,
) {
    // Only allow manual camera movement in debug mode
    if !debug_mode.enabled {
        return;
    }

    if let Ok((mut transform, camera, projection)) = camera_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        // WASD movement
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

        // Normalize direction to prevent diagonal movement from being faster
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        // Apply movement - adjust speed based on zoom level for consistent feel
        let mut speed_adjusted = camera.speed * time.delta_seconds();

        // Adjust speed based on current zoom level (projection scale)
        speed_adjusted *= projection.scale;

        // Double speed if left shift is held
        if keyboard.pressed(KeyCode::ShiftLeft) {
            speed_adjusted *= 2.0;
        }

        transform.translation += direction * speed_adjusted;

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

// System to make the camera follow the player (only in normal mode)
fn camera_follow_player(
    debug_mode: Res<DebugMode>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    // Only follow player when not in debug mode
    if debug_mode.enabled {
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
