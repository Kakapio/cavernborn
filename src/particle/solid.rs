use strum_macros::EnumIter;

use super::ParticleType;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, EnumIter)]
pub enum Solid {
    #[default]
    Obsidian,
}

impl ParticleType for Solid {
    fn get_spritesheet_index(&self) -> u32 {
        match self {
            Solid::Obsidian => 7,
        }
    }
}
