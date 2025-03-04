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
pub fn world_to_chunk(world_pos: Vec2) -> UVec2 {
    UVec2::new(
        (world_pos.x / CHUNK_SIZE as f32).floor() as u32,
        (world_pos.y / CHUNK_SIZE as f32).floor() as u32,
    )
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

/// Convert screen position directly to chunk coordinates
pub fn screen_to_chunk(screen_pos: Vec2, map_width: u32, map_height: u32) -> UVec2 {
    let world_pos = screen_to_world(screen_pos, map_width, map_height);
    world_to_chunk(world_pos)
}
