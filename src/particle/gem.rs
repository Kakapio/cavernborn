use bevy::prelude::*;
use strum_macros::EnumIter;

use super::SpecialType;

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
