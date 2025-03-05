#![allow(unused)]
#![allow(dead_code)]

use crate::chunk::{Chunk, CHUNK_SIZE};
use crate::map::Map;
use crate::particle::Particle;
use bevy::prelude::*;

use crate::render::chunk_material::ChunkMaterial;

use super::chunk_material::ChunkMaterialPlugin;

/// Plugin that handles rendering the map
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkMaterialPlugin)
            .add_systems(Startup, setup_map_renderer)
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
pub struct ChunkRenderer;

/// System that sets up the map renderer
fn setup_map_renderer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Calculate the mesh size in pixels
    let chunk_size_pixels = (CHUNK_SIZE * crate::particle::PARTICLE_SIZE) as f32;

    // Create the map renderer entity
    commands.spawn((
        MapRenderer {
            sprite_atlas: asset_server.load("textures\\particle_atlas.png"),
            chunk_mesh: meshes.add(Rectangle::new(chunk_size_pixels, chunk_size_pixels)),
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
    mut materials: ResMut<Assets<ChunkMaterial>>,
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
    let sprite_atlas = map_renderer.sprite_atlas.clone();

    // Spawn new renderers for the active chunks.
    for chunk in active_chunks {
        // Check if the chunk exists
        if let Some(chunk_data) = map.get_chunk_at(chunk) {
            // Calculate world position for this chunk in pixels
            let chunk_pixels = crate::utils::coords::chunk_to_pixels(*chunk);
            let chunk_size_pixels = (CHUNK_SIZE * crate::particle::PARTICLE_SIZE) as f32;

            // Adjust for world centering
            let centered_pos =
                crate::utils::coords::center_in_screen(chunk_pixels, map.width, map.height);

            // Create our new renderer entity...
            let chunk_renderer = commands
                .spawn((
                    ChunkRenderer,
                    // Copy the handle to the central mesh we created in setup_map_renderer.
                    Mesh2d(map_renderer.chunk_mesh.clone()),
                    MeshMaterial2d(materials.add(ChunkMaterial::from_indices(
                        sprite_atlas.clone(),
                        chunk_data.to_spritesheet_indices(),
                    ))),
                    // Position the renderer at the correct location.
                    Transform::from_xyz(
                        centered_pos.x + chunk_size_pixels / 2.0,
                        centered_pos.y + chunk_size_pixels / 2.0,
                        1.0,
                    ),
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
