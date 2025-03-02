use bevy::prelude::*;
use strum_macros::EnumIter;

pub const PARTICLE_SIZE: u32 = 3;

/// Represents 100% but in terms of discrete values. Ex: If this is 1000, then 5 is 0.5%.
pub const SPAWN_CHANCE_SCALE: i32 = 1000;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Particle {
    Common(Common),
    Special(Special),
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]

pub enum Common {
    #[default]
    Dirt,
    Stone,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]

pub enum Special {
    Gold,
    #[default]
    Ruby,
}

impl Common {
    pub fn min_depth(&self) -> u32 {
        match self {
            Common::Dirt => 0,
            Common::Stone => 5,
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Common::Dirt => 5,
            Common::Stone => u32::MAX,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Common::Dirt => Color::srgb(0.6, 0.4, 0.2),
            Common::Stone => Color::srgb(0.5, 0.5, 0.5),
        }
    }
}

impl Special {
    pub fn min_depth(&self) -> u32 {
        match self {
            Special::Gold | Special::Ruby => 23,
        }
    }
    pub fn max_depth(&self) -> u32 {
        match self {
            Special::Gold | Special::Ruby => u32::MAX,
        }
    }

    pub fn spawn_chance(&self) -> i32 {
        match self {
            Special::Gold => 20,
            Special::Ruby => 3,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Special::Gold => Color::srgb(1.0, 0.84, 0.0),
            Special::Ruby => Color::srgb(0.9, 0.1, 0.1),
        }
    }
}

#[allow(dead_code)]
impl Particle {
    pub fn min_depth(&self) -> u32 {
        match self {
            Particle::Common(common) => common.min_depth(),
            Particle::Special(special) => special.min_depth(),
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Particle::Common(common) => common.max_depth(),
            Particle::Special(special) => special.max_depth(),
        }
    }

    pub fn spawn_chance(&self) -> i32 {
        match self {
            Particle::Common(_) => SPAWN_CHANCE_SCALE,
            Particle::Special(special) => special.spawn_chance(),
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Particle::Common(common) => common.get_color(),
            Particle::Special(special) => special.get_color(),
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
