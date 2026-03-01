use bevy::math::UVec2;
use dashmap::DashMap;

use crate::{
    particle::{
        interaction::{InteractionPair, InteractionType, INTERACTION_RULES},
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

/// The result of attempting to move a particle.
pub enum MoveResult {
    /// Source particle moves to the new position, becoming the given particle.
    /// Used for empty-cell moves and Replace interactions.
    Move(UVec2, Particle),
    /// Source particle stays at its current position. The result particle
    /// should be placed at the target position (Preserve interaction).
    Preserve {
        source_particle: Particle,
        target_pos: UVec2,
        result: Particle,
    },
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

/// Tries to move a particle to a new position, handling interactions and validation.
pub fn try_move(
    context: &SimulationContext,
    new_pos: UVec2,
    particle: Particle,
) -> Option<MoveResult> {
    // First try to move to an empty spot.
    if validate_move_empty(context, new_pos) {
        Some(MoveResult::Move(new_pos, particle))
    } else if let Some((result, interaction_type)) = resolve_interaction(context, new_pos, particle)
    {
        match interaction_type {
            InteractionType::Replace => Some(MoveResult::Move(new_pos, result)),
            InteractionType::Preserve => Some(MoveResult::Preserve {
                source_particle: particle,
                target_pos: new_pos,
                result,
            }),
        }
    } else {
        None
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
    context.map.is_valid_position(new_pos)
        && match context.original_chunk.is_within_chunk(new_pos) {
            // We're within the same new chunk... Let's make sure it's empty in the new chunk too.
            true => {
                let local = world_to_chunk_local(new_pos);
                context.new_cells[local.x as usize][local.y as usize].is_none()
            }
            // Not within the same chunk, so have we already queued a move to this location?
            false => !context.chunk_queue.contains_key(&new_pos),
        }
}

/// Attempts to resolve an interaction between a moving particle and the particle at `new_pos`.
/// Returns the resulting particle and interaction type if an interaction is possible.
fn resolve_interaction(
    context: &SimulationContext,
    new_pos: UVec2,
    particle: Particle,
) -> Option<(Particle, InteractionType)> {
    if !context.map.within_bounds(new_pos) {
        return None;
    }

    // Ensure there's a particle at target...
    let target_particle = context.map.get_particle_at(new_pos)?;

    let interaction_pair = InteractionPair {
        source: particle,
        target: target_particle,
    };

    // Ensure these two particles can interact...
    let rule = INTERACTION_RULES.get(&interaction_pair)?;

    // Now handle whether it's within the same chunk or not.
    if context.original_chunk.is_within_chunk(new_pos) {
        // Check if the new chunk has a valid interaction rule
        let local_pos = world_to_chunk_local(new_pos);
        let new_target = context.new_cells[local_pos.x as usize][local_pos.y as usize]?;
        INTERACTION_RULES
            .get(&InteractionPair {
                source: particle,
                target: new_target,
            })
            .map(|r| (r.result, r.interaction_type))
    } else {
        // If it's outside the chunk, check if it's already queued for movement
        if context.chunk_queue.contains_key(&new_pos) {
            None
        } else {
            Some((rule.result, rule.interaction_type))
        }
    }
}

/// Handles the result of a particle movement calculation, either updating the local chunk
/// or queueing for inter-chunk movement.
pub fn handle_particle_movement(
    original_chunk: &Chunk,
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    source_pos: UVec2,
    new_pos: UVec2,
    particle: Particle,
    preserve_source: bool,
) -> Option<ParticleMove> {
    // If the new position is not within the chunk, queue it for inter-chunk movement.
    if !original_chunk.is_within_chunk(new_pos) {
        Some(ParticleMove {
            source_pos,
            target_pos: new_pos,
            particle,
            preserve_source,
        })
    } else {
        // Otherwise, update the local chunk's new_cells directly
        let particle_local_pos = world_to_chunk_local(new_pos);
        new_cells[particle_local_pos.x as usize][particle_local_pos.y as usize] = Some(particle);
        None
    }
}
