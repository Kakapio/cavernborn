use bevy::ecs::component::Component;
use strum_macros::EnumIter;

use super::{Direction, ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter)]
pub enum Liquid {
    Water(Direction),
    Lava(Direction),
}

impl Default for Liquid {
    fn default() -> Self {
        Self::Water(Direction::default())
    }
}

impl Liquid {
    /// Describes the movement of a fluid at every step of the simulation.
    /// -1: Downward
    /// 0: None
    /// 1: Upward
    pub fn get_buoyancy(&self) -> i32 {
        match self {
            Liquid::Water(_) => -1,
            Liquid::Lava(_) => -1,
        }
    }

    /// Describes how easily a fluid flows and spreads.
    /// Higher values mean more spread.
    pub fn get_viscosity(&self) -> i32 {
        match self {
            Liquid::Water(_) => 5,
            Liquid::Lava(_) => 3,
        }
    }

    /// Returns the direction of the fluid.
    pub fn get_direction(&self) -> &Direction {
        match self {
            Liquid::Water(direction) => direction,
            Liquid::Lava(direction) => direction,
        }
    }

    /// Returns the direction of the fluid.
    pub fn get_flipped_direction(&self) -> Self {
        match self {
            Liquid::Water(direction) => Liquid::Water(direction.get_opposite()),
            Liquid::Lava(direction) => Liquid::Lava(direction.get_opposite()),
        }
    }
}

impl ParticleType for Liquid {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Liquid::Water(_) => 5,
            Liquid::Lava(_) => 6,
        }
    }
}

//TODO: Temp values.
impl WorldGenType for Liquid {
    fn min_depth(&self) -> u32 {
        match self {
            Liquid::Water(_) => 0,
            Liquid::Lava(_) => 1,
        }
    }

    fn max_depth(&self) -> u32 {
        match self {
            Liquid::Water(_) => 100,
            Liquid::Lava(_) => 100,
        }
    }

    fn spawn_chance(&self) -> i32 {
        match self {
            Liquid::Water(_) => 100,
            Liquid::Lava(_) => 100,
        }
    }
}
