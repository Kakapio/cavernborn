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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionType {
    // Two particles turn into another in place.
    Combine,
}

struct InteractionRule {
    pub pair: InteractionPair,
    pub interaction_type: InteractionType,
    pub result: Option<Particle>, // The result of the interaction, if any
    pub probability: f32,         // 0.0-1.0 chance of interaction occurring
}

pub struct InteractionPair {
    pub source: Particle,
    pub target: Particle,
}

pub struct InteractionRules {
    pub rules: HashMap<InteractionPair, InteractionRule>,
}

pub struct InteractionPlugin;
