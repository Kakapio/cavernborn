use bevy::prelude::*;
use strum_macros::EnumIter;

pub const PARTICLE_SIZE: u32 = 3;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Particle {
    Gold,
    Ruby,
    Dirt,
    Stone,
}

impl Particle {
    pub fn min_depth(&self) -> u32 {
        match self {
            Particle::Dirt => 0,
            Particle::Stone => 5,
            Particle::Gold | Particle::Ruby => 23, // Both precious minerals start at same depth
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Particle::Dirt => 5,
            Particle::Stone | Particle::Gold | Particle::Ruby => u32::MAX,
        }
    }

    pub fn spawn_chance(&self) -> f32 {
        match self {
            Particle::Gold => 0.01,
            Particle::Ruby => 0.008, // Slightly rarer than gold
            Particle::Dirt | Particle::Stone => 1.0,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Particle::Dirt => Color::srgb(0.6, 0.4, 0.2),
            Particle::Stone => Color::srgb(0.5, 0.5, 0.5),
            Particle::Gold => Color::srgb(1.0, 0.84, 0.0),
            Particle::Ruby => Color::srgb(0.9, 0.1, 0.1),
        }
    }

    pub fn create_sprite(&self) -> Sprite {
        Sprite {
            color: self.get_color(),
            custom_size: Some(Vec2::new(PARTICLE_SIZE as f32, PARTICLE_SIZE as f32)),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct ParticleBundle {
    pub particle_type: Particle,
    pub sprite: SpriteBundle,
}
