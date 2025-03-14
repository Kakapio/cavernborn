use bevy::math::UVec2;

use crate::{
    particle::{Particle, ParticleType},
    utils::coords::world_to_local,
    world::{
        chunk::{Chunk, ParticleMove, CHUNK_SIZE},
        Map,
    },
};

pub mod fluid;

/// A trait for types that can simulate particles.
pub trait Simulator<P: ParticleType> {
    fn simulate(
        &mut self,
        map: &Map,
        original_chunk: &Chunk,
        new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        particle: P,
        x: u32,
        y: u32,
    ) -> Vec<ParticleMove>;
}

/// Checks if a particle can move to a new position.
///
/// This function first verifies that the new position is valid within the map's boundaries.
/// If the new position is within the same chunk, it also ensures that the spot is empty
/// in the chunk's updated state. If the position is outside the original chunk, movement
/// is considered valid and will be handled by the queue system.
fn validate_move(
    map: &Map,
    original_chunk: &Chunk,
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    new_pos: UVec2,
) -> bool {
    // Was it valid on the older not-yet-updated map?
    let valid_old_map = map.is_valid_position(new_pos);
    let valid_new_chunk = if original_chunk.is_within_chunk(new_pos) {
        // We're within the same new chunk... Let's make sure it's empty in the new chunk too.
        let local_pos = world_to_local(new_pos);
        new_cells[local_pos.x as usize][local_pos.y as usize].is_none()
    } else {
        // Not within the same chunk, so no need for additional validation.
        true
    };

    valid_old_map && valid_new_chunk
}
