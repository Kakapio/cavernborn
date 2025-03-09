use crate::{
    particle::{Common, Particle, Special},
    utils::coords::{world_to_chunk, world_to_local},
    world::chunk::Chunk,
    world::map::{chunk_count_height, chunk_count_width},
};
use bevy::{
    ecs::system::{Commands, ResMut},
    log::info_span,
    math::UVec2,
};
use rand::Rng;
use std::{cell::UnsafeCell, sync::Arc};

use super::Map;

pub(crate) struct UnsafeChunkData {
    pub chunks: UnsafeCell<Vec<Chunk>>,
}

unsafe impl Sync for UnsafeChunkData {}

/// Generate terrain data for the entire map.
pub(crate) fn generate_all_data(map_width: u32, map_height: u32) -> Vec<Chunk> {
    let _ = info_span!("generate_map_data_all").entered();
    let start_method = std::time::Instant::now();

    // Pre-compute all surface heights
    let surface_heights = calculate_surface_heights(map_width, map_height);

    // Calculate chunk counts
    let chunks_width = chunk_count_width(map_width);
    let chunks_height = chunk_count_height(map_height);

    // Create empty chunks
    let chunks = create_empty_chunks(chunks_width, chunks_height);

    // Create unsafe wrapper to allow parallel writing
    let unsafe_data = Arc::new(UnsafeChunkData {
        chunks: UnsafeCell::new(chunks),
    });

    // Determine number of threads to use
    let num_cpus = num_cpus::get();
    // Used to calculate the number of columns to process per thread.
    let work_unit = (map_width as usize / num_cpus).max(1);

    // Process columns in parallel
    let start_parallel = std::time::Instant::now();
    let mut handles = Vec::new();

    for thread_id in 0..num_cpus {
        let unsafe_data_clone = Arc::clone(&unsafe_data);
        let surface_heights_clone = surface_heights.clone();

        let start_x = thread_id * work_unit;
        let end_x = if thread_id == num_cpus - 1 {
            map_width as usize
        } else {
            (thread_id + 1) * work_unit
        };

        handles.push(std::thread::spawn(move || {
            process_columns_range(
                start_x,
                end_x,
                &surface_heights_clone,
                map_width,
                map_height,
                unsafe_data_clone,
                chunks_width,
            );
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

/// Process a range of columns in the map
fn process_columns_range(
    start_x: usize,
    end_x: usize,
    surface_heights: &[u32],
    map_width: u32,
    map_height: u32,
    unsafe_data: Arc<UnsafeChunkData>,
    chunks_width: u32,
) {
    let _ = info_span!(
        "generate_map_data_thread",
        width_range = format!("{}..{}", start_x, end_x)
    )
    .entered();
    let mut rng = rand::rng();

    for (x, _) in surface_heights
        .iter()
        .enumerate()
        .skip(start_x)
        .take(end_x - start_x)
    {
        let surface_height = surface_heights[x];

        for y in 0..map_height as usize {
            let position = UVec2::new(x as u32, y as u32);
            let special_particle = if y as u32 > surface_height {
                None
            } else {
                let depth = surface_height - y as u32;
                Map::roll_special_particle(depth, &mut rng)
            };

            if let Some(Particle::Special(special)) = special_particle {
                process_special_particle(
                    position,
                    special,
                    map_width,
                    map_height,
                    &unsafe_data,
                    chunks_width,
                );
            } else if y as u32 <= surface_height {
                // If no special particle was rolled, use common particle
                let depth = surface_height - y as u32;
                process_common_particle(position, depth, &unsafe_data, chunks_width);
            }
        }
    }
}

/// Helper function to convert world position to chunk index
fn world_to_chunk_index(position: UVec2, chunks_width: u32) -> (UVec2, usize) {
    let chunk_pos = world_to_chunk(position);
    let local_pos = world_to_local(position);
    let chunk_index = (chunk_pos.x + chunk_pos.y * chunks_width) as usize;
    (local_pos, chunk_index)
}

/// Process special particles (ores and gems) and place them in the world.
/// Note: Special particles are allowed to overwrite common particles.
fn process_special_particle(
    position: UVec2,
    special: Special,
    map_width: u32,
    map_height: u32,
    unsafe_data: &Arc<UnsafeChunkData>,
    chunks_width: u32,
) {
    let particles = match special {
        Special::Ore(_) => spawn_vein(position, Particle::Special(special), map_width, map_height),
        Special::Gem(_) => spawn_gem(position, Particle::Special(special)),
    };

    // Place all the spawned particles
    for (spawn_pos, particle) in particles {
        let (local_pos, chunk_index) = world_to_chunk_index(spawn_pos, chunks_width);

        // Use unsafe to set the particle in the shared chunk data
        unsafe {
            let chunks = &mut *unsafe_data.chunks.get();
            chunks[chunk_index].set_particle(local_pos, Some(particle));
        }
    }
}

/// Process common particles and place them in the world.
/// Note: Common particles are not allowed to overwrite special particles.
fn process_common_particle(
    position: UVec2,
    depth: u32,
    unsafe_data: &Arc<UnsafeChunkData>,
    chunks_width: u32,
) {
    // Get common particle based on depth
    let common_particle = Common::get_exclusive_at_depth(depth).into();

    // Convert world position to chunk and local coordinates
    let (local_pos, chunk_index) = world_to_chunk_index(position, chunks_width);

    // Use unsafe to set the particle in the shared chunk data
    unsafe {
        let chunks = &mut *unsafe_data.chunks.get();

        if chunks[chunk_index].get_particle(local_pos).is_none() {
            chunks[chunk_index].set_particle(local_pos, Some(common_particle));
        }
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
    let map = Map::generate(20, 20);
    commands.insert_resource(map);
}

/// System that updates all dirty chunks in the active set
pub fn update_map_dirty_chunks(mut map: ResMut<Map>) {
    map.update_dirty_chunks();
}

/// System that simulates active particles in chunks
pub fn simulate_active_particles(mut map: ResMut<Map>) {
    map.update_active_chunks();
}

/// Create and initialize empty chunks
fn create_empty_chunks(chunks_width: u32, chunks_height: u32) -> Vec<Chunk> {
    let mut chunks = Vec::with_capacity(chunks_width as usize * chunks_height as usize);
    for x in 0..chunks_width {
        for y in 0..chunks_height {
            chunks.push(Chunk::new(UVec2::new(x, y)));
        }
    }
    chunks
}

/// Calculate surface heights for terrain generation
fn calculate_surface_heights(map_width: u32, map_height: u32) -> Vec<u32> {
    let _ = info_span!("calculate_surface_heights").entered();
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

    surface_heights
}
