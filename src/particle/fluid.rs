use bevy::ecs::component::Component;
use strum_macros::EnumIter;

use super::{ParticleType, WorldGenType};

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, Default)]
pub enum Fluid {
    #[default]
    Water,
    Lava,
}

impl ParticleType for Fluid {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Fluid::Water => 0,
            Fluid::Lava => 1,
        }
    }
}

impl WorldGenType for Fluid {
    fn min_depth(&self) -> u32 {
        todo!()
    }

    fn max_depth(&self) -> u32 {
        todo!()
    }

    fn spawn_chance(&self) -> i32 {
        todo!()
    }
}
