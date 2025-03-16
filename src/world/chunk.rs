use std::{collections::HashMap, sync::Arc};

use crate::{
    particle::{
        interaction::{InteractionPair, INTERACTION_RULES},
        Particle, ParticleType,
    },
    render::chunk_material::INDICE_BUFFER_SIZE,
    simulation::{fluid::FluidSimulator, SimulationContext, Simulator},
};
use bevy::prelude::*;
use dashmap::DashMap;

use super::Map;

/// The square size of a chunk in particle units (not pixels).
/// Note: If you modify this, you must update the shader's indices buffer size.
pub(crate) const CHUNK_SIZE: u32 = 32;

/// The range (in chunks) at which chunks are considered active around the player.
pub(crate) const ACTIVE_CHUNK_RANGE: u32 = 12;

/// Represents a particle that needs to move to a new position. Used in queue system.
/// Note: This is used in a HashMap where the key is the target position, which is why we don't store it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ParticleMove {
    /// Source position in world coordinates
    pub source_pos: UVec2,
    /// Target position in world coordinates
    pub target_pos: UVec2,
    /// The particle to move
    pub particle: Particle,
}

/// A chunk represents a square section of the world map
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Position of this chunk in chunk coordinates (not world coordinates)
    pub position: UVec2,
    /// Particles stored in this chunk, indexed by local coordinates
    /// Only contains entries for cells that have particles
    pub cells: [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Whether this chunk has been modified since last update
    pub dirty: bool,
    /// Whether this chunk is non-homogenous and needs active simulation
    pub should_simulate: bool,
    /// Cached world-coordinate boundaries of this chunk
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
}

impl Chunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            cells: [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            dirty: false,
            should_simulate: false,
            x_min: position.x * CHUNK_SIZE,
            x_max: (position.x + 1) * CHUNK_SIZE,
            y_min: position.y * CHUNK_SIZE,
            y_max: (position.y + 1) * CHUNK_SIZE,
        }
    }

    /// Get a particle at the given local position. None if out of bounds.
    pub fn get_particle(&self, local_pos: UVec2) -> Option<Particle> {
        if !self.is_in_bounds(local_pos) {
            return None;
        }
        self.cells[local_pos.x as usize][local_pos.y as usize]
    }

    /// Set a particle at the given local position
    pub fn set_particle(&mut self, local_pos: UVec2, particle: Option<Particle>) {
        if !self.is_in_bounds(local_pos) {
            return;
        }

        self.cells[local_pos.x as usize][local_pos.y as usize] = particle;
        self.dirty = true;
    }

    /// Updates the should_simulate flag by checking if the chunk contains any fluid particles.
    fn update_active_state(&mut self) {
        self.should_simulate = false;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if let Some(Particle::Liquid(_)) = self.cells[x as usize][y as usize] {
                    self.should_simulate = true;
                    return; // Early return once we find a fluid
                }
            }
        }
    }

    /// Update particles in this chunk if it's dirty
    pub fn trigger_refresh(&mut self) {
        if !self.dirty {
            return;
        }

        // TODO: Perform logic for collider regeneration, etc. here.

        // Did an active particle enter or leave this chunk?
        self.update_active_state();

        self.dirty = false;
    }

    /// Simulate active particles (like fluids) in this chunk.
    /// This method handles simulation for particles that stay within this chunk.
    pub fn simulate(
        &mut self,
        map: &Map,
        interchunk_queue: Arc<DashMap<UVec2, ParticleMove>>,
    ) -> Chunk {
        // Only proceed if this chunk has active particles.
        if !self.should_simulate {
            return self.clone();
        }

        // Create a copy of the current state to read from.
        let original_cells = self.cells;
        // Create a new state to write to (initially empty).
        let mut new_cells = [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Process all particles in the chunk.
        for (x, column) in original_cells.iter().enumerate() {
            for (y, &particle) in column.iter().enumerate() {
                // Skip empty cells.
                let Some(particle) = particle else { continue };

                match particle {
                    Particle::Liquid(fluid) => {
                        // For fluids, calculate new position using the original state.
                        // This will append to the queue of ParticleMoves if there is interchunk movement.
                        if let Some(particle_move) = FluidSimulator.simulate(
                            SimulationContext::new(
                                map,
                                self,
                                interchunk_queue.as_ref(),
                                &mut new_cells,
                            ),
                            fluid,
                            x as u32,
                            y as u32,
                        ) {
                            interchunk_queue
                                .entry(particle_move.target_pos)
                                .and_modify(|existing| {
                                    // Very occasionally, we get a race condition where two particles
                                    // try to move to the same position at the same time.
                                    // This is a hacky fix that allows the closer particle to take priority.
                                    // TODO: Hacky fix.
                                    let particle_move = particle_move.clone();
                                    // Calculate Manhattan distance for both particles
                                    let existing_distance = (existing.source_pos.x as i32
                                        - existing.target_pos.x as i32)
                                        .abs()
                                        + (existing.source_pos.y as i32
                                            - existing.target_pos.y as i32)
                                            .abs();

                                    let new_distance = (particle_move.source_pos.x as i32
                                        - particle_move.target_pos.x as i32)
                                        .abs()
                                        + (particle_move.source_pos.y as i32
                                            - particle_move.target_pos.y as i32)
                                            .abs();

                                    // Particle that's closer to the target position wins
                                    if new_distance < existing_distance {
                                        *existing = particle_move;
                                    }
                                    // If equal distance, we could use a deterministic tiebreaker like particle ID or properties
                                })
                                .or_insert(particle_move);
                        }
                    }
                    _ => new_cells[x][y] = Some(particle),
                }
            }
        }

        // Update the chunk with the new state. Swap is fast.
        std::mem::swap(&mut self.cells, &mut new_cells);

        // Mark the chunk as dirty after simulation to ensure other systems update.
        self.dirty = true;

        self.clone()
    }

    /// Process interactions between particles in this chunk.
    pub fn process_interactions(&mut self) -> Chunk {
        // Create a copy of the current state to read from.
        let original_cells = self.cells;
        // Create a new state to write to (initially empty).
        let mut new_cells = [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Process all particles in the chunk.
        for x in 0..CHUNK_SIZE as usize {
            for y in 0..CHUNK_SIZE as usize {
                // Skip empty cells.
                if y == CHUNK_SIZE as usize - 1 {
                    new_cells[x][y] = original_cells[x][y];
                    continue;
                }

                let Some(particle_above) = original_cells[x][y + 1] else {
                    new_cells[x][y] = original_cells[x][y];
                    continue;
                };
                let Some(particle_below) = original_cells[x][y] else {
                    new_cells[x][y] = original_cells[x][y];
                    continue;
                };

                if let Some(rule) = INTERACTION_RULES.get(&InteractionPair {
                    source: particle_above,
                    target: particle_below,
                }) {
                    new_cells[x][y] = rule.result;
                } else {
                    new_cells[x][y] = original_cells[x][y];
                }
            }
        }

        std::mem::swap(&mut self.cells, &mut new_cells);
        self.dirty = true;

        self.clone()
    }

    /// Convert the particles in this chunk to a list of spritesheet indices.
    /// Returns a vector of size CHUNK_SIZE * CHUNK_SIZE with the spritesheet indices for each cell.
    /// Cells without particles will have index 0 (transparent).
    pub fn to_spritesheet_indices(&self) -> [Vec4; INDICE_BUFFER_SIZE] {
        let mut indices = [Vec4::ZERO; INDICE_BUFFER_SIZE];
        // Fill in the indices for cells that have particles
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let index = (y * CHUNK_SIZE + x) as usize;
                if index < indices.len() {
                    if let Some(particle) = self.cells[x as usize][y as usize] {
                        indices[index].x = particle.get_spritesheet_index() as f32;
                    }
                }
            }
        }

        indices
    }

    pub fn get_composition(&self) -> HashMap<Particle, u32> {
        let mut composition = HashMap::new();
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if let Some(particle) = self.cells[x as usize][y as usize] {
                    *composition.entry(particle).or_insert(0) += 1;
                }
            }
        }
        composition
    }

    /// Checks if the given local position is within chunk bounds.
    pub fn is_in_bounds(&self, local_pos: UVec2) -> bool {
        local_pos.x < CHUNK_SIZE && local_pos.y < CHUNK_SIZE
    }

    /// Checks if the given world position is within this chunk.
    pub fn is_within_chunk(&self, world_pos: UVec2) -> bool {
        world_pos.x >= self.x_min
            && world_pos.x < self.x_max
            && world_pos.y >= self.y_min
            && world_pos.y < self.y_max
    }
}
