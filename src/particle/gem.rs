use bevy::prelude::*;
use strum_macros::EnumIter;

use super::{ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Gem {
    #[default]
    Ruby,
}

impl WorldGenType for Gem {
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
}

impl ParticleType for Gem {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Gem::Ruby => 3,
        }
    }
}
