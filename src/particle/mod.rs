use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// Declare the submodules
mod fluid;
mod gem;
mod ore;

// Import from submodules
pub use self::fluid::Fluid;
pub use self::gem::Gem;
pub use self::ore::Ore;

/// The square size of the particle in pixels.
/// This is used in all logic that utilizes particles.
pub(crate) const PARTICLE_SIZE: u32 = 3;

/// Represents 100% but in terms of discrete values. Ex: If this is 1000, then 5 is 0.5%.
const SPAWN_CHANCE_SCALE: i32 = 1000;

/// Define a trait for types that can be used for world generation.
pub trait WorldGenType: ParticleType {
    fn min_depth(&self) -> u32;
    fn max_depth(&self) -> u32;
    fn spawn_chance(&self) -> i32;
}

/// Trait for all particles.
pub trait ParticleType: Copy + IntoEnumIterator {
    fn get_spritesheet_index(&self) -> u32;
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Particle {
    Common(Common),
    Special(Special),
    Fluid(Fluid),
}

impl Default for Particle {
    fn default() -> Self {
        Self::Common(Common::default())
    }
}

impl ParticleType for Particle {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Particle::Common(common) => common.get_spritesheet_index(),
            Particle::Special(special) => special.get_spritesheet_index(),
            Particle::Fluid(fluid) => fluid.get_spritesheet_index(),
        }
    }
}
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Common {
    #[default]
    Dirt,
    Stone,
}

impl ParticleType for Common {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Common::Dirt => 1,
            Common::Stone => 2,
        }
    }
}

impl ParticleType for Special {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Special::Ore(ore) => ore.get_spritesheet_index(),
            Special::Gem(gem) => gem.get_spritesheet_index(),
        }
    }
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

    /// Returns the appropriate common particle for a given depth, if the depth falls within an exclusive range.
    /// Uses half-open intervals [min, max) where min is inclusive and max is exclusive.
    /// Panics if no variant's range contains the depth or if multiple variants' ranges contain the depth.
    pub fn get_exclusive_at_depth(depth: u32) -> Common {
        // Find all variants whose range contains the given depth
        let mut matching_variants = Vec::new();

        for variant in Common::iter() {
            if depth >= variant.min_depth() && depth < variant.max_depth() {
                matching_variants.push(variant);
            }
        }

        // Check if we found exactly one matching variant
        match matching_variants.len() {
            0 => panic!("Cannot get common particle at depth {}.", depth),
            1 => matching_variants[0],
            _ => panic!(
                "Multiple common particles valid at depth {}. This indicates overlapping ranges.",
                depth
            ),
        }
    }
}

#[allow(dead_code)]
impl Particle {
    pub fn min_depth(&self) -> u32 {
        match self {
            Particle::Common(common) => common.min_depth(),
            Particle::Special(special) => special.min_depth(),
            Particle::Fluid(fluid) => fluid.min_depth(),
        }
    }

    pub fn max_depth(&self) -> u32 {
        match self {
            Particle::Common(common) => common.max_depth(),
            Particle::Special(special) => special.max_depth(),
            Particle::Fluid(fluid) => fluid.max_depth(),
        }
    }

    pub fn spawn_chance(&self) -> i32 {
        match self {
            Particle::Common(_) => SPAWN_CHANCE_SCALE,
            Particle::Special(special) => special.spawn_chance(),
            Particle::Fluid(fluid) => fluid.spawn_chance(),
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

impl From<Common> for Particle {
    fn from(common: Common) -> Self {
        Particle::Common(common)
    }
}
