use bevy::ecs::component::Component;
use strum_macros::EnumIter;

use crate::utils::Direction;

use super::{ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Fluid {
    Water(Direction),
    Lava(Direction),
}

impl Default for Fluid {
    fn default() -> Self {
        Self::Water(Direction::default())
    }
}

impl Fluid {
    /// Describes the movement of a fluid at every step of the simulation.
    /// -1: Downward
    /// 0: None
    /// 1: Upward
    pub fn get_buoyancy(&self) -> i32 {
        match self {
            Fluid::Water(_) => -1,
            Fluid::Lava(_) => -1,
        }
    }

    /// Describes how easily a fluid flows and spreads.
    /// Higher values mean more spread.
    pub fn get_viscosity(&self) -> i32 {
        match self {
            Fluid::Water(_) => 5,
            Fluid::Lava(_) => 3,
        }
    }

    /// Returns the direction of the fluid.
    pub fn get_direction(&self) -> &Direction {
        match self {
            Fluid::Water(direction) => direction,
            Fluid::Lava(direction) => direction,
        }
    }

    /// Returns the direction of the fluid.
    pub fn get_flipped_direction(&self) -> Self {
        match self {
            Fluid::Water(direction) => Fluid::Water(direction.get_opposite()),
            Fluid::Lava(direction) => Fluid::Lava(direction.get_opposite()),
        }
    }
}

impl ParticleType for Fluid {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Fluid::Water(_) => 5,
            Fluid::Lava(_) => 6,
        }
    }
}

//TODO: Temp values.
impl WorldGenType for Fluid {
    fn min_depth(&self) -> u32 {
        match self {
            Fluid::Water(_) => 0,
            Fluid::Lava(_) => 1,
        }
    }

    fn max_depth(&self) -> u32 {
        match self {
            Fluid::Water(_) => 100,
            Fluid::Lava(_) => 100,
        }
    }

    fn spawn_chance(&self) -> i32 {
        match self {
            Fluid::Water(_) => 100,
            Fluid::Lava(_) => 100,
        }
    }
}
