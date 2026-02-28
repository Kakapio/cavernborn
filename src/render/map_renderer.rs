use std::collections::HashMap;

use crate::player::Player;
use crate::utils::{self, coords};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::map::Map;
use bevy::prelude::*;

use crate::render::chunk_material::ChunkMaterial;

use super::chunk_material::ChunkMaterialPlugin;

/// The range (in chunks) at which chunks are rendered around the player.
/// It is used to spawn the chunk renderers, so it is not quite culling.
/// The actual frustum culling is done in the `render_map` system.
const RENDER_DISTANCE: u32 = 16;

/// Plugin that handles rendering the map
pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkMaterialPlugin)
            .add_systems(Startup, setup_map_renderer)
            .add_systems(Update, render_map);
    }
}

/// Component that marks an entity as the map renderer and tracks chunk renderer entities.
#[derive(Component)]
pub struct MapRenderer {
    /// Maps chunk positions to (entity, material handle, last-rendered version).
    pub chunk_renderers: HashMap<UVec2, (Entity, Handle<ChunkMaterial>, u64)>,
}

/// Component that marks an individual chunk's renderer and stores handles to resources.
#[derive(Component)]
pub struct ChunkRenderer;

/// Resource to store shared rendering resources
#[derive(Resource)]
pub struct MapRenderResources {
    sprite_atlas: Handle<Image>,
    chunk_mesh: Handle<Mesh>,
}

/// System that sets up the map renderer
fn setup_map_renderer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Calculate the mesh size in pixels. 32x32 chunks and a particle size of 3 mean 96x96 pixels.
    let chunk_size_pixels = (CHUNK_SIZE * crate::particle::PARTICLE_SIZE) as f32;

    // Create shared resources
    let sprite_atlas = asset_server.load("textures/particle_atlas.png");
    let chunk_mesh = meshes.add(Rectangle::new(chunk_size_pixels, chunk_size_pixels));

    // Insert resources
    commands.insert_resource(MapRenderResources {
        sprite_atlas: sprite_atlas.clone(),
        chunk_mesh: chunk_mesh.clone(),
    });

    // Create the map renderer entity
    commands.spawn((
        MapRenderer {
            chunk_renderers: HashMap::new(),
        },
        Name::new("MapRenderer"),
        Transform::default(),
        InheritedVisibility::default(),
    ));
}

/// Get chunks to render based on player position and `RENDER_DISTANCE`.
fn get_chunks_to_render<'a>(map: &'a Map, player_transform: &Transform) -> Vec<(UVec2, &'a Chunk)> {
    // Convert RENDER_DISTANCE from chunks to world units
    const RENDER_RANGE: u32 = RENDER_DISTANCE * CHUNK_SIZE;

    // Convert player position to world coordinates
    let player_pos = utils::coords::screen_to_world(
        player_transform.translation.truncate(),
        map.width,
        map.height,
    );

    // Get chunk positions within range and pair them with chunk references
    map.get_chunks_near(player_pos, RENDER_RANGE)
        .into_iter()
        .map(|pos| (pos, map.get_chunk_at(&pos)))
        .collect()
}

/// System that renders chunks near the player based on RENDER_DISTANCE.
/// Uses cached chunk renderers to avoid despawning/respawning entities every frame.
fn render_map(
    mut commands: Commands,
    map: Res<Map>,
    player_query: Query<&Transform, With<Player>>,
    mut map_renderer_query: Query<(Entity, &mut MapRenderer)>,
    render_resources: Res<MapRenderResources>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    // Get player transform and chunks to render first
    let player_transform = match player_query.get_single() {
        Ok(transform) => transform,
        Err(_) => return, // Early return if player not found
    };

    let chunks_to_render = get_chunks_to_render(&map, player_transform);

    // Now access the renderer after gathering all required data
    let (map_renderer_entity, mut map_renderer) = match map_renderer_query.get_single_mut() {
        Ok((entity, renderer)) => (entity, renderer),
        Err(e) => {
            panic!("Failed to get MapRenderer component: {:?}", e);
        }
    };

    // Build a set of chunk positions that should be visible this frame
    let visible_positions: std::collections::HashSet<UVec2> =
        chunks_to_render.iter().map(|(pos, _)| *pos).collect();

    // Remove renderers for chunks that are no longer visible
    map_renderer
        .chunk_renderers
        .retain(|pos, (entity, _handle, _version)| {
            if visible_positions.contains(pos) {
                true
            } else {
                commands.entity(*entity).despawn_recursive();
                false
            }
        });

    // Update existing renderers or spawn new ones
    for (chunk_pos, chunk) in chunks_to_render {
        if let Some((_entity, handle, last_version)) =
            map_renderer.chunk_renderers.get_mut(&chunk_pos)
        {
            // Only update material if the chunk has changed since last render
            if chunk.version != *last_version {
                if let Some(material) = materials.get_mut(handle.id()) {
                    material.indices = chunk.to_spritesheet_indices();
                }
                *last_version = chunk.version;
            }
        } else {
            // Spawn a new renderer entity for this chunk
            let (_chunk_size, center_pos) =
                coords::chunk_screen_rect(chunk_pos, map.width, map.height);

            let material_handle = materials.add(ChunkMaterial::from_indices(
                render_resources.sprite_atlas.clone(),
                chunk.to_spritesheet_indices(),
            ));

            let chunk_renderer = commands
                .spawn((
                    ChunkRenderer,
                    Mesh2d(render_resources.chunk_mesh.clone()),
                    MeshMaterial2d(material_handle.clone()),
                    Transform::from_xyz(center_pos.x, center_pos.y, 1.0),
                    Visibility::Inherited,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            // Add the chunk renderer as a child of the map renderer
            commands
                .entity(map_renderer_entity)
                .add_child(chunk_renderer);

            map_renderer
                .chunk_renderers
                .insert(chunk_pos, (chunk_renderer, material_handle, chunk.version));
        }
    }
}
