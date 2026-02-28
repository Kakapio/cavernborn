use crate::particle::Solid;

use super::{Direction, Liquid, Particle};
use std::{collections::HashMap, hash::Hasher, sync::LazyLock};

pub static INTERACTION_RULES: LazyLock<HashMap<InteractionPair, InteractionRule>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            InteractionPair {
                source: Particle::Liquid(Liquid::Water(Direction::Still)),
                target: Particle::Liquid(Liquid::Lava(Direction::Still)),
            },
            InteractionRule {
                result: Particle::Solid(Solid::Obsidian),
            },
        );

        m.insert(
            InteractionPair {
                source: Particle::Liquid(Liquid::Water(Direction::Still)),
                target: Particle::Liquid(Liquid::Acid(Direction::Still)),
            },
            InteractionRule {
                result: Particle::Liquid(Liquid::Water(Direction::random())),
            },
        );

        m
    });

// Create a key type for interactions.
#[derive(Clone, Copy)]
pub struct InteractionPair {
    pub source: Particle,
    pub target: Particle,
}

impl std::hash::Hash for InteractionPair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash both particles individually and sort to ensure commutativity
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();

        self.source.hash(&mut hasher1);
        self.target.hash(&mut hasher2);

        let (a, b) = (hasher1.finish(), hasher2.finish());
        let (lo, hi): (u64, u64) = if a <= b { (a, b) } else { (b, a) };
        lo.hash(state);
        hi.hash(state);
    }
}

impl PartialEq for InteractionPair {
    fn eq(&self, other: &Self) -> bool {
        (self.source == other.source && self.target == other.target)
            || (self.source == other.target && self.target == other.source)
    }
}

impl Eq for InteractionPair {}

pub struct InteractionRule {
    pub result: Particle,
}
