use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::particle::Fluid::Water;
use crate::particle::Particle::Fluid;
use crate::utils::coords::bresenham_line;

// Constants for player
const PLAYER_SIZE: u32 = 20;
const PLAYER_SPEED: f32 = 150.0;

// Player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMode>()
            .init_resource::<CameraConnection>()
            .init_resource::<LastMousePosition>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, spawn_player)
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, player_movement)
            .add_systems(Update, toggle_debug_mode)
            .add_systems(Update, toggle_camera_connection)
            .add_systems(Update, update_fps_counter)
            .add_systems(Update, handle_mouse_interactions);
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

#[derive(Resource)]
pub struct CameraConnection {
    pub connected_to_player: bool,
}

impl Default for CameraConnection {
    fn default() -> Self {
        Self {
            connected_to_player: true, // Initialize to true
        }
    }
}

// Resource to track the last mouse position
#[derive(Resource, Default)]
struct LastMousePosition(Option<UVec2>);

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
    camera_connection: Res<CameraConnection>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if !camera_connection.connected_to_player {
        return;
    }

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

// New system to toggle camera connection with spacebar
fn toggle_camera_connection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_connection: ResMut<CameraConnection>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        camera_connection.connected_to_player = !camera_connection.connected_to_player;
        info!(
            "Camera {} player",
            if camera_connection.connected_to_player {
                "connected to"
            } else {
                "disconnected from"
            }
        );
    }
}

// Unified system to handle all mouse interactions
fn handle_mouse_interactions(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut map: ResMut<crate::map::Map>,
    mut last_pos: ResMut<LastMousePosition>,
) {
    // Handle case when left mouse button is released - reset last position
    if mouse_input.just_released(MouseButton::Left) {
        last_pos.0 = None;
        return;
    }

    // Check which mouse button is being pressed
    let left_pressed = mouse_input.pressed(MouseButton::Left);
    let right_pressed = mouse_input.pressed(MouseButton::Right);

    if !left_pressed && !right_pressed {
        return; // Exit early if no relevant mouse button is pressed
    }

    // Get the primary window
    let window = windows.single();

    // Get cursor position in window if available
    if let Some(cursor_position) = window.cursor_position() {
        // Get camera for screen to world conversion
        let (camera, camera_transform) = camera_q.single();

        // Convert screen position to world coordinates using the 2D-specific method
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            // Convert to our map's coordinates
            let current_pos =
                crate::utils::coords::cursor_to_map_coords(world_position, map.width, map.height);

            // Handle left click (remove particles)
            if left_pressed {
                if let Some(last_mouse_pos) = last_pos.0 {
                    // Draw a line using Bresenham's line algorithm to get all points between last and current
                    let line_points = bresenham_line(last_mouse_pos, current_pos);

                    // Remove particles at all points along the line
                    for point in line_points {
                        remove_particles_at(point, &mut map);
                    }
                } else {
                    // First click, just remove at current position
                    remove_particles_at(current_pos, &mut map);
                }

                // Update last position to current
                last_pos.0 = Some(current_pos);
            }

            // Handle right click (place water)
            if right_pressed {
                place_water_at(current_pos, &mut map);
            }
        }
    }
}

// Helper function to remove particles in a 2x2 area at the given position
fn remove_particles_at(center_pos: UVec2, map: &mut crate::map::Map) {
    // Remove particles in a 2x2 area
    for x_offset in 0..2 {
        for y_offset in 0..2 {
            let pos = UVec2::new(center_pos.x + x_offset, center_pos.y + y_offset);

            // Skip if outside map bounds
            if pos.x >= map.width || pos.y >= map.height {
                continue;
            }

            // Set particle to Air (None)
            map.set_particle_at(pos, None);
        }
    }
}

// Helper function to place water particles in a 3x3 area at the given position
fn place_water_at(center_pos: UVec2, map: &mut crate::map::Map) {
    // Place water in a 3x3 area
    for x_offset in 0..3 {
        for y_offset in 0..3 {
            // Calculate position with the center point in the middle of the 3x3 area
            let x = center_pos.x as i32 + x_offset - 1;
            let y = center_pos.y as i32 + y_offset - 1;

            // Skip if outside map bounds (checking with i32 to avoid underflow)
            if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
                continue;
            }

            let pos = UVec2::new(x as u32, y as u32);

            // Set particle to Water
            map.set_particle_at(pos, Some(Fluid(Water)));
        }
    }
}
