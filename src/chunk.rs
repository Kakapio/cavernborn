use crate::particle::{Particle, ParticleBundle, PARTICLE_SIZE};
use bevy::prelude::*;
use std::collections::HashMap;

/// The square size of a chunk in particle units (not pixels)
pub const CHUNK_SIZE: u32 = 32;

/// A chunk represents a square section of the world map
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Position of this chunk in chunk coordinates (not world coordinates)
    pub position: UVec2,
    /// Particles stored in this chunk, indexed by local coordinates (0,0) to (CHUNK_SIZE-1, CHUNK_SIZE-1)
    pub particles: Vec<Vec<Option<Particle>>>,
    /// Whether this chunk has been modified since last update
    pub dirty: bool,
    /// Entity IDs for each particle in this chunk, used for updating/removing entities
    pub entity_map: HashMap<UVec2, Entity>,
}

impl Chunk {
    /// Create a new empty chunk at the given chunk position
    pub fn new(position: UVec2) -> Self {
        Self {
            position,
            particles: vec![vec![None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            dirty: false,
            entity_map: HashMap::new(),
        }
    }

    /// Convert world coordinates to local chunk coordinates
    pub fn world_to_local(world_pos: UVec2) -> UVec2 {
        UVec2::new(world_pos.x % CHUNK_SIZE, world_pos.y % CHUNK_SIZE)
    }

    /// Convert world coordinates to chunk coordinates
    pub fn world_to_chunk(world_pos: UVec2) -> UVec2 {
        UVec2::new(world_pos.x / CHUNK_SIZE, world_pos.y / CHUNK_SIZE)
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
        self.particles[local_pos.x as usize][local_pos.y as usize]
    }

    /// Set a particle at the given local position
    pub fn set_particle(&mut self, local_pos: UVec2, particle: Option<Particle>) {
        if local_pos.x >= CHUNK_SIZE || local_pos.y >= CHUNK_SIZE {
            return;
        }
        self.particles[local_pos.x as usize][local_pos.y as usize] = particle;
        self.dirty = true;
    }

    /// Spawn all particles in this chunk
    pub fn spawn_particles(&mut self, commands: &mut Commands, map_width: u32, map_height: u32) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let local_pos = UVec2::new(x, y);
                let world_pos = self.to_world_coords(local_pos);

                // Skip if outside map bounds
                if world_pos.x >= map_width || world_pos.y >= map_height {
                    continue;
                }

                if let Some(particle) = self.get_particle(local_pos) {
                    let entity = commands
                        .spawn(ParticleBundle {
                            particle_type: particle,
                            sprite: SpriteBundle {
                                sprite: particle.create_sprite(),
                                transform: Transform::from_xyz(
                                    (world_pos.x * PARTICLE_SIZE) as f32
                                        - ((map_width * PARTICLE_SIZE) / 2) as f32,
                                    (world_pos.y * PARTICLE_SIZE) as f32
                                        - ((map_height * PARTICLE_SIZE) / 2) as f32,
                                    0.0,
                                ),
                                ..default()
                            },
                        })
                        .id();

                    self.entity_map.insert(local_pos, entity);
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

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let local_pos = UVec2::new(x, y);
                let world_pos = self.to_world_coords(local_pos);

                // Skip if outside map bounds
                if world_pos.x >= map_width || world_pos.y >= map_height {
                    continue;
                }

                let particle = self.get_particle(local_pos);

                // If there's an existing entity for this position
                if let Some(entity) = self.entity_map.get(&local_pos) {
                    match particle {
                        Some(p) => {
                            // Update existing entity
                            commands.entity(*entity).insert(p);
                        }
                        None => {
                            // Remove entity if particle is now None
                            commands.entity(*entity).despawn();
                            self.entity_map.remove(&local_pos);
                        }
                    }
                } else if let Some(p) = particle {
                    // Spawn new entity if there's a particle but no entity
                    let entity = commands
                        .spawn(ParticleBundle {
                            particle_type: p,
                            sprite: SpriteBundle {
                                sprite: p.create_sprite(),
                                transform: Transform::from_xyz(
                                    (world_pos.x * PARTICLE_SIZE) as f32
                                        - ((map_width * PARTICLE_SIZE) / 2) as f32,
                                    (world_pos.y * PARTICLE_SIZE) as f32
                                        - ((map_height * PARTICLE_SIZE) / 2) as f32,
                                    0.0,
                                ),
                                ..default()
                            },
                        })
                        .id();

                    self.entity_map.insert(local_pos, entity);
                }
            }
        }

        self.dirty = false;
    }

    /// Check if this chunk is within range of a given position
    pub fn is_within_range(&self, position: UVec2, range: u32) -> bool {
        let chunk_pos = Self::world_to_chunk(position);
        let dx = if self.position.x > chunk_pos.x {
            self.position.x - chunk_pos.x
        } else {
            chunk_pos.x - self.position.x
        };

        let dy = if self.position.y > chunk_pos.y {
            self.position.y - chunk_pos.y
        } else {
            chunk_pos.y - self.position.y
        };

        // Convert range from world units to chunk units
        let chunk_range = range.div_ceil(CHUNK_SIZE);

        dx <= chunk_range && dy <= chunk_range
    }
}
