use bevy::ecs::component::Component;
use strum_macros::EnumIter;

use super::{ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Fluid {
    #[default]
    Water,
    Lava,
}

impl Fluid {
    /// Describes the movement of a fluid at every step of the simulation.
    /// -X: Downward
    /// 0: None
    /// X: Upward
    pub fn get_buoyancy(&self) -> i32 {
        match self {
            Fluid::Water => -1,
            Fluid::Lava => -1,
        }
    }
}

impl ParticleType for Fluid {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Fluid::Water => 5,
            Fluid::Lava => 6,
        }
    }
}

//TODO: Temp values.
impl WorldGenType for Fluid {
    fn min_depth(&self) -> u32 {
        match self {
            Fluid::Water => 0,
            Fluid::Lava => 1,
        }
    }

    fn max_depth(&self) -> u32 {
        match self {
            Fluid::Water => 100,
            Fluid::Lava => 100,
        }
    }

    fn spawn_chance(&self) -> i32 {
        match self {
            Fluid::Water => 100,
            Fluid::Lava => 100,
        }
    }
}
