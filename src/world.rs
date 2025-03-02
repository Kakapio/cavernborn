use crate::particle::{Common, Particle, ParticleBundle, Special, PARTICLE_SIZE};
use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Resource)]
pub struct World {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Vec<Option<Particle>>>,
}

impl World {
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
        info!("\nWorld Composition Analysis:");
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
    fn spawn_special_particle(depth: u32) -> Option<Particle> {
        let mut rng = rand::thread_rng();
        // Collect all valid special particles for this depth.
        let valid_particles: Vec<_> = Special::iter()
            .filter(|p| depth >= p.min_depth() && depth < p.max_depth())
            .collect();

        // Calculate total spawn weight out of 1000
        let total_weight = valid_particles.iter().map(|p| p.spawn_chance()).sum();

        // First roll: Determine if we spawn any special particle at all (out of 1000)
        let spawn_special = rng.gen_range(0..1000) < total_weight;

        if spawn_special {
            // Second roll: Determine which special particle to spawn
            let random_val = rng.gen_range(0..total_weight);
            let mut cumulative_weight = 0;

            valid_particles
                .iter()
                .find(|&&p| {
                    let range_start = cumulative_weight;
                    let range_end = range_start + p.spawn_chance();
                    cumulative_weight = range_end;
                    random_val >= range_start && random_val < range_end
                })
                .copied()
                .map(Particle::Special)
        } else {
            None
        }
    }

    /// Create a new world with terrain.
    pub fn generate(commands: &mut Commands, world_width: u32, world_height: u32) -> Self {
        let mut world = World::empty(world_width, world_height);

        // Generate terrain
        for x in 0..world_width {
            // Basic height variation - start at 95% of height for 5% air
            let base_height = (world_height as f32 * 0.95) as u32;
            let height_variation = (x as f32 * 0.05).sin() * 10.0;
            let surface_height = base_height + height_variation as u32;

            for y in 0..world_height {
                let particle_type = if y > surface_height {
                    // Above surface is air.
                    None
                } else {
                    // Below surface
                    let depth = surface_height - y;
                    Some(
                        Self::spawn_special_particle(depth)
                            .unwrap_or(Particle::Common(Common::get_exclusive_at_depth(depth))),
                    )
                };

                world.spawn_particle(commands, particle_type, UVec2::new(x, y));
            }
        }

        // Log the composition of the generated world
        world.log_composition();

        world
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
    let world = World::generate(&mut commands, 300, 300);
    commands.insert_resource(world);
}
