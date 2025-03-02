use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

// Plugin to handle camera systems
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_movement, camera_zoom));
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

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(450.0, 150.0, 999.9), // Start in the middle of the world
            ..default()
        },
        GameCamera {
            speed: 300.0, // Units per second
            zoom_speed: 1.0,
            min_zoom: 0.1, // Allow zooming out quite far
            max_zoom: 5.0, // Allow zooming in quite close
        },
    ));

    info!("Camera initialized at position (450.0, 150.0)");
}

// System to handle camera movement with WASD keys
fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &GameCamera)>,
) {
    if let Ok((mut transform, camera)) = camera_query.get_single_mut() {
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

        // Double speed if left shift is held
        if keyboard.pressed(KeyCode::ShiftLeft) {
            speed_adjusted *= 2.0;
        }

        transform.translation += direction * speed_adjusted;

        // Debug log when moving
        if direction != Vec3::ZERO {
            debug!(
                "Camera moving: {:?}, Position: ({:.1}, {:.1}), Speed: {:.1}",
                direction,
                transform.translation.x,
                transform.translation.y,
                if keyboard.pressed(KeyCode::ShiftLeft) {
                    "FAST"
                } else {
                    "normal"
                }
            );
        }
    }
}

// System to handle camera zoom with Q and E keys and mouse wheel
fn camera_zoom(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &GameCamera)>,
) {
    if let Ok((mut transform, camera)) = camera_query.get_single_mut() {
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
                (transform.scale.x * zoom_factor).clamp(camera.min_zoom, camera.max_zoom);

            transform.scale = Vec3::new(new_scale, new_scale, 1.0);

            // Log zoom changes
            if zoom_delta.abs() > 0.01 {
                debug!("Camera zoom: {:.2}x", 1.0 / new_scale);
            }
        }
    }
}
