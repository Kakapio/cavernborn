//! Coordinate conversion functions for the chunk system

use crate::chunk::CHUNK_SIZE;
use crate::particle::PARTICLE_SIZE;
use bevy::math::{UVec2, Vec2};

/// Convert screen-space coordinates to world-space coordinates (in particle units)
pub fn screen_to_world(screen_pos: Vec2, map_width: u32, map_height: u32) -> Vec2 {
    Vec2::new(
        (screen_pos.x + ((map_width * PARTICLE_SIZE) / 2) as f32) / PARTICLE_SIZE as f32,
        (screen_pos.y + ((map_height * PARTICLE_SIZE) / 2) as f32) / PARTICLE_SIZE as f32,
    )
}

/// Convert world-space coordinates (in particle units) to chunk coordinates
pub fn world_to_chunk(world_pos: UVec2) -> UVec2 {
    UVec2::new(world_pos.x / CHUNK_SIZE, world_pos.y / CHUNK_SIZE)
}

/// Convert floating-point world coordinates to chunk coordinates
pub fn world_vec2_to_chunk(world_pos: Vec2) -> UVec2 {
    // Convert Vec2 to UVec2 by flooring the values to integers
    let world_uvec = UVec2::new(world_pos.x as u32, world_pos.y as u32);
    world_to_chunk(world_uvec)
}

/// Convert world coordinates to local chunk coordinates
pub fn world_to_local(world_pos: UVec2) -> UVec2 {
    UVec2::new(world_pos.x % CHUNK_SIZE, world_pos.y % CHUNK_SIZE)
}

/// Convert chunk coordinates to world-space pixel coordinates
pub fn chunk_to_pixels(chunk_pos: UVec2) -> Vec2 {
    Vec2::new(
        (chunk_pos.x * CHUNK_SIZE * PARTICLE_SIZE) as f32,
        (chunk_pos.y * CHUNK_SIZE * PARTICLE_SIZE) as f32,
    )
}

/// Center coordinates in screen space based on map dimensions
pub fn center_in_screen(pos: Vec2, map_width: u32, map_height: u32) -> Vec2 {
    Vec2::new(
        pos.x - ((map_width * PARTICLE_SIZE) / 2) as f32,
        pos.y - ((map_height * PARTICLE_SIZE) / 2) as f32,
    )
}

/// Convert cursor world position from Bevy's camera system to map coordinates (in particle units)
pub fn cursor_to_map_coords(cursor_world_pos: Vec2, map_width: u32, map_height: u32) -> UVec2 {
    // Convert to our world coordinate system
    let world_pos = screen_to_world(cursor_world_pos, map_width, map_height);

    // Convert to UVec2 for map operations, clamping to avoid underflow
    UVec2::new(world_pos.x.max(0.0) as u32, world_pos.y.max(0.0) as u32)
}
