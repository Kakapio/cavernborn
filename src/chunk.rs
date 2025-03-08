use crate::{
    particle::{Fluid, Particle, ParticleType},
    render::chunk_material::INDICE_BUFFER_SIZE,
};
use bevy::{prelude::*, utils::HashMap};

/// The square size of a chunk in particle units (not pixels)
/// Note: If you modify this, you must update the shader's indices buffer size.
pub(crate) const CHUNK_SIZE: u32 = 32;

/// The range (in chunks) at which chunks are considered active around the player
pub(crate) const ACTIVE_CHUNK_RANGE: u32 = 12;

/// A chunk represents a square section of the world map
#[derive(Debug, Clone)]
pub struct Chunk {
    #[allow(dead_code)]
    /// Position of this chunk in chunk coordinates (not world coordinates)
    pub position: UVec2,
    /// Particles stored in this chunk, indexed by local coordinates
    /// Only contains entries for cells that have particles
    pub cells: [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    /// Whether this chunk has been modified since last update
    pub dirty: bool,
    /// Whether this chunk contains any fluid particles that need active simulation
    pub has_active_particles: bool,
}

impl Chunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            cells: [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            dirty: false,
            has_active_particles: false,
        }
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

        // Update has_active_particles flag based on chunk contents
        self.update_active_state();
    }

    /// Updates the has_active_particles flag by checking if any cells contain fluid particles
    fn update_active_state(&mut self) {
        self.has_active_particles = false;

        // Scan the chunk for any fluid particles
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if let Some(Particle::Fluid(_)) = self.cells[x as usize][y as usize] {
                    self.has_active_particles = true;
                    return; // Early return once we find a fluid
                }
            }
        }
    }

    /// Load all particles in this chunk from hard drive.
    /// TODO: This will be useful with dynamically loaded chunks.
    #[expect(dead_code, unused_variables)]
    pub fn load_particles(&mut self, map_width: u32, map_height: u32) {
        todo!();
    }

    /// Update particles in this chunk if it's dirty
    pub fn trigger_refresh(&mut self) {
        if !self.dirty {
            return;
        }

        // TODO: Perform logic for collider regeneration, etc. here.

        // Always update the active state when processing a dirty chunk
        self.update_active_state();

        self.dirty = false;
    }

    /// Simulate active particles (like fluids) in this chunk
    pub fn simulate(&mut self) {
        // Only proceed if this chunk has active particles
        if !self.has_active_particles {
            return;
        }

        // Create a copy of the current state to read from
        let original_cells = self.cells;
        // Create a new state to write to (initially empty)
        let mut new_cells = [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Process all particles in the chunk
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if let Some(particle) = original_cells[x as usize][y as usize] {
                    // Calculate the new position for this particle.
                    match particle {
                        Particle::Common(_) | Particle::Special(_) => {
                            // Non-fluid particles stay in place.
                            new_cells[x as usize][y as usize] = Some(particle);
                        }
                        Particle::Fluid(fluid) => {
                            // For fluids, calculate new position using the original state.
                            simulate_fluid(&original_cells, &mut new_cells, fluid, x, y);
                        }
                    }
                }
                // Empty cells remain empty in the new state
            }
        }

        // Update the chunk with the new state
        self.cells = new_cells;

        // Mark the chunk as dirty after simulation to ensure rendering updates
        self.dirty = true;
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
}

/// Calculates the new position for a fluid particle, reading from original_cells and writing to new_cells
fn simulate_fluid(
    original_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    fluid: Fluid,
    x: u32,
    y: u32,
) {
    let buoyancy = fluid.get_buoyancy();

    // Skip if buoyancy is 0 (no movement)
    if buoyancy == 0 {
        // Keep the fluid in place
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid));
        return;
    }

    // Determine the vertical direction and check boundaries
    let new_y = y as i32 + buoyancy;

    // Check if we're at a vertical boundary
    if new_y < 0 || new_y >= CHUNK_SIZE as i32 {
        // Keep the fluid in place at the boundary
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid));
        return;
    }

    let new_y = new_y as usize;

    // Try to move in one of three directions based on priority
    // Note: We check the original cells for obstacles, but write to new_cells
    if original_cells[x as usize][new_y].is_none() && new_cells[x as usize][new_y].is_none() {
        // Move vertically
        new_cells[x as usize][new_y] = Some(Particle::Fluid(fluid));
    } else if x > 0
        && original_cells[(x - 1) as usize][new_y].is_none()
        && new_cells[(x - 1) as usize][new_y].is_none()
    {
        // Move diagonally to the left
        new_cells[(x - 1) as usize][new_y] = Some(Particle::Fluid(fluid));
    } else if x < CHUNK_SIZE - 1
        && original_cells[(x + 1) as usize][new_y].is_none()
        && new_cells[(x + 1) as usize][new_y].is_none()
    {
        // Move diagonally to the right
        new_cells[(x + 1) as usize][new_y] = Some(Particle::Fluid(fluid));
    } else {
        // If fluid can't move in any of these directions, it stays in place
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid));
    }
}
