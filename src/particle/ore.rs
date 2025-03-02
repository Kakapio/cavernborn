use bevy::prelude::*;
use strum_macros::EnumIter;

use super::SpecialType;

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
