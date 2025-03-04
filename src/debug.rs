use crate::{
    chunk::{self, coords},
    map::Map,
    particle::PARTICLE_SIZE,
    player::{DebugMode, Player},
};
use bevy::{prelude::*, utils::HashSet};
use std::collections::HashMap;

// Plugin for debug visualization features
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>().add_systems(
            Update,
            (
                toggle_debug_features,
                update_debug_chunk_visuals,
                cleanup_debug_visuals,
            ),
        );
    }
}

// Centralized debug state to track various debug features
#[derive(Resource, Default)]
pub struct DebugState {
    // Whether to show chunk visualization
    pub show_chunks: bool,
    // Track chunk visualization entities for cleanup
    pub chunk_entities: HashSet<Entity>,
    // Previous visibility state to detect changes
    pub chunks_visible_last_frame: bool,
}

// Component for chunk visualization
#[derive(Component)]
pub struct ChunkVisual {
    pub chunk_pos: UVec2,
    pub is_active: bool,
}

// Toggle debug features with keyboard shortcuts
fn toggle_debug_features(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    // Only process debug keys if debug mode is enabled
    if debug_mode.enabled {
        // F4 toggles chunk visualization (both outlines and labels)
        if keyboard.just_pressed(KeyCode::F4) {
            debug_state.show_chunks = !debug_state.show_chunks;
            info!(
                "Chunk visualization: {}",
                if debug_state.show_chunks { "ON" } else { "OFF" }
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
    mut chunk_visual_query: Query<(Entity, &mut ChunkVisual, &mut Sprite)>,
) {
    // Determine if chunk visualization should be visible
    let chunks_enabled = debug_mode.enabled && debug_state.show_chunks;

    // If visualization state changed from visible to hidden, clear all visualizations
    if debug_state.chunks_visible_last_frame && !chunks_enabled {
        for entity in debug_state.chunk_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }
        debug_state.chunks_visible_last_frame = false;
        return;
    }

    // Skip if debug mode or chunk visualization is disabled
    if !chunks_enabled {
        return;
    }

    // Mark the visualization as visible for this frame
    debug_state.chunks_visible_last_frame = true;

    // Get player position in chunk coordinates
    let player_chunk_pos = if let Ok(transform) = player_query.get_single() {
        coords::screen_to_chunk(transform.translation.truncate(), map.width, map.height)
    } else {
        return;
    };

    // Get all chunk positions from the map
    let active_chunks = map.chunks.clone();

    // Track existing and new chunk visuals
    let mut existing_chunks = HashMap::new();
    for (entity, chunk_visual, _) in chunk_visual_query.iter() {
        existing_chunks.insert(chunk_visual.chunk_pos, entity);
    }

    // Process each chunk from the map
    for (chunk_pos, _) in active_chunks.iter() {
        let chunk_entity = if let Some(&entity) = existing_chunks.get(chunk_pos) {
            // Update existing chunk visual
            if let Ok(entry) = chunk_visual_query.get_mut(entity) {
                let (_, mut visual_comp, mut sprite) = entry;

                // Check if the chunk is active based on distance in chunk coordinates
                let is_active = chunk::is_within_range(*chunk_pos, player_chunk_pos);
                visual_comp.is_active = is_active;

                // Update color based on active state
                if is_active {
                    sprite.color = Color::srgba(0.0, 1.0, 0.0, 0.3); // Green for active
                } else {
                    sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.3); // Red for inactive
                }
            }

            entity
        } else {
            // Calculate world position for this chunk in pixels
            let chunk_pixels = coords::chunk_to_pixels(*chunk_pos);
            let chunk_size_pixels = (chunk::CHUNK_SIZE * PARTICLE_SIZE) as f32;

            // Adjust for world centering
            let centered_pos = coords::center_in_screen(chunk_pixels, map.width, map.height);

            // Check if chunk is active
            let is_active = chunk::is_within_range(*chunk_pos, player_chunk_pos);

            // Create chunk outline
            let chunk_entity = commands
                .spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(chunk_size_pixels, chunk_size_pixels)),
                        color: if is_active {
                            Color::srgba(0.0, 1.0, 0.0, 0.3) // Green for active
                        } else {
                            Color::srgba(1.0, 0.0, 0.0, 0.3) // Red for inactive
                        },
                        ..default()
                    },
                    Transform::from_xyz(
                        centered_pos.x + chunk_size_pixels / 2.0,
                        centered_pos.y + chunk_size_pixels / 2.0,
                        10.0,
                    ),
                    ChunkVisual {
                        chunk_pos: *chunk_pos,
                        is_active,
                    },
                ))
                .with_children(|parent| {
                    // Add text label as a child entity
                    parent.spawn((
                        Text::from(format!("{},{}", chunk_pos.x, chunk_pos.y)),
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                })
                .id();

            // Track the new entity
            debug_state.chunk_entities.insert(chunk_entity);

            chunk_entity
        };

        // Ensure it's in our tracking set
        debug_state.chunk_entities.insert(chunk_entity);

        // Remove from the map of existing chunks to find chunks that no longer exist
        existing_chunks.remove(chunk_pos);
    }

    // Remove chunk visuals for chunks no longer in the map
    for (_, entity) in existing_chunks {
        commands.entity(entity).despawn_recursive();
        debug_state.chunk_entities.remove(&entity);
    }
}

// Clean up debug visuals when needed
fn cleanup_debug_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    // Only run this cleanup when debug mode is turned off
    if !debug_mode.enabled && !debug_state.chunk_entities.is_empty() {
        // Clean up all chunk visualization entities
        for entity in debug_state.chunk_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }

        debug_state.chunks_visible_last_frame = false;
    }
}
