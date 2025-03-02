use crate::particle::{Particle, ParticleBundle, PARTICLE_SIZE};
use bevy::prelude::*;
use rand::prelude::*;
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

    /// Create a new world with terrain.
    pub fn generate(commands: &mut Commands, world_width: u32, world_height: u32) -> Self {
        let mut world = World::empty(world_width, world_height);
        let mut rng = rand::thread_rng();

        // Generate terrain
        for x in 0..world_width {
            // Basic height variation
            let base_height = (world_height as f32 * 0.3) as u32;
            let height_variation = (x as f32 * 0.05).sin() * 10.0;
            let surface_height = base_height + height_variation as u32;

            for y in 0..world_height {
                let particle_type = if y > surface_height {
                    // Above surface is air.
                    None
                } else {
                    // Below surface
                    let depth = surface_height - y;

                    // Collect all valid particles for this depth with their spawn chances
                    let valid_particles: Vec<_> = Particle::iter()
                        .filter(|p| depth >= p.min_depth() && depth <= p.max_depth())
                        .collect();

                    // Calculate total spawn weight
                    let total_weight: f32 = valid_particles.iter().map(|p| p.spawn_chance()).sum();

                    // Pick a random value in the total weight range
                    let mut random_val = rng.gen_range(0.0..total_weight);

                    // Default to stone if no special particle is selected
                    let selected_particle = valid_particles
                        .iter()
                        .find(|&&p| {
                            random_val -= p.spawn_chance();
                            random_val <= 0.0
                        })
                        .copied()
                        .unwrap_or(Particle::Stone);

                    Some(selected_particle)
                };

                world.spawn_particle(commands, particle_type, UVec2::new(x, y));
            }
        }

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
