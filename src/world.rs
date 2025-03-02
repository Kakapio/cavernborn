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
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            chunks: vec![vec![None; height as usize]; width as usize],
        }
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
                particle_type: particle.clone(),
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

pub fn generate_world(mut commands: Commands) {
    let world_width = 300;
    let world_height = 300;
    let mut world = World::new(world_width, world_height);
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
                let mut selected_particle = Particle::Stone; // Default to stone

                // Try to spawn particles in priority order (defined by enum variant order)
                for particle_type in Particle::iter() {
                    if depth >= particle_type.min_depth()
                        && depth <= particle_type.max_depth()
                        && rng.gen_range(0.0..1.0) < particle_type.spawn_chance()
                    {
                        selected_particle = particle_type;
                        break;
                    }
                }

                Some(selected_particle)
            };

            world.spawn_particle(&mut commands, particle_type, UVec2::new(x, y));
        }
    }

    commands.insert_resource(world);
}
