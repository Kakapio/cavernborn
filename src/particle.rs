use bevy::prelude::*;
use strum_macros::EnumIter;

pub const PARTICLE_SIZE: u32 = 3;

#[derive(Component, Clone, PartialEq, EnumIter)]
pub enum Particle {
    Gold, // Most valuable first
    Dirt,
    Stone, // Fallback last
}

impl Particle {
    pub fn min_depth(&self) -> u32 {
        match self {
            Particle::Dirt => 0,
            Particle::Stone => 5,
            Particle::Gold => 23, // 5 (stone depth) + 18 units deeper
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Particle::Dirt => 5,
            Particle::Stone | Particle::Gold => u32::MAX,
        }
    }

    pub fn spawn_chance(&self) -> f32 {
        match self {
            Particle::Gold => 0.01,                  // 1% chance as used in world.rs
            Particle::Dirt | Particle::Stone => 1.0, // Always spawn if depth conditions are met
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Particle::Dirt => Color::srgb(0.6, 0.4, 0.2),
            Particle::Stone => Color::srgb(0.5, 0.5, 0.5),
            Particle::Gold => Color::srgb(1.0, 0.84, 0.0),
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
