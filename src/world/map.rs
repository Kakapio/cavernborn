use crate::chunk::{Chunk, ACTIVE_CHUNK_RANGE, CHUNK_SIZE};
use crate::particle::{Particle, Special};
use crate::player::Player;
use crate::utils;
use crate::utils::coords::{screen_to_world, world_vec2_to_chunk};
use crate::world::generator::generate_all_data;
use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::ThreadRng;
use std::collections::HashMap;
use std::collections::HashSet;

/// The rate at which the map is simulated per second.
pub(crate) const SIMULATION_RATE: f64 = 40.0;

#[derive(Resource)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Vec<Chunk>>,
    pub active_chunks: HashSet<UVec2>,
}

// Function to get chunk count width
pub fn chunk_count_width(map_width: u32) -> u32 {
    map_width.div_ceil(CHUNK_SIZE)
}

// Function to get chunk count height
pub fn chunk_count_height(map_height: u32) -> u32 {
    map_height.div_ceil(CHUNK_SIZE)
}

impl Map {
    /// Create a new empty world with the given width and height.
    pub fn empty(width: u32, height: u32) -> Self {
        // Calculate how many chunks we need
        let chunk_count_width = chunk_count_width(width) as usize;
        let chunk_count_height = chunk_count_height(height) as usize;

        let mut chunks: Vec<Vec<Chunk>> = vec![vec![]; chunk_count_width];

        // Initialize all chunks
        for (cx, chunk_col) in chunks.iter_mut().enumerate().take(chunk_count_width) {
            *chunk_col = Vec::with_capacity(chunk_count_height);
            for cy in 0..chunk_count_height {
                let chunk_pos = UVec2::new(cx as u32, cy as u32);
                chunk_col.push(Chunk::new(chunk_pos));
            }
        }

        Self {
            width,
            height,
            chunks,
            active_chunks: HashSet::new(),
        }
    }

    /// Analyze and log the composition of the world
    fn log_composition(&self) {
        let mut particle_counts: HashMap<Particle, u32> = HashMap::new();
        let mut total_particles = 0;

        // Count particles
        for chunk_col in self.chunks.iter() {
            for chunk in chunk_col.iter() {
                let chunk_composition = chunk.get_composition();

                for (particle, count) in chunk_composition {
                    *particle_counts.entry(particle).or_insert(0) += count;
                    total_particles += count;
                }
            }
        }

        let air_count = self.width * self.height - total_particles;

        let total_cells = total_particles + air_count;

        // Log results
        info!("\nMap Composition Analysis:");
        info!("Total cells: {}", total_cells);
        info!("Solid particles: {}", total_particles);
        info!(
            "Air particles: {} ({:.2}%)",
            air_count,
            (air_count as f32 / total_cells as f32) * 100.0
        );
        info!("Breakdown by type:");

        // Convert to vec for sorting
        let mut counts: Vec<_> = particle_counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count, descending

        for (particle_type, count) in counts {
            let percentage = (count as f32 / total_cells as f32) * 100.0;
            info!(
                "{:?}: {} particles ({:.2}%)",
                particle_type, count, percentage
            );
        }
    }

    /// Uses a weighted random roll to determine if a special particle should spawn, and if so, which one.
    /// Returns `None` if no special particle should spawn.
    pub(crate) fn roll_special_particle(depth: u32, rng: &mut ThreadRng) -> Option<Particle> {
        // Get valid special particles for this depth
        let mut valid_particles: Vec<_> = Special::all_variants()
            .into_iter()
            .filter(|p| depth >= p.min_depth() && depth < p.max_depth())
            .collect();

        if valid_particles.is_empty() {
            return None;
        }

        // Sort particles from lowest to highest spawn chance
        valid_particles.sort_unstable_by_key(|p| p.spawn_chance());

        // Calculate total spawn weight
        let total_weight: i32 = valid_particles.iter().map(|p| p.spawn_chance()).sum();

        // First check: determine if we spawn any special particle
        // Illustration of the first check:
        // [0 ... total_weight ... 1000]
        //  |<---spawn--->|<---no spawn--->|
        //        ^random point
        if rng.random_range(0..1000) >= total_weight {
            return None;
        }

        // Second check: weighted selection of which particle to spawn
        // Illustration of the second check:
        // [0 ... p1 ... p2 ... p3 ... total_weight]
        //  |<-p1->|<-p2->|<-p3->|
        //        ^random point
        let random_val = rng.random_range(0..total_weight);

        // Use fold to perform weighted selection in a more functional way
        valid_particles
            .iter()
            // Start off with weight 0 and Air tile.
            .fold((0, None), |(acc_weight, selected), &special| {
                // Get this iteration's weight.
                let new_weight = acc_weight + special.spawn_chance();

                // No particle yet. "Hit" condition is random value is less than the new weight.
                if selected.is_none() && random_val < new_weight {
                    (new_weight, Some(special))
                } else {
                    // Otherwise, account for the failed roll.
                    (new_weight, selected)
                }
            })
            .1
            .map(Particle::Special)
    }

    /// Distribute chunks into the 2D vector structure
    fn distribute_among_chunks(&mut self, chunks_vec: Vec<Chunk>) {
        // Convert chunks vector back to our 2D vector structure
        for (i, chunk) in chunks_vec.iter().enumerate() {
            let cw = chunk_count_width(self.width);
            let x = i % cw as usize;
            let y = i / cw as usize;
            self.chunks[x][y] = chunk.clone();
        }
    }

    /// Create a new world with terrain.
    /// - `width`: Number of chunks wide the map should be
    /// - `height`: Number of chunks tall the map should be
    pub fn generate(width: u32, height: u32) -> Self {
        let _ = info_span!("map_generate").entered();
        let start_total = std::time::Instant::now();

        // Convert chunk counts to particle dimensions
        let map_width = width * CHUNK_SIZE;
        let map_height = height * CHUNK_SIZE;

        // Create an empty map
        let mut map = Map::empty(map_width, map_height);

        // Generate all map data and get the populated chunks
        let chunks_vec = generate_all_data(map_width, map_height);

        // Distribute chunks into the 2D vector structure
        map.distribute_among_chunks(chunks_vec);

        // Print composition statistics
        let start_log = std::time::Instant::now();
        map.log_composition();
        println!("log_composition took: {:?}", start_log.elapsed());

        // Print total time
        println!("Total Map::generate time: {:?}", start_total.elapsed());

        map
    }

    /// Helper function to get a particle at the specified position
    #[expect(dead_code)]
    pub fn get_particle_at(&self, position: UVec2) -> Option<Particle> {
        if position.x >= self.width || position.y >= self.height {
            return None;
        }

        let chunk_pos = utils::coords::world_to_chunk(position);
        let local_pos = utils::coords::world_to_local(position);

        let chunk = &self.chunks[chunk_pos.x as usize][chunk_pos.y as usize];
        chunk.get_particle(local_pos)
    }

    /// Helper function to set a particle at the specified map position while handling chunk boundaries.
    pub fn set_particle_at(&mut self, position: UVec2, particle: Option<Particle>) {
        if position.x >= self.width || position.y >= self.height {
            return;
        }

        let chunk_pos = utils::coords::world_to_chunk(position);
        let local_pos = utils::coords::world_to_local(position);

        let chunk = &mut self.chunks[chunk_pos.x as usize][chunk_pos.y as usize];
        chunk.set_particle(local_pos, particle);
    }

    /// Returns a list of chunk positions within a radius of the given world position
    ///
    /// # Arguments
    ///
    /// * `position` - The world position to check around
    /// * `range` - The range in world units
    ///
    /// # Returns
    ///
    /// A vector of chunk positions (in chunk coordinates) within the specified range
    pub fn get_chunks_near(&self, position: Vec2, range: u32) -> Vec<UVec2> {
        let center_chunk = utils::coords::world_vec2_to_chunk(position);
        let chunk_range = range.div_ceil(CHUNK_SIZE);

        let mut nearby_chunks = Vec::new();

        // Calculate the bounds of the square area that contains the circle
        let min_x = center_chunk.x.saturating_sub(chunk_range);
        let max_x = center_chunk.x.saturating_add(chunk_range);
        let min_y = center_chunk.y.saturating_sub(chunk_range);
        let max_y = center_chunk.y.saturating_add(chunk_range);

        // Calculate map bounds in chunk coordinates
        let max_chunk_x = self.width.div_ceil(CHUNK_SIZE) - 1;
        let max_chunk_y = self.height.div_ceil(CHUNK_SIZE) - 1;

        // Collect all chunk positions within the circular range and map bounds
        for x in min_x..=max_x {
            // Skip if outside map bounds
            if x > max_chunk_x {
                continue;
            }

            for y in min_y..=max_y {
                // Skip if outside map bounds
                if y > max_chunk_y {
                    continue;
                }

                let chunk_pos = UVec2::new(x, y);

                // Calculate squared distance to avoid using sqrt
                let dx = if center_chunk.x > chunk_pos.x {
                    center_chunk.x - chunk_pos.x
                } else {
                    chunk_pos.x - center_chunk.x
                };

                let dy = if center_chunk.y > chunk_pos.y {
                    center_chunk.y - chunk_pos.y
                } else {
                    chunk_pos.y - center_chunk.y
                };

                // Use squared distance comparison to avoid square root calculation
                let squared_distance = (dx * dx + dy * dy) as f32;
                let squared_range = (chunk_range * chunk_range) as f32;

                if squared_distance <= squared_range {
                    nearby_chunks.push(chunk_pos);
                }
            }
        }

        nearby_chunks
    }

    /// Update all active chunks that are marked as dirty.
    pub fn update_dirty_chunks(&mut self) {
        for chunk_pos in self.active_chunks.iter() {
            let chunk = &mut self.chunks[chunk_pos.x as usize][chunk_pos.y as usize];
            if chunk.dirty {
                chunk.trigger_refresh();
            }
        }
    }

    /// Trigger a simulation of active particles in all active chunks.
    pub fn update_active_chunks(&mut self) {
        // Clone the active_chunks to avoid borrowing issues
        let active_chunks: Vec<UVec2> = self.active_chunks.iter().cloned().collect();

        for chunk_pos in active_chunks {
            let chunk = &mut self.chunks[chunk_pos.x as usize][chunk_pos.y as usize];
            if chunk.has_active_particles {
                // Call the new simulate method instead of trigger_refresh
                chunk.simulate();
            }
        }
    }

    // Get a chunk at a specific position in local map coordinates.
    pub fn get_chunk_at(&self, position: &UVec2) -> &Chunk {
        &self.chunks[position.x as usize][position.y as usize]
    }
}

/// Updates the active chunks to be those around the player.
pub fn update_active_chunks(mut map: ResMut<Map>, player_query: Query<&Transform, With<Player>>) {
    // Use ACTIVE_CHUNK_RANGE from the chunk module for consistency
    const UPDATE_RANGE: u32 = ACTIVE_CHUNK_RANGE;

    if let Ok(player_transform) = player_query.get_single() {
        // Convert screen position to world position
        let player_pos = screen_to_world(
            player_transform.translation.truncate(),
            map.width,
            map.height,
        );

        // Convert player world position to chunk position
        let center_chunk = world_vec2_to_chunk(player_pos);

        // Calculate map bounds in chunk coordinates
        let max_chunk_x = map.width.div_ceil(CHUNK_SIZE) - 1;
        let max_chunk_y = map.height.div_ceil(CHUNK_SIZE) - 1;

        // Calculate the rectangular bounds around the player
        let min_x = center_chunk.x.saturating_sub(UPDATE_RANGE);
        let max_x = (center_chunk.x + UPDATE_RANGE).min(max_chunk_x);
        let min_y = center_chunk.y.saturating_sub(UPDATE_RANGE);
        let max_y = (center_chunk.y + UPDATE_RANGE).min(max_chunk_y);

        // Debug information
        debug!(
            "Player at world coords: ({}, {}), updating rectangular chunk region: x={}..{}, y={}..{}",
            player_pos.x, player_pos.y, min_x, max_x, min_y, max_y
        );

        // Clear the current active chunks
        map.active_chunks.clear();

        // Add all chunks in the rectangular region to active_chunks
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                map.active_chunks.insert(UVec2::new(x, y));
            }
        }

        // Update any dirty chunks in the active area
        map.update_dirty_chunks();
    }
}
