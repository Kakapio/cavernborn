use crate::particle::{Common, Particle, ParticleBundle, Special, PARTICLE_SIZE};
use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Vec<Option<Particle>>>,
}

impl Map {
    /// Create a new empty world with the given width and height.
    pub fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            chunks: vec![vec![None; height as usize]; width as usize],
        }
    }

    /// Analyze and log the composition of the world
    fn log_composition(&self) {
        let mut particle_counts: HashMap<Particle, u32> = HashMap::new();
        let mut total_particles = 0;
        let mut air_count = 0;

        // Count particles
        for row in &self.chunks {
            for cell in row {
                match cell {
                    Some(particle) => {
                        *particle_counts.entry(*particle).or_insert(0) += 1;
                        total_particles += 1;
                    }
                    None => air_count += 1,
                }
            }
        }

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
    fn roll_special_particle(depth: u32) -> Option<Particle> {
        let mut rng = rand::thread_rng();

        // Get valid special particles for this depth
        let valid_particles: Vec<_> = Special::all_variants()
            .into_iter()
            .filter(|p| depth >= p.min_depth() && depth < p.max_depth())
            .collect();

        if valid_particles.is_empty() {
            return None;
        }

        // Calculate total spawn weight
        let total_weight: i32 = valid_particles.iter().map(|p| p.spawn_chance()).sum();

        // First check: determine if we spawn any special particle
        // Illustration of the first check:
        // [0 ... total_weight ... 1000]
        //  |<---spawn--->|<---no spawn--->|
        //        ^random point
        if rng.gen_range(0..1000) >= total_weight {
            return None;
        }

        // Second check: weighted selection of which particle to spawn
        // Illustration of the second check:
        // [0 ... p1 ... p2 ... p3 ... total_weight]
        //  |<-p1->|<-p2->|<-p3->|
        //        ^random point
        let random_val = rng.gen_range(0..total_weight);

        // Use fold to perform weighted selection in a more functional way
        valid_particles
            .iter()
            .fold((0, None), |(acc_weight, selected), &special| {
                let new_weight = acc_weight + special.spawn_chance();
                if selected.is_none() && random_val < new_weight {
                    (new_weight, Some(special))
                } else {
                    (new_weight, selected)
                }
            })
            .1
            .map(Particle::Special)
    }

    /// Create a new world with terrain.
    pub fn generate(commands: &mut Commands, map_width: u32, map_height: u32) -> Self {
        let mut map = Map::empty(map_width, map_height);

        // Generate terrain
        for x in 0..map_width {
            // Basic height variation - start at 95% of height for 5% air
            let base_height = (map_height as f32 * 0.95) as u32;
            let height_variation = (x as f32 * 0.05).sin() * 10.0;
            let surface_height = base_height + height_variation as u32;

            for y in 0..map_height {
                let particle_type = if y > surface_height {
                    // Above surface is air.
                    None
                } else {
                    // Below surface
                    let depth = surface_height - y;
                    Some(
                        Self::roll_special_particle(depth)
                            .unwrap_or(Common::get_exclusive_at_depth(depth).into()),
                    )
                };

                map.spawn_particle(commands, particle_type, UVec2::new(x, y));
            }
        }

        // Log the composition of the generated world
        map.log_composition();

        map
    }

    pub fn spawn_particle(
        &mut self,
        commands: &mut Commands,
        particle_type: Option<Particle>,
        position: UVec2,
    ) {
        let x = position.x;
        let y = position.y;

        if x >= self.width || y >= self.height {
            return;
        }

        if let Some(particle) = particle_type {
            match particle {
                Particle::Special(Special::Gem(_)) => {
                    self.spawn_gem(commands, particle, position);
                }
                Particle::Special(Special::Ore(_)) => {
                    self.spawn_vein(commands, particle, position);
                }
                _ => {
                    // For common particles, just spawn directly
                    self.spawn_single_particle(commands, Some(particle), position);
                }
            }
        } else {
            // For air (None), just update the chunk data
            self.chunks[x as usize][y as usize] = None;
        }
    }

    /// Spawns a single gem particle at the specified position
    fn spawn_gem(&mut self, commands: &mut Commands, particle: Particle, position: UVec2) {
        // Simply spawn a single particle for gems
        self.spawn_single_particle(commands, Some(particle), position);
    }

    /// Spawns an ore vein (a small cluster of ore particles) around the specified position
    fn spawn_vein(&mut self, commands: &mut Commands, particle: Particle, position: UVec2) {
        let mut rng = rand::thread_rng();

        // Spawn the central ore particle
        self.spawn_single_particle(commands, Some(particle), position);

        // Determine vein size (3-6 additional particles)
        let vein_size = rng.gen_range(3..=6);

        // Try to spawn additional ore particles in adjacent positions
        for _ in 0..vein_size {
            // Random offset between -1 and 1 in both x and y directions
            let offset_x = rng.gen_range(-1..=1);
            let offset_y = rng.gen_range(-1..=1);

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

            // Only spawn if the position is not air (None)
            if self.chunks[new_x as usize][new_y as usize].is_some() {
                // 70% chance to actually place the ore
                if rng.gen_bool(0.7) {
                    self.spawn_single_particle(commands, Some(particle), new_position);
                }
            }
        }
    }

    /// Helper function to spawn a single particle at the specified position
    fn spawn_single_particle(
        &mut self,
        commands: &mut Commands,
        particle_type: Option<Particle>,
        position: UVec2,
    ) {
        let x = position.x;
        let y = position.y;

        if x >= self.width || y >= self.height {
            return;
        }

        if let Some(particle) = &particle_type {
            commands.spawn(ParticleBundle {
                particle_type: *particle,
                sprite: SpriteBundle {
                    sprite: particle.create_sprite(),
                    transform: Transform::from_xyz(
                        (x * PARTICLE_SIZE) as f32 - ((self.width * PARTICLE_SIZE) / 2) as f32,
                        (y * PARTICLE_SIZE) as f32 - ((self.height * PARTICLE_SIZE) / 2) as f32,
                        0.0,
                    ),
                    ..default()
                },
            });
        }
        self.chunks[x as usize][y as usize] = particle_type;
    }
}

pub fn setup_world(mut commands: Commands) {
    let world = Map::generate(&mut commands, 900, 300);
    commands.insert_resource(world);
}
