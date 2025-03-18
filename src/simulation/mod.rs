use bevy::math::UVec2;
use dashmap::DashMap;

use crate::{
    particle::{
        interaction::{InteractionPair, INTERACTION_RULES},
        Particle, ParticleType,
    },
    utils::coords::world_to_chunk_local,
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
        context: SimulationContext,
        particle: P,
        x: u32,
        y: u32,
    ) -> Option<ParticleMove>;
}

/// A context for particle simulation.
/// Contains references to the map, original chunk, chunk queue, and new cells.
pub struct SimulationContext<'a> {
    pub map: &'a Map,
    pub original_chunk: &'a Chunk,
    pub chunk_queue: &'a DashMap<UVec2, ParticleMove>,
    pub new_cells: &'a mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
}

impl<'a> SimulationContext<'a> {
    pub fn new(
        map: &'a Map,
        original_chunk: &'a Chunk,
        chunk_queue: &'a DashMap<UVec2, ParticleMove>,
        new_cells: &'a mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    ) -> Self {
        Self {
            map,
            original_chunk,
            chunk_queue,
            new_cells,
        }
    }
}

/// Checks if a particle can move to a new position.
///
/// This function first verifies that the new position is valid within the map's boundaries.
/// If the new position is within the same chunk, it also ensures that the spot is empty
/// in the chunk's updated state. If the position is outside the original chunk, movement
/// is checked against what is currently in the queue.
fn validate_move_empty(context: &SimulationContext, new_pos: UVec2) -> bool {
    // Was it valid on the older not-yet-updated map?
    let valid_old_map = context.map.is_valid_position(new_pos);
    let valid_new_chunk = if context.original_chunk.is_within_chunk(new_pos) {
        // We're within the same new chunk... Let's make sure it's empty in the new chunk too.
        let local_pos = world_to_chunk_local(new_pos);
        context.new_cells[local_pos.x as usize][local_pos.y as usize].is_none()
    } else {
        // Not within the same chunk, so have we already queued a move to this location?
        !context.chunk_queue.contains_key(&new_pos)
    };

    valid_old_map && valid_new_chunk
}

/// Checks if a particle can move to a new position which yields an interaction.
/// Returns false if the target position is empty.
fn validate_move_interaction(
    context: &SimulationContext,
    new_pos: UVec2,
    particle: Particle,
) -> bool {
    // Get the target particle from the map first
    let Some(target_particle) = context.map.get_particle_at(new_pos) else {
        return false;
    };

    // Check if this is a valid interaction in the interaction rules
    let interaction_pair = InteractionPair {
        source: particle,
        target: target_particle,
    };

    // First validate against the map
    let valid_old_map =
        context.map.within_bounds(new_pos) && INTERACTION_RULES.contains_key(&interaction_pair);

    let valid_new_chunk = if context.original_chunk.is_within_chunk(new_pos) {
        // We're within the same new chunk... Let's make sure it's valid in the new chunk too.
        let local_pos = world_to_chunk_local(new_pos);
        if let Some(new_target) = context.new_cells[local_pos.x as usize][local_pos.y as usize] {
            INTERACTION_RULES.contains_key(&InteractionPair {
                source: particle,
                target: new_target,
            })
        } else {
            false
        }
    } else {
        // Not within the same chunk, so have we already queued a move to this location?
        !context.chunk_queue.contains_key(&new_pos)
    };

    valid_old_map && valid_new_chunk
}

/// Handles the result of a particle movement calculation, either updating the local chunk
/// or queueing for inter-chunk movement.
///
/// This utility function encapsulates the common logic used by particle simulators to process
/// the result of movement calculations. It either:
/// 1. Adds the particle to the inter-chunk queue if the new position is outside the current chunk
/// 2. Updates the new cells matrix directly if the position is within the current chunk
///
/// # Arguments
/// * `original_chunk` - The chunk being processed
/// * `new_cells` - Matrix of new cell states for the chunk
/// * `source_pos` - Original world position of the particle
/// * `new_pos` - New world position the particle should move to
/// * `particle` - The particle to move
///
/// # Returns
/// A vector of `ParticleMove` entries for inter-chunk movement, or an empty vector if
/// the particle was placed within the current chunk.
pub fn handle_particle_movement(
    original_chunk: &Chunk,
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    source_pos: UVec2,
    new_pos: UVec2,
    particle: Particle,
) -> Option<ParticleMove> {
    let mut interchunk_move = None;

    // If the new position is not within the chunk, queue it for inter-chunk movement.
    if !original_chunk.is_within_chunk(new_pos) {
        interchunk_move = Some(ParticleMove {
            source_pos,
            target_pos: new_pos,
            particle,
        });
    } else {
        // Otherwise, update the local chunk's new_cells directly
        let particle_local_pos = world_to_chunk_local(new_pos);
        new_cells[particle_local_pos.x as usize][particle_local_pos.y as usize] = Some(particle);
    }

    interchunk_move
}
