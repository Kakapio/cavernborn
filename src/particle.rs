use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// The square size of the particle in pixels.
pub const PARTICLE_SIZE: u32 = 3;

/// Represents 100% but in terms of discrete values. Ex: If this is 1000, then 5 is 0.5%.
pub const SPAWN_CHANCE_SCALE: i32 = 1000;

// Define a trait for special particle types
pub trait SpecialType: Copy + IntoEnumIterator {
    fn min_depth(&self) -> u32;
    fn max_depth(&self) -> u32;
    fn spawn_chance(&self) -> i32;
    fn get_color(&self) -> Color;
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Ore {
    #[default]
    Gold,
}

impl SpecialType for Ore {
    fn min_depth(&self) -> u32 {
        match self {
            Ore::Gold => 23,
        }
    }

    fn max_depth(&self) -> u32 {
        match self {
            Ore::Gold => u32::MAX,
        }
    }

    fn spawn_chance(&self) -> i32 {
        match self {
            Ore::Gold => 20,
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Ore::Gold => Color::srgb(1.0, 0.84, 0.0),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Gem {
    #[default]
    Ruby,
}

impl SpecialType for Gem {
    fn min_depth(&self) -> u32 {
        match self {
            Gem::Ruby => 80,
        }
    }

    fn max_depth(&self) -> u32 {
        match self {
            Gem::Ruby => 150,
        }
    }

    fn spawn_chance(&self) -> i32 {
        match self {
            Gem::Ruby => 3,
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Gem::Ruby => Color::srgb(0.9, 0.1, 0.1),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Special {
    Ore(Ore),
    Gem(Gem),
}

impl Default for Special {
    fn default() -> Self {
        Self::Ore(Ore::default())
    }
}

impl Special {
    pub fn min_depth(&self) -> u32 {
        match self {
            Special::Ore(ore) => ore.min_depth(),
            Special::Gem(gem) => gem.min_depth(),
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Special::Ore(ore) => ore.max_depth(),
            Special::Gem(gem) => gem.max_depth(),
        }
    }

    pub fn spawn_chance(&self) -> i32 {
        match self {
            Special::Ore(ore) => ore.spawn_chance(),
            Special::Gem(gem) => gem.spawn_chance(),
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Special::Ore(ore) => ore.get_color(),
            Special::Gem(gem) => gem.get_color(),
        }
    }

    // Helper function to get all possible special particles
    pub fn all_variants() -> Vec<Special> {
        let mut variants = Vec::new();

        // Add all ore variants
        for ore in Ore::iter() {
            variants.push(Special::Ore(ore));
        }

        // Add all gem variants
        for gem in Gem::iter() {
            variants.push(Special::Gem(gem));
        }

        variants
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Particle {
    Common(Common),
    Special(Special),
}

impl Default for Particle {
    fn default() -> Self {
        Self::Common(Common::default())
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Common {
    #[default]
    Dirt,
    Stone,
}

impl Common {
    pub fn min_depth(&self) -> u32 {
        match self {
            Common::Dirt => 0,
            Common::Stone => 12,
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Common::Dirt => 12,
            Common::Stone => u32::MAX,
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Common::Dirt => Color::srgb(0.6, 0.4, 0.2),
            Common::Stone => Color::srgb(0.5, 0.5, 0.5),
        }
    }

    /// Returns the appropriate common particle for a given depth, if the depth falls within an exclusive range.
    /// Uses half-open intervals [min, max) where min is inclusive and max is exclusive.
    pub fn get_exclusive_at_depth(depth: u32) -> Common {
        // Iterate through all Common variants and find the one whose range contains the given depth
        for variant in Common::iter() {
            if depth >= variant.min_depth() && depth < variant.max_depth() {
                return variant;
            }
        }

        // If no variant's range contains the depth, panic
        panic!("Cannot get common particle at depth {}.", depth);
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
