use bevy::prelude::*;
use strum_macros::EnumIter;

use super::{ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Ore {
    #[default]
    Gold,
}

impl WorldGenType for Ore {
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
}

impl ParticleType for Ore {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Ore::Gold => 4,
        }
    }
}
