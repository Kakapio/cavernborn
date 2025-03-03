use crate::{
    chunk::CHUNK_SIZE,
    player::{DebugMode, Player},
    world::Map,
};
use bevy::{prelude::*, utils::HashSet};

// Plugin for debug visualization features
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>().add_systems(
            Update,
            (
                toggle_debug_features,
                update_debug_chunk_visuals,
                update_chunk_coordinate_labels,
                cleanup_debug_visuals,
            ),
        );
    }
}

// Centralized debug state to track various debug features
#[derive(Resource, Default)]
pub struct DebugState {
    // Whether to show chunk boundaries
    pub show_chunks: bool,
    // Whether to show chunk coordinates
    pub show_chunk_coords: bool,
    // Set of active chunk entities (for cleanup)
    pub chunk_visual_entities: HashSet<Entity>,
}

// Component to mark debug visualization entities
#[derive(Component)]
pub struct DebugVisual;

// Component for chunk visualization
#[derive(Component)]
pub struct ChunkVisual {
    pub chunk_pos: UVec2,
}

// Component for chunk coordinate labels
#[derive(Component)]
pub struct ChunkCoordLabel {
    pub chunk_pos: UVec2,
}

// Toggle debug features with keyboard shortcuts
fn toggle_debug_features(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    // Only process debug keys if debug mode is enabled
    if debug_mode.enabled {
        // F4 toggles chunk visualization
        if keyboard.just_pressed(KeyCode::F4) {
            debug_state.show_chunks = !debug_state.show_chunks;
            info!(
                "Chunk visualization: {}",
                if debug_state.show_chunks { "ON" } else { "OFF" }
            );
        }

        // F5 toggles chunk coordinate display
        if keyboard.just_pressed(KeyCode::F5) {
            debug_state.show_chunk_coords = !debug_state.show_chunk_coords;
            info!(
                "Chunk coordinates: {}",
                if debug_state.show_chunk_coords {
                    "ON"
                } else {
                    "OFF"
                }
            );
        }
    }
}

// Update chunk visualization based on debug state
fn update_debug_chunk_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
    map: Res<Map>,
    player_query: Query<&Transform, With<Player>>,
    mut debug_query: Query<(&mut ChunkVisual, &mut Sprite)>,
) {
    // If debug mode is disabled or chunk visualization is off, return early
    if !debug_mode.enabled || !debug_state.show_chunks {
        return;
    }

    // Define the range (in world units) around the player to consider chunks active
    const ACTIVE_CHUNK_RANGE: u32 = 200; // Should match UPDATE_RANGE in world.rs

    // Get player position for active chunk determination
    let player_pos_opt = if let Ok(player_transform) = player_query.get_single() {
        // Convert player position to world coordinates (same as in update_chunks_around_player)
        let player_x = (player_transform.translation.x
            + (map.width * crate::particle::PARTICLE_SIZE / 2) as f32)
            / crate::particle::PARTICLE_SIZE as f32;
        let player_y = (player_transform.translation.y
            + (map.height * crate::particle::PARTICLE_SIZE / 2) as f32)
            / crate::particle::PARTICLE_SIZE as f32;

        // Clamp to valid world coordinates
        let player_x = player_x.clamp(0.0, map.width as f32 - 1.0) as u32;
        let player_y = player_y.clamp(0.0, map.height as f32 - 1.0) as u32;

        Some(UVec2::new(player_x, player_y))
    } else {
        None
    };

    // Track which chunk positions already have visuals
    let mut existing_chunks: HashSet<UVec2> = HashSet::new();

    // Update existing chunk visuals
    for (chunk_visual, mut sprite) in debug_query.iter_mut() {
        existing_chunks.insert(chunk_visual.chunk_pos);

        // Check if this chunk is still in the map
        if let Some(chunk) = map.chunks.get(&chunk_visual.chunk_pos) {
            // Set color based on whether the chunk is active (near player)
            let is_active = if let Some(player_pos) = player_pos_opt {
                chunk.is_within_range(player_pos, ACTIVE_CHUNK_RANGE)
            } else {
                false // No player found, consider all chunks inactive
            };

            sprite.color = if is_active {
                Color::srgba(0.0, 1.0, 0.0, 0.3) // Green for active chunks
            } else {
                Color::srgba(0.3, 0.3, 1.0, 0.3) // Blue for inactive chunks
            };
        }
    }

    // Spawn visuals for chunks that don't have them yet
    for (pos, chunk) in map.chunks.iter() {
        if existing_chunks.contains(pos) {
            continue;
        }

        // Calculate chunk size in pixels
        let chunk_pixel_size = CHUNK_SIZE * crate::particle::PARTICLE_SIZE;

        // Calculate world position of chunk's top-left corner
        let chunk_world_pos = UVec2::new(pos.x * CHUNK_SIZE, pos.y * CHUNK_SIZE);

        // Convert to world space by multiplying by particle size
        let pixel_x = chunk_world_pos.x * crate::particle::PARTICLE_SIZE;
        let pixel_y = chunk_world_pos.y * crate::particle::PARTICLE_SIZE;

        // Calculate the center of the chunk in world space
        let center_x = pixel_x + (chunk_pixel_size / 2);
        let center_y = pixel_y + (chunk_pixel_size / 2);

        // Adjust for world centering (center of the world should be at (0,0))
        let half_world_width = (map.width * crate::particle::PARTICLE_SIZE) / 2;
        let half_world_height = (map.height * crate::particle::PARTICLE_SIZE) / 2;

        // Convert to i64 to safely handle subtraction without overflow
        let center_x_i64 = center_x as i64;
        let center_y_i64 = center_y as i64;
        let half_world_width_i64 = half_world_width as i64;
        let half_world_height_i64 = half_world_height as i64;

        // Calculate final position (centered in world coords)
        let final_x = center_x_i64 - half_world_width_i64;
        let final_y = center_y_i64 - half_world_height_i64;

        // Determine if this chunk is active (near the player)
        let is_active = if let Some(player_pos) = player_pos_opt {
            chunk.is_within_range(player_pos, ACTIVE_CHUNK_RANGE)
        } else {
            false // No player found, consider all chunks inactive
        };

        // Spawn a chunk outline (convert to f32 for Bevy's Transform/Vec2)
        let entity = commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if is_active {
                            Color::srgba(0.0, 1.0, 0.0, 0.3) // Green for active chunks
                        } else {
                            Color::srgba(0.3, 0.3, 1.0, 0.3) // Blue for inactive chunks
                        },
                        custom_size: Some(Vec2::new(
                            chunk_pixel_size as f32,
                            chunk_pixel_size as f32,
                        )),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        final_x as f32,
                        final_y as f32,
                        5.0, // Just above terrain, below player
                    ),
                    ..default()
                },
                ChunkVisual { chunk_pos: *pos },
                DebugVisual,
            ))
            .id();

        // Add to the set of debug visuals for cleanup
        debug_state.chunk_visual_entities.insert(entity);
    }
}

// System to update chunk coordinate labels
fn update_chunk_coordinate_labels(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    debug_state: Res<DebugState>,
    map: Res<Map>,
    label_query: Query<(Entity, &ChunkCoordLabel)>,
) {
    // Return early if debug mode or chunk coordinate display is disabled
    if !debug_mode.enabled || !debug_state.show_chunk_coords {
        // Clean up existing labels if feature was disabled
        for (entity, _) in label_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Track existing labels
    let mut existing_labels = HashSet::new();
    for (_, label) in label_query.iter() {
        existing_labels.insert(label.chunk_pos);
    }

    // Create font size based on zoom level (would need to access camera for this)
    let font_size = 14.0;

    // Add labels for chunks that don't have them
    for (pos, _) in map.chunks.iter() {
        if existing_labels.contains(pos) {
            continue;
        }

        // Calculate chunk size in pixels
        let chunk_pixel_size = CHUNK_SIZE * crate::particle::PARTICLE_SIZE;

        // Calculate world position of chunk's top-left corner
        let chunk_world_pos = UVec2::new(pos.x * CHUNK_SIZE, pos.y * CHUNK_SIZE);

        // Convert to world space by multiplying by particle size
        let pixel_x = chunk_world_pos.x * crate::particle::PARTICLE_SIZE;
        let pixel_y = chunk_world_pos.y * crate::particle::PARTICLE_SIZE;

        // Calculate the center of the chunk in world space
        let center_x = pixel_x + (chunk_pixel_size / 2);
        let center_y = pixel_y + (chunk_pixel_size / 2);

        // Adjust for world centering (center of the world should be at (0,0))
        let half_world_width = (map.width * crate::particle::PARTICLE_SIZE) / 2;
        let half_world_height = (map.height * crate::particle::PARTICLE_SIZE) / 2;

        // Convert to i64 to safely handle subtraction without overflow
        let center_x_i64 = center_x as i64;
        let center_y_i64 = center_y as i64;
        let half_world_width_i64 = half_world_width as i64;
        let half_world_height_i64 = half_world_height as i64;

        // Calculate final position (centered in world coords)
        let final_x = center_x_i64 - half_world_width_i64;
        let final_y = center_y_i64 - half_world_height_i64;

        // Spawn text label at the center of the chunk (convert to f32 for Bevy's Transform)
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("{},{}", pos.x, pos.y),
                    TextStyle {
                        font_size,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(
                    final_x as f32,
                    final_y as f32,
                    6.0, // Above chunk outline
                ),
                // Make text smaller based on font size
                text_anchor: bevy::sprite::Anchor::Center,
                ..default()
            },
            ChunkCoordLabel { chunk_pos: *pos },
            DebugVisual,
        ));
    }
}

// Cleanup debug visuals when debug mode is turned off
fn cleanup_debug_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    debug_state: Res<DebugState>,
    query: Query<Entity, With<DebugVisual>>,
) {
    // If debug mode is off or chunk visualization is off, clean up visuals
    if !debug_mode.enabled || !debug_state.show_chunks {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
