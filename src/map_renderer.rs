#![allow(unused)]
#![allow(dead_code)]

use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::map::Map;
use crate::particle::Particle;
use bevy::prelude::*;

/// Plugin that handles rendering the map
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map_renderer)
            .add_systems(Update, render_map);
    }
}
// MapRenderer entity is the parent
// ChunkRenderer entities are children
/// Component that marks an entity as the map renderer and holds the sprite atlas.
#[derive(Component)]
pub struct MapRenderer {
    sprite_atlas: Handle<Image>,
    chunk_mesh: Handle<Mesh>,
    pub chunk_renderers: Vec<Entity>,
}

/// Component that marks an individual chunk's renderer and stores a handle to its mesh.
#[derive(Component)]
pub struct ChunkRenderer {
    /// The spritesheet indices for the particles in this chunk.
    /// The total number of indices should be CHUNK_SIZE * CHUNK_SIZE.
    pub spritesheet_indices: Vec<u32>,
}

impl ChunkRenderer {
    pub fn new(spritesheet_indices: Vec<u32>) -> Self {
        if spritesheet_indices.len() != (CHUNK_SIZE * CHUNK_SIZE) as usize {
            panic!(
                "ChunkRenderer spritesheet indices must be of length CHUNK_SIZE * CHUNK_SIZE. Got {}.",
                spritesheet_indices.len()
            );
        }

        Self {
            spritesheet_indices,
        }
    }
}

/// System that sets up the map renderer
fn setup_map_renderer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create the map renderer entity
    commands.spawn((
        MapRenderer {
            sprite_atlas: asset_server.load("textures\\particle_atlas.png"),
            chunk_mesh: meshes.add(Rectangle::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32)),
            chunk_renderers: Vec::new(),
        },
        Transform::default(),
    ));
}

/// System that renders the active chunks in the map
fn render_map(
    mut commands: Commands,
    map: Res<Map>,
    // Query for just the MapRenderer component with mutable access
    mut map_renderer_query: Query<&mut MapRenderer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let active_chunks = &map.active_chunks;

    // Return early if there are no active chunks
    if active_chunks.is_empty() {
        warn!("No active chunks to render, skipping.");
        return;
    }

    // Access renderer.
    let mut map_renderer = match map_renderer_query.get_single_mut() {
        Ok(renderer) => renderer,
        Err(e) => {
            panic!("Failed to get MapRenderer component: {:?}", e);
        }
    };

    // Despawn all the old chunk renderer entities
    for entity in &map_renderer.chunk_renderers {
        commands.entity(*entity).despawn_recursive();
    }

    // Then clear the Vec of entity reference IDs.
    map_renderer.chunk_renderers.clear();

    // Get a reference to the sprite atlas.
    let sprite_atlas = &map_renderer.sprite_atlas;

    // Spawn new renderers for the active chunks.
    for chunk in active_chunks {
        // Check if the chunk exists
        if let Some(chunk_data) = map.get_chunk_at(chunk) {
            // Create our new renderer entity...
            let chunk_renderer = commands
                .spawn((
                    ChunkRenderer::new(chunk_data.to_spritesheet_indices()),
                    // Copy the handle to the central mesh we created in setup_map_renderer.
                    Mesh2d(map_renderer.chunk_mesh.clone()),
                    // Position the renderer at the correct location.
                    Transform::from_translation(Vec3::new(
                        chunk.x as f32 * CHUNK_SIZE as f32,
                        chunk.y as f32 * CHUNK_SIZE as f32,
                        0.0,
                    )),
                ))
                .id();

            // And add it to our list of renderers.
            map_renderer.chunk_renderers.push(chunk_renderer);
        } else {
            warn!(
                "Attempted to render non-existent chunk at position: {:?}",
                chunk
            );
        }
    }
}
