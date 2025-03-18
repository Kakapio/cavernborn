use bevy::math::UVec2;
use rand::Rng;

use crate::{
    particle::{
        interaction::{InteractionPair, INTERACTION_RULES},
        Liquid, Particle,
    },
    utils::coords::chunk_local_to_world,
    world::chunk::ParticleMove,
};

use super::{
    handle_particle_movement, validate_move_empty, validate_move_interaction, SimulationContext,
    Simulator,
};

pub struct FluidSimulator;

impl Simulator<Liquid> for FluidSimulator {
    /// Calculates the new position for a fluid particle, reading old positions from the map and writing to new_cells.
    fn simulate(
        &mut self,
        context: SimulationContext,
        fluid: Liquid,
        x: u32,
        y: u32,
    ) -> Option<ParticleMove> {
        let particle_world_pos =
            chunk_local_to_world(context.original_chunk.position, UVec2::new(x, y));
        let (new_pos, new_fluid) =
            self.calculate_step(&context, fluid, particle_world_pos.x, particle_world_pos.y);

        // Use the shared utility function to handle the movement result.
        handle_particle_movement(
            context.original_chunk,
            context.new_cells,
            particle_world_pos,
            new_pos,
            new_fluid,
        )
    }
}

impl FluidSimulator {
    /// Calculates the new position of a fluid particle in world coordinates.
    /// Inputted x and y positions must also be in world coordinates.
    pub fn calculate_step(
        &self,
        context: &SimulationContext,
        fluid: Liquid,
        x: u32,
        y: u32,
    ) -> (UVec2, Particle) {
        let buoyancy = fluid.get_buoyancy();
        let viscosity = fluid.get_viscosity();

        // Try vertical movement first
        for offset in (0..viscosity).rev() {
            // Lowest index we can have is 0.
            let new_y = (y as i32 + buoyancy * offset).max(0) as u32;
            let new_pos = UVec2::new(x, new_y);

            if validate_move_empty(context, new_pos) {
                return (new_pos, fluid.into());
            } else if validate_move_interaction(context, new_pos, fluid.into()) {
                return (
                    new_pos,
                    INTERACTION_RULES
                        .get(&InteractionPair {
                            source: fluid.into(),
                            target: context.map.get_particle_at(new_pos).unwrap(),
                        })
                        .unwrap()
                        .result,
                );
            }
        }

        // Diagonal movement.
        for offset in (0..viscosity).rev() {
            // Only check 1 space below for diagonal movement.
            let new_y = (y as i32 + buoyancy).max(0) as u32;
            let new_x_right = (x as i32 + offset * buoyancy).max(0) as u32;
            let new_x_left = (x as i32 - offset * buoyancy).max(0) as u32;
            let new_pos_right = UVec2::new(new_x_right, new_y);
            let new_pos_left = UVec2::new(new_x_left, new_y);

            // If both spaces are available, pick one randomly.
            if validate_move_empty(context, new_pos_right)
                && validate_move_empty(context, new_pos_left)
            {
                let mut rng = rand::rng();
                let random_direction = rng.random_range(0..2);
                if random_direction == 0 {
                    return (new_pos_right, fluid.into());
                } else {
                    return (new_pos_left, fluid.into());
                }
            } else if validate_move_interaction(context, new_pos_right, fluid.into())
                && validate_move_interaction(context, new_pos_left, fluid.into())
            {
                let mut rng = rand::rng();
                let random_direction = rng.random_range(0..2);
                if random_direction == 0 {
                    return (
                        new_pos_right,
                        INTERACTION_RULES
                            .get(&InteractionPair {
                                source: fluid.into(),
                                target: context.map.get_particle_at(new_pos_right).unwrap(),
                            })
                            .unwrap()
                            .result,
                    );
                } else {
                    return (
                        new_pos_left,
                        INTERACTION_RULES
                            .get(&InteractionPair {
                                source: fluid.into(),
                                target: context.map.get_particle_at(new_pos_left).unwrap(),
                            })
                            .unwrap()
                            .result,
                    );
                }
            }
            // Check if the right space is available.
            else if validate_move_empty(context, new_pos_right) {
                return (new_pos_right, fluid.into());
            } else if validate_move_interaction(context, new_pos_right, fluid.into()) {
                return (
                    new_pos_right,
                    INTERACTION_RULES
                        .get(&InteractionPair {
                            source: fluid.into(),
                            target: context.map.get_particle_at(new_pos_right).unwrap(),
                        })
                        .unwrap()
                        .result,
                );
            }
            // Check if the left space is available.
            else if validate_move_empty(context, new_pos_left) {
                return (new_pos_left, fluid.into());
            } else if validate_move_interaction(context, new_pos_left, fluid.into()) {
                return (
                    new_pos_left,
                    INTERACTION_RULES
                        .get(&InteractionPair {
                            source: fluid.into(),
                            target: context.map.get_particle_at(new_pos_left).unwrap(),
                        })
                        .unwrap()
                        .result,
                );
            }
        }

        // If we've checked all spaces and still haven't moved, move one unit.
        let new_x = (x as i32 + fluid.get_direction().as_int()).max(0) as u32;

        // Try to move in the direction of the fluid.
        if validate_move_empty(context, UVec2::new(new_x, y)) {
            return (UVec2::new(new_x, y), fluid.into());
        } else if validate_move_interaction(context, UVec2::new(new_x, y), fluid.into()) {
            return (
                UVec2::new(new_x, y),
                INTERACTION_RULES
                    .get(&InteractionPair {
                        source: fluid.into(),
                        target: context.map.get_particle_at(UVec2::new(new_x, y)).unwrap(),
                    })
                    .unwrap()
                    .result,
            );
        }

        // If the space is not available, flip the direction.
        (UVec2::new(x, y), fluid.get_flipped_direction().into())
    }
}
