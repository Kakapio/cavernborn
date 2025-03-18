use bevy::math::UVec2;

use crate::{
    particle::{Liquid, Particle},
    utils::coords::chunk_local_to_world,
    world::chunk::ParticleMove,
};

use super::{handle_particle_movement, try_move, SimulationContext, Simulator};

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
    /// It will either move to a new position, or interact with a neighboring particle if possible.
    pub fn calculate_step(
        &self,
        context: &SimulationContext,
        fluid: Liquid,
        x: u32,
        y: u32,
    ) -> (UVec2, Particle) {
        let particle = fluid.into();
        let buoyancy = fluid.get_buoyancy();
        let viscosity = fluid.get_viscosity();

        // Try vertical movement first
        for offset in (0..viscosity).rev() {
            let new_pos = UVec2::new(x, (y as i32 + buoyancy * offset).max(0) as u32);
            if let Some(result) = try_move(context, new_pos, particle) {
                return result;
            }
        }

        // Try diagonal movement
        for offset in (0..viscosity).rev() {
            let new_y = (y as i32 + buoyancy).max(0) as u32;
            let new_x_right = (x as i32 + offset * buoyancy).max(0) as u32;
            let new_x_left = (x as i32 - offset * buoyancy).max(0) as u32;

            let move_right = try_move(context, UVec2::new(new_x_right, new_y), particle);
            let move_left = try_move(context, UVec2::new(new_x_left, new_y), particle);

            match (move_right, move_left) {
                // If both are possible, choose one randomly.
                (Some(right), Some(left)) => return if rand::random() { right } else { left },
                // If one is possible, return that.
                (Some(result), None) | (None, Some(result)) => return result,
                // If neither are possible, do nothing.
                (None, None) => {}
            }
        }

        // Try moving horizontally
        let new_x = (x as i32 + fluid.get_direction().as_int()).max(0) as u32;
        if let Some(result) = try_move(context, UVec2::new(new_x, y), particle) {
            return result;
        }

        // If no movement is possible, flip direction
        (UVec2::new(x, y), fluid.get_flipped_direction().into())
    }
}
