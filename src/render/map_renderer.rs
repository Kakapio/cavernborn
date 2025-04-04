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

/// Component that marks an entity as the map renderer and holds the sprite atlas.
#[derive(Component)]
pub struct MapRenderer {
    pub chunk_renderers: Vec<Entity>,
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
    let sprite_atlas = asset_server.load("textures\\particle_atlas.png");
    let chunk_mesh = meshes.add(Rectangle::new(chunk_size_pixels, chunk_size_pixels));

    // Insert resources
    commands.insert_resource(MapRenderResources {
        sprite_atlas: sprite_atlas.clone(),
        chunk_mesh: chunk_mesh.clone(),
    });

    // Create the map renderer entity
    commands.spawn((
        MapRenderer {
            chunk_renderers: Vec::new(),
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

    // Get chunk positions within range
    let chunk_positions = map.get_chunks_near(player_pos, RENDER_RANGE);

    // Convert to (position, chunk) pairs
    let mut result = Vec::new();
    for pos in chunk_positions {
        result.push((pos, map.get_chunk_at(&pos)));
    }

    result
}

/// System that renders chunks near the player based on RENDER_DISTANCE
fn render_map(
    mut commands: Commands,
    map: Res<Map>,
    player_query: Query<&Transform, With<Player>>,
    // Query for just the MapRenderer component with mutable access
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

    // Clean up old chunk renderers
    for entity in map_renderer.chunk_renderers.drain(..) {
        commands.entity(entity).despawn_recursive();
    }

    // Spawn new renderers for the chunks to render.
    for (chunk_pos, chunk) in chunks_to_render {
        // Calculate world position for this chunk in pixels
        let chunk_pixels = coords::chunk_pos_to_screen(chunk_pos);
        let chunk_size_pixels = (CHUNK_SIZE * crate::particle::PARTICLE_SIZE) as f32;

        // Adjust for world centering
        let centered_pos = coords::center_in_screen(chunk_pixels, map.width, map.height);

        // Calculate the position for the chunk
        let chunk_pos_x = centered_pos.x + chunk_size_pixels / 2.0;
        let chunk_pos_y = centered_pos.y + chunk_size_pixels / 2.0;

        // Create our new renderer entity...
        let chunk_renderer = commands
            .spawn((
                ChunkRenderer,
                Name::new(format!("ChunkRenderer({})", chunk_pos)),
                // Copy the handle to the central mesh/sprite atlas we created in setup_map_renderer.
                Mesh2d(render_resources.chunk_mesh.clone()),
                MeshMaterial2d(materials.add(ChunkMaterial::from_indices(
                    render_resources.sprite_atlas.clone(),
                    chunk.to_spritesheet_indices(),
                ))),
                // Position the chunk renderer relative to parent
                Transform::from_xyz(chunk_pos_x, chunk_pos_y, 1.0),
                // Add Visibility components for frustum culling
                Visibility::Inherited,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();

        // Add the chunk renderer as a child of the map renderer
        commands
            .entity(map_renderer_entity)
            .add_child(chunk_renderer);

        // Add it to our list of renderers
        map_renderer.chunk_renderers.push(chunk_renderer);
    }
}
