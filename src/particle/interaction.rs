#![allow(dead_code)]
/*

The implementation below provides a framework for handling particle interactions after movement. Key points:

1. **Define interaction types and rules**: Create a data-driven approach where interactions are defined as data rather than hard-coded logic

2. **Store rules in a resource**: Use Bevy's resource system to store and retrieve applicable rules

3. **Process interactions after movement**: Make sure your interaction system runs after all movement has completed

4. **Prevent cascade effects**: Process interactions based on the state at the beginning of the frame

5. **Apply probability**: Some interactions may have a chance to occur rather than happening every time

To complete the implementation, you'll need to:
- Create a grid system to track particle positions efficiently
- Define your specific interaction rules
- Implement the spatial relationship logic (get_neighbors)
- Add the plugin to your main app

This approach gives you a flexible, maintainable system that you can easily extend with new particles and interaction types.
*/

use crate::particle::Solid;

use super::{Direction, Liquid, Particle};
use lazy_static::lazy_static;
use std::{collections::HashMap, hash::Hasher};

lazy_static! {
    pub static ref INTERACTION_RULES: HashMap<InteractionPair, InteractionRule> = {
        let mut m = HashMap::new();
        m.insert(
            InteractionPair {
                source: Particle::Liquid(Liquid::Water(Direction::Still)),
                target: Particle::Liquid(Liquid::Lava(Direction::Still)),
            },
            InteractionRule {
                interaction_type: InteractionType::Replace,
                result: Particle::Solid(Solid::Obsidian),
            },
        );

        m.insert(
            InteractionPair {
                source: Particle::Liquid(Liquid::Water(Direction::Still)),
                target: Particle::Liquid(Liquid::Acid(Direction::Still)),
            },
            InteractionRule {
                interaction_type: InteractionType::Preserve,
                result: Particle::Liquid(Liquid::Water(Direction::random())),
            },
        );

        m
    };
}

// Create a key type for interactions.
#[derive(Clone, Copy)]
pub struct InteractionPair {
    pub source: Particle,
    pub target: Particle,
}

impl std::hash::Hash for InteractionPair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash both particles individually, order doesn't matter due to XOR
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();

        self.source.hash(&mut hasher1);
        self.target.hash(&mut hasher2);

        // XOR the hashes so order doesn't matter (a XOR b = b XOR a)
        (hasher1.finish() ^ hasher2.finish()).hash(state);
    }
}

impl PartialEq for InteractionPair {
    fn eq(&self, other: &Self) -> bool {
        (self.source == other.source && self.target == other.target)
            || (self.source == other.target && self.target == other.source)
    }
}

impl Eq for InteractionPair {}

// Define the interaction types.
#[derive(Clone, Copy)]
pub enum InteractionType {
    // One particle is replaced with another (e.g., water + lava -> obsidian)
    Replace,
    // The source remains and the target is replaced (e.g., water + acid -> water)
    Preserve,
}

pub struct InteractionRule {
    pub interaction_type: InteractionType,
    pub result: Particle,
}
