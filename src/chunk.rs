use crate::{
    particle::{Particle, PARTICLE_SIZE},
    render::chunk_material::INDICE_BUFFER_SIZE,
    utils::coords,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// The square size of a chunk in particle units (not pixels)
pub(crate) const CHUNK_SIZE: u32 = 32;

/// The range (in chunks) at which chunks are considered active around the player
pub(crate) const ACTIVE_CHUNK_RANGE: u32 = 12;

/// A particle cell contains both the particle data and its corresponding entity (if spawned)
#[derive(Debug, Clone, Default)]
pub struct ParticleCell {
    /// The particle type at this cell, if any
    pub particle: Option<Particle>,
    /// The entity ID for this particle, if spawned
    pub entity: Option<Entity>,
}

/// A chunk represents a square section of the world map
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Position of this chunk in chunk coordinates (not world coordinates)
    pub position: UVec2,
    /// Particles and their entities stored in this chunk, indexed by local coordinates
    /// Only contains entries for cells that have particles or entities
    pub cells: HashMap<UVec2, ParticleCell>,
    /// Whether this chunk has been modified since last update
    pub dirty: bool,
}

impl Chunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            cells: HashMap::new(),
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

    /// Get a particle at the given local position
    pub fn get_particle(&self, local_pos: UVec2) -> Option<Particle> {
        if local_pos.x >= CHUNK_SIZE || local_pos.y >= CHUNK_SIZE {
            return None;
        }
        self.cells.get(&local_pos).and_then(|cell| cell.particle)
    }

    /// Set a particle at the given local position
    pub fn set_particle(&mut self, local_pos: UVec2, particle: Option<Particle>) {
        if local_pos.x >= CHUNK_SIZE || local_pos.y >= CHUNK_SIZE {
            return;
        }

        match particle {
            Some(p) => {
                // Get or create the cell
                let cell = self.cells.entry(local_pos).or_default();
                cell.particle = Some(p);
            }
            None => {
                // If removing a particle, only update if the cell exists
                if let Some(cell) = self.cells.get_mut(&local_pos) {
                    cell.particle = None;

                    // If the cell is now empty (no particle and no entity), remove it entirely
                    if cell.entity.is_none() {
                        self.cells.remove(&local_pos);
                    }
                }
            }
        }

        self.dirty = true;
    }

    /// Spawn all particles in this chunk
    pub fn spawn_particles(&mut self, commands: &mut Commands, map_width: u32, map_height: u32) {
        // First generate a list of all local positions to check
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let local_pos = UVec2::new(x, y);
                let world_pos = self.to_world_coords(local_pos);

                // Skip if outside map bounds
                if world_pos.x >= map_width || world_pos.y >= map_height {
                    continue;
                }

                // Get or create the cell
                if let Some(particle) = self.get_particle(local_pos) {
                    // Get or create a cell entry
                    let cell = self.cells.entry(local_pos).or_default();

                    // Only spawn if the entity doesn't already exist
                    if cell.entity.is_none() {
                        let bundle =
                            ParticleBundle::new(particle, world_pos, map_width, map_height);
                        let entity = commands.spawn(bundle).id();

                        cell.entity = Some(entity);
                    }
                }
            }
        }

        self.dirty = false;
    }

    /// Update particles in this chunk if it's dirty
    pub fn update_particles(&mut self, commands: &mut Commands, map_width: u32, map_height: u32) {
        if !self.dirty {
            return;
        }

        // We need to update all positions in the chunk, not just the ones in the HashMap,
        // because particles might have been added/removed
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let local_pos = UVec2::new(x, y);
                let world_pos = self.to_world_coords(local_pos);

                // Skip if outside map bounds
                if world_pos.x >= map_width || world_pos.y >= map_height {
                    continue;
                }

                let particle = self.get_particle(local_pos);

                // Check if we have a cell for this position
                if let Some(cell) = self.cells.get_mut(&local_pos) {
                    // If there's an existing entity for this position
                    if let Some(entity) = cell.entity {
                        match particle {
                            Some(p) => {
                                // Update existing entity with all components
                                let bundle =
                                    ParticleBundle::new(p, world_pos, map_width, map_height);
                                commands.entity(entity).insert(bundle);
                            }
                            None => {
                                // Remove entity if particle is now None
                                commands.entity(entity).despawn();
                                cell.entity = None;

                                // Remove cell from HashMap if it's now empty
                                if cell.particle.is_none() {
                                    self.cells.remove(&local_pos);
                                }
                            }
                        }
                    } else if let Some(p) = particle {
                        // Spawn new entity if there's a particle but no entity
                        let bundle = ParticleBundle::new(p, world_pos, map_width, map_height);
                        let entity = commands.spawn(bundle).id();

                        cell.entity = Some(entity);
                    }
                } else if let Some(p) = particle {
                    // If there's no cell yet but there's a particle, create one and spawn an entity
                    let bundle = ParticleBundle::new(p, world_pos, map_width, map_height);
                    let entity = commands.spawn(bundle).id();

                    let cell = ParticleCell {
                        particle: Some(p),
                        entity: Some(entity),
                    };

                    self.cells.insert(local_pos, cell);
                }
            }
        }

        self.dirty = false;
    }

    /// Convert the particles in this chunk to a list of spritesheet indices.
    /// Returns a vector of size CHUNK_SIZE * CHUNK_SIZE with the spritesheet indices for each cell.
    /// Cells without particles will have index 0 (transparent).
    pub fn to_spritesheet_indices(&self) -> [Vec4; INDICE_BUFFER_SIZE] {
        let mut indices = [Vec4::ZERO; INDICE_BUFFER_SIZE];

        // Fill in the indices for cells that have particles
        for (local_pos, cell) in &self.cells {
            let index = (local_pos.y * CHUNK_SIZE + local_pos.x) as usize;
            if index < indices.len() {
                if let Some(particle) = cell.particle {
                    indices[index].x = particle.get_spritesheet_index() as f32;
                }
            }
        }

        indices
    }
}

/// Bundle of components for a particle entity
#[derive(Bundle)]
pub struct ParticleBundle {
    pub particle: Particle,
    pub transform: Transform,
}

impl ParticleBundle {
    /// Create a new particle bundle for the given particle at the specified world position
    pub fn new(particle: Particle, world_pos: UVec2, map_width: u32, map_height: u32) -> Self {
        // Convert world position to pixel coordinates
        let pixel_pos = Vec2::new(
            (world_pos.x * PARTICLE_SIZE) as f32,
            (world_pos.y * PARTICLE_SIZE) as f32,
        );

        // Center the position in screen space
        let centered_pos = coords::center_in_screen(pixel_pos, map_width, map_height);

        Self {
            particle,
            transform: Transform::from_xyz(centered_pos.x, centered_pos.y, 0.0),
        }
    }
}
