use crate::chunk::{Chunk, ACTIVE_CHUNK_RANGE, CHUNK_SIZE};
use crate::particle::{Common, Particle, Special};
use crate::player::Player;
use crate::utils;
use bevy::prelude::*;
use rand::prelude::*;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

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

struct UnsafeChunkData {
    chunks: UnsafeCell<Vec<Chunk>>,
}

unsafe impl Sync for UnsafeChunkData {}

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
    fn generate_all_data(&self, map_width: u32, map_height: u32) -> Vec<Chunk> {
        let _ = info_span!("generate_map_data_all").entered();
        let start_method = std::time::Instant::now();

        // Pre-compute all surface heights
        let start_surface = std::time::Instant::now();
        let surface_heights: Vec<u32> = (0..map_width)
            .map(|x| {
                let base_height = (map_height as f32 * 0.95) as u32;
                let height_variation = (x as f32 * 0.05).sin() * 10.0;
                base_height + height_variation as u32
            })
            .collect();
        println!(
            "  Surface heights calculation took: {:?}",
            start_surface.elapsed()
        );

        // Calculate chunk counts
        let chunks_width = chunk_count_width(map_width);
        let chunks_height = chunk_count_height(map_height);

        // Create empty chunks - use Vec instead of fixed-size array
        let mut chunks = Vec::with_capacity(chunks_width as usize * chunks_height as usize);
        for x in 0..chunks_width {
            for y in 0..chunks_height {
                chunks.push(Chunk::new(UVec2::new(x, y)));
            }
        }

        // Create unsafe wrapper to allow parallel writing
        let unsafe_data = Arc::new(UnsafeChunkData {
            chunks: UnsafeCell::new(chunks),
        });

        // Determine number of threads to use
        let num_cpus = num_cpus::get();
        let chunk_size = (map_width as usize / num_cpus).max(1);

        // Process columns in parallel
        let start_parallel = std::time::Instant::now();
        let mut handles = Vec::new();

        for thread_id in 0..num_cpus {
            let unsafe_data_clone = Arc::clone(&unsafe_data);
            let surface_heights_clone = surface_heights.clone();

            let start_x = thread_id * chunk_size;
            let end_x = if thread_id == num_cpus - 1 {
                map_width as usize
            } else {
                (thread_id + 1) * chunk_size
            };

            handles.push(std::thread::spawn(move || {
                let _ = info_span!(
                    "generate_map_data_thread",
                    width_range = format!("{}..{}", start_x, end_x)
                )
                .entered();
                let mut rng = rand::rng();

                for (x, _) in surface_heights_clone
                    .iter()
                    .enumerate()
                    .skip(start_x)
                    .take(end_x - start_x)
                {
                    let surface_height = surface_heights_clone[x];

                    for y in 0..map_height as usize {
                        let position = UVec2::new(x as u32, y as u32);
                        let special_particle = if y as u32 > surface_height {
                            None
                        } else {
                            let depth = surface_height - y as u32;
                            Map::roll_special_particle(depth, &mut rng)
                        };

                        if let Some(Particle::Special(special)) = special_particle {
                            let out = match special {
                                Special::Ore(_) => spawn_vein(
                                    position,
                                    Particle::Special(special),
                                    map_width,
                                    map_height,
                                ),
                                Special::Gem(_) => spawn_gem(position, Particle::Special(special)),
                            };

                            // Place all the spawned particles
                            for (spawn_pos, particle) in out {
                                let chunk_pos = utils::coords::world_to_chunk(spawn_pos);
                                let local_pos = utils::coords::world_to_local(spawn_pos);
                                let cw = chunks_width;
                                let chunk_index = (chunk_pos.x + chunk_pos.y * cw) as usize;

                                // Use unsafe to set the particle in the shared chunk data
                                unsafe {
                                    let chunks = &mut *unsafe_data_clone.chunks.get();
                                    chunks[chunk_index].set_particle(local_pos, Some(particle));
                                }
                            }
                        } else if y as u32 <= surface_height {
                            // If no special particle was rolled, use common particle
                            let depth = surface_height - y as u32;
                            let common_particle = Common::get_exclusive_at_depth(depth).into();

                            // Convert world position to chunk and local coordinates
                            let chunk_pos = utils::coords::world_to_chunk(position);
                            let local_pos = utils::coords::world_to_local(position);
                            let cw = chunks_width;
                            let chunk_index = (chunk_pos.x + chunk_pos.y * cw) as usize;

                            // Use unsafe to set the particle in the shared chunk data
                            unsafe {
                                let chunks = &mut *unsafe_data_clone.chunks.get();
                                chunks[chunk_index].set_particle(local_pos, Some(common_particle));
                            }
                        }
                    }
                }
            }));
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        println!("  Parallel processing took: {:?}", start_parallel.elapsed());
        println!("Total generate_all_data time: {:?}", start_method.elapsed());

        // Return the completed chunks vector
        unsafe { (*unsafe_data.chunks.get()).clone() }
    }

    /// Spawn particles based on generated data
    #[expect(dead_code)]
    fn distribute_among_chunks(&mut self, spawn_data: Vec<Vec<(Option<Particle>, UVec2)>>) {
        let num_cpus = num_cpus::get();
        let start = std::time::Instant::now();

        // First, divide the data into chunks
        let chunk_size = (spawn_data.len() / num_cpus.max(1)).max(1);
        let data_len = spawn_data.len();

        // Collect results from each thread
        let mut thread_results = Vec::new();

        // Process data in parallel
        crossbeam::scope(|s| {
            let handles = (0..data_len)
                .step_by(chunk_size)
                .map(|chunk_start| {
                    let chunk_end = (chunk_start + chunk_size).min(data_len);
                    let chunk_slice = &spawn_data[chunk_start..chunk_end];

                    // Spawn a thread to process this chunk
                    s.spawn(move |_| {
                        let mut results = Vec::new();

                        // Process all particles in this chunk
                        for column in chunk_slice {
                            for (particle, world_pos) in column {
                                if let Some(p) = particle {
                                    // Convert world position to chunk and local coordinates
                                    let chunk_pos = utils::coords::world_to_chunk(*world_pos);
                                    let local_pos = utils::coords::world_to_local(*world_pos);

                                    results.push((chunk_pos, local_pos, *p));
                                }
                            }
                        }

                        results
                    })
                })
                .collect::<Vec<_>>();

            // Collect results from all threads
            for handle in handles {
                if let Ok(result) = handle.join() {
                    thread_results.extend(result);
                }
            }
        })
        .unwrap();

        println!("Multithreaded processing took: {:?}", start.elapsed());
        let placement_start = std::time::Instant::now();

        // Now place all particles in their respective chunks
        for (chunk_pos, local_pos, particle) in thread_results {
            if chunk_pos.x < self.width / CHUNK_SIZE && chunk_pos.y < self.height / CHUNK_SIZE {
                let chunk = &mut self.chunks[chunk_pos.x as usize][chunk_pos.y as usize];
                chunk.set_particle(local_pos, Some(particle));
            }
        }

        println!(
            "Placing particles in chunks took: {:?}",
            placement_start.elapsed()
        );
        println!("Total distribute_among_chunks time: {:?}", start.elapsed());
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
        let chunks_vec = map.generate_all_data(map_width, map_height);

        // Convert chunks vector back to our 2D vector structure
        for (i, chunk) in chunks_vec.iter().enumerate() {
            let cw = chunk_count_width(map_width);
            let x = i % cw as usize;
            let y = i / cw as usize;
            map.chunks[x][y] = chunk.clone();
        }

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

/// Generates and returns a single gem particle at the specified position
pub fn spawn_gem(position: UVec2, particle: Particle) -> Vec<(UVec2, Particle)> {
    // Simply spawn a single particle for gems
    vec![(position, particle)]
}

/// Generates and returns a vein (a small cluster of ore particles) around the specified position
pub fn spawn_vein(
    position: UVec2,
    particle: Particle,
    map_width: u32,
    map_height: u32,
) -> Vec<(UVec2, Particle)> {
    let mut rng = rand::rng();
    let mut vein_particles = vec![(position, particle)]; // Start with the central particle

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
        if new_x < 0 || new_y < 0 || new_x >= map_width as i32 || new_y >= map_height as i32 {
            continue;
        }

        let new_position = UVec2::new(new_x as u32, new_y as u32);

        // 70% chance to place an ore particle
        if rng.random_bool(0.7) {
            vein_particles.push((new_position, particle));
        }
    }

    vein_particles
}

pub fn setup_map(mut commands: Commands) {
    let map = Map::generate(125, 125);
    commands.insert_resource(map);
}

/// Updates the active chunks to be those around the player.
pub fn update_active_chunks(mut map: ResMut<Map>, player_query: Query<&Transform, With<Player>>) {
    // Use ACTIVE_CHUNK_RANGE from the chunk module for consistency
    const UPDATE_RANGE: u32 = ACTIVE_CHUNK_RANGE;

    if let Ok(player_transform) = player_query.get_single() {
        // Convert screen position to world position
        let player_pos = utils::coords::screen_to_world(
            player_transform.translation.truncate(),
            map.width,
            map.height,
        );

        // Convert player world position to chunk position
        let center_chunk = utils::coords::world_vec2_to_chunk(player_pos);

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
                update_active_chunks,
                simulate_active_particles,
                update_map_dirty_chunks,
            ),
        );
    }
}
