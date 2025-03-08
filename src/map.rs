use crate::chunk::{Chunk, ACTIVE_CHUNK_RANGE, CHUNK_SIZE};
use crate::particle::{Common, Particle, Special};
use crate::player::Player;
use crate::utils;
use bevy::prelude::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Resource)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Vec<Chunk>>,
    pub active_chunks: HashSet<UVec2>,
}

impl Map {
    /// Create a new empty world with the given width and height.
    pub fn empty(width: u32, height: u32) -> Self {
        // Calculate how many chunks we need
        let chunk_count_width = width.div_ceil(CHUNK_SIZE) as usize;
        let chunk_count_height = height.div_ceil(CHUNK_SIZE) as usize;

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
    fn roll_special_particle(depth: u32, rng: &mut impl Rng) -> Option<Particle> {
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

    /// Generate terrain data for the entire map.
    fn generate_all_data(&self, map_width: u32, map_height: u32) -> Vec<Option<Particle>> {
        let _ = info_span!("generate_map_data_all").entered();

        // Pre-compute all surface heights
        let surface_heights: Vec<u32> = (0..map_width)
            .map(|x| {
                let base_height = (map_height as f32 * 0.95) as u32;
                let height_variation = (x as f32 * 0.05).sin() * 10.0;
                base_height + height_variation as u32
            })
            .collect();

        // Create a single flat vector, processed in parallel chunks
        let result_size = (map_width * map_height) as usize;
        let mut result = vec![None; result_size];

        result
            .par_chunks_mut(map_height as usize)
            .enumerate()
            .for_each(|(x, column)| {
                let _ = info_span!("generate_map_data_thread", width_index = x).entered();
                let mut rng = rand::rng();
                let surface_height = surface_heights[x];

                for y in 0..map_height {
                    column[y as usize] = if y > surface_height {
                        None
                    } else {
                        let depth = surface_height - y;
                        Self::roll_special_particle(depth, &mut rng)
                            .or_else(|| Some(Common::get_exclusive_at_depth(depth).into()))
                    };
                }
            });

        result
    }

    /// Spawn particles based on generated data
    fn distribute_among_chunks(&mut self, spawn_data: Vec<Option<Particle>>) {
        let _ = info_span!("distribute_among_chunks").entered();

        for y in 0..self.height {
            for x in 0..self.width {
                let index = (x * self.height + y) as usize;
                let position = UVec2::new(x, y);
                self.spawn_particle(spawn_data[index], position);
            }
        }
    }

    /// Create a new world with terrain.
    pub fn generate(map_width: u32, map_height: u32) -> Self {
        let _ = info_span!("generate_map", name = "generate_map").entered();
        let mut map = Map::empty(map_width, map_height);

        // Step 1: Generate terrain data
        let spawn_data = map.generate_all_data(map_width, map_height);

        // Step 2: Spawn particles based on the generated data
        map.distribute_among_chunks(spawn_data);

        // Log the composition of the generated world
        map.log_composition();

        map
    }

    // Helper function to spawn a particle at a specific position with proper chunk handling.
    pub fn spawn_particle(&mut self, particle_type: Option<Particle>, position: UVec2) {
        let _ = info_span!("spawn_particle").entered();
        let x = position.x;
        let y = position.y;

        if x >= self.width || y >= self.height {
            return;
        }

        if let Some(particle) = particle_type {
            match particle {
                Particle::Special(Special::Gem(_)) => {
                    self.spawn_gem(particle, position);
                }
                Particle::Special(Special::Ore(_)) => {
                    self.spawn_vein(particle, position);
                }
                _ => {
                    // For common particles, just spawn directly.
                    self.set_particle_at(position, Some(particle));
                }
            }
        } else {
            // For air (None), just update the chunk data.
            self.set_particle_at(position, None);
        }
    }

    /// Spawns a single gem particle at the specified position.
    fn spawn_gem(&mut self, particle: Particle, position: UVec2) {
        // Simply spawn a single particle for gems
        self.set_particle_at(position, Some(particle));
    }

    /// Spawns an ore vein (a small cluster of ore particles) around the specified position.
    fn spawn_vein(&mut self, particle: Particle, position: UVec2) {
        let mut rng = rand::rng();

        // Spawn the central ore particle
        self.set_particle_at(position, Some(particle));

        // Determine vein size (3-6 additional particles)
        let vein_size = rng.random_range(3..=6);

        // Try to spawn additional ore particles in adjacent positions
        for _ in 0..vein_size {
            // Random offset between -1 and 1 in both x and y directions
            let offset_x = rng.random_range(-1..=1);
            let offset_y = rng.random_range(-1..=1);

            // Skip if offset is (0,0) as we already placed a particle there
            if offset_x == 0 && offset_y == 0 {
                continue;
            }

            // Calculate new position
            let new_x = position.x as i32 + offset_x;
            let new_y = position.y as i32 + offset_y;

            // Check bounds
            if new_x < 0 || new_y < 0 || new_x >= self.width as i32 || new_y >= self.height as i32 {
                continue;
            }

            let new_position = UVec2::new(new_x as u32, new_y as u32);

            // 70% chance to place an ore particle
            if rng.random_bool(0.7) {
                self.set_particle_at(new_position, Some(particle));
            }
        }
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

pub fn setup_map(mut commands: Commands) {
    let map = Map::generate(400, 400);
    commands.insert_resource(map);
}

pub fn update_chunks_around_player(
    mut map: ResMut<Map>,
    player_query: Query<&Transform, With<Player>>,
) {
    // Use ACTIVE_CHUNK_RANGE from the chunk module instead of hardcoding the range
    // This ensures consistency across the codebase
    const UPDATE_RANGE: u32 = ACTIVE_CHUNK_RANGE * CHUNK_SIZE;

    if let Ok(player_transform) = player_query.get_single() {
        // Use the coords module to convert screen position to world position
        let player_pos = utils::coords::screen_to_world(
            player_transform.translation.truncate(),
            map.width,
            map.height,
        );

        // Get chunks within range of player using our function
        let active_chunk_positions = map.get_chunks_near(player_pos, UPDATE_RANGE);

        // Debug information
        debug!(
            "Player at world coords: ({}, {}), updating {} nearby chunks",
            player_pos.x,
            player_pos.y,
            active_chunk_positions.len()
        );

        // Update the active chunks cache
        map.active_chunks.clear();
        map.active_chunks
            .extend(active_chunk_positions.iter().cloned());

        // Update any dirty chunks in the active area
        map.update_dirty_chunks();
    }
}

/// System that updates all dirty chunks in the active set
pub fn update_map_dirty_chunks(mut map: ResMut<Map>) {
    map.update_dirty_chunks();
}

/// System that simulates active particles in chunks
pub fn simulate_active_particles(mut map: ResMut<Map>) {
    map.update_active_chunks();
}

/// Plugin that handles the map systems
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map).add_systems(
            Update,
            (
                update_chunks_around_player,
                simulate_active_particles,
                update_map_dirty_chunks,
            ),
        );
    }
}
