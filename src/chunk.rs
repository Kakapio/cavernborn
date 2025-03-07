use crate::{particle::Particle, render::chunk_material::INDICE_BUFFER_SIZE};
use bevy::prelude::*;

/// The square size of a chunk in particle units (not pixels)
pub(crate) const CHUNK_SIZE: u32 = 32;

/// The range (in chunks) at which chunks are considered active around the player
pub(crate) const ACTIVE_CHUNK_RANGE: u32 = 12;

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
}

impl Chunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            cells: [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            dirty: false,
        }
    }

    /// Convert chunk coordinates and local coordinates to world coordinates
    pub fn to_world_coords(&self, local_pos: UVec2) -> UVec2 {
        UVec2::new(
            self.position.x * CHUNK_SIZE + local_pos.x,
            self.position.y * CHUNK_SIZE + local_pos.y,
        )
    }

    /// Get a particle at the given local position. None if out of bounds.
    pub fn get_particle(&self, local_pos: UVec2) -> Option<Particle> {
        if local_pos.x >= CHUNK_SIZE || local_pos.y >= CHUNK_SIZE {
            return None;
        }
        self.cells[local_pos.x as usize][local_pos.y as usize]
    }

    /// Set a particle at the given local position
    pub fn set_particle(&mut self, local_pos: UVec2, particle: Option<Particle>) {
        if local_pos.x >= CHUNK_SIZE || local_pos.y >= CHUNK_SIZE {
            return;
        }

        self.cells[local_pos.x as usize][local_pos.y as usize] = particle;

        self.dirty = true;
    }

    /// Load all particles in this chunk from hard drive.
    /// TODO: This will be useful with dynamically loaded chunks.
    #[expect(dead_code, unused_variables)]
    pub fn load_particles(&mut self, map_width: u32, map_height: u32) {
        todo!();
    }

    /// Update particles in this chunk if it's dirty
    #[expect(unused_variables)]
    pub fn update_particles(&mut self, map_width: u32, map_height: u32) {
        if !self.dirty {
            return;
        }

        //TODO: Perform logic for collider regeneration, etc. here.

        self.dirty = false;
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
}
