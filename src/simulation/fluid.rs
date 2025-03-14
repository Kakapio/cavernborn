use bevy::math::UVec2;
use rand::Rng;

use crate::{
    particle::{Fluid, Particle},
    utils::coords::{local_to_world, world_to_local},
    world::{
        chunk::{Chunk, ParticleMove, CHUNK_SIZE},
        Map,
    },
};

use super::{is_valid_cell, Simulator};

pub struct FluidSimulator;

impl Simulator<Fluid> for FluidSimulator {
    /// Calculates the new position for a fluid particle, reading from original_cells and writing to new_cells.
    fn simulate(
        &mut self,
        map: &Map,
        original_chunk: &Chunk,
        new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        fluid: Fluid,
        x: u32,
        y: u32,
    ) -> Vec<ParticleMove> {
        let particle_world_pos = local_to_world(original_chunk.position, UVec2::new(x, y));
        let (new_pos, new_fluid) =
            self.calculate_step(map, fluid, particle_world_pos.x, particle_world_pos.y);
        let mut interchunk_queue = Vec::new();

        // If the new position is not within the chunk, we need to move the particle to the new chunk.
        if !original_chunk.is_within_chunk(new_pos) {
            interchunk_queue.push(ParticleMove {
                source_pos: particle_world_pos,
                target_pos: new_pos,
                particle: Particle::Fluid(new_fluid),
            });
        } else {
            let particle_local_pos = world_to_local(new_pos);
            new_cells[particle_local_pos.x as usize][particle_local_pos.y as usize] =
                Some(Particle::Fluid(new_fluid));
        }

        // Return the queue if we have interchunk movement, otherwise return None.
        interchunk_queue
    }
}

impl FluidSimulator {
    /// Calculates the new position of a fluid particle in world coordinates.
    pub fn calculate_step(&self, map: &Map, fluid: Fluid, x: u32, y: u32) -> (UVec2, Fluid) {
        let buoyancy = fluid.get_buoyancy();
        let viscosity = fluid.get_viscosity();

        // Try vertical movement first
        for offset in (0..viscosity).rev() {
            // Lowest index we can have is 0.
            let new_y = (y as i32 + buoyancy * offset).max(0) as u32;
            let new_pos = UVec2::new(x, new_y);

            // Return world position of vertical movement
            if map.is_valid_position(new_pos) {
                return (new_pos, fluid);
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
            if map.is_valid_position(new_pos_right) && map.is_valid_position(new_pos_left) {
                let mut rng = rand::rng();
                let random_direction = rng.random_range(0..2);
                if random_direction == 0 {
                    return (new_pos_right, fluid);
                } else {
                    return (new_pos_left, fluid);
                }
            }
            // Check if the right space is available.
            else if map.is_valid_position(new_pos_right) {
                return (new_pos_right, fluid);
            }
            // Check if the left space is available.
            else if map.is_valid_position(new_pos_left) {
                return (new_pos_left, fluid);
            }
        }

        // If we've checked all spaces and still haven't moved, move one unit.
        let new_x = (x as i32 + fluid.get_direction().as_int()).max(0) as u32;

        // Try to move in the direction of the fluid.
        if map.is_valid_position(UVec2::new(new_x, y)) {
            return (UVec2::new(new_x, y), fluid);
        }

        // If the space is not available, flip the direction.
        (UVec2::new(x, y), fluid.get_flipped_direction())
    }
}

/// Calculates the new position for a sand particle, reading from original_cells and writing to new_cells
#[expect(dead_code)]
pub fn simulate_sand(
    original_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    fluid: Fluid,
    x: u32,
    y: u32,
) {
    let buoyancy = fluid.get_buoyancy();

    // Skip if buoyancy is 0 (no movement)
    if buoyancy == 0 {
        // Keep the fluid in place
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid));
        return;
    }

    // Determine the vertical direction and check boundaries
    let new_y_i32 = y as i32 + buoyancy;
    // Use max to ensure new_y is at least 0
    let new_y = (new_y_i32.max(0)) as u32;

    let new_y = new_y as usize;

    // Try to move in one of three directions based on priority
    // Note: We check the original cells for obstacles, but write to new_cells
    if is_valid_cell(original_cells, new_cells, x as i32, new_y as i32) {
        // Move vertically
        new_cells[x as usize][new_y] = Some(Particle::Fluid(fluid));
    } else if x > 0 && is_valid_cell(original_cells, new_cells, (x - 1) as i32, new_y as i32) {
        // Move diagonally to the left
        new_cells[(x - 1) as usize][new_y] = Some(Particle::Fluid(fluid));
    } else if is_valid_cell(original_cells, new_cells, (x + 1) as i32, new_y as i32) {
        // Move diagonally to the right
        new_cells[(x + 1) as usize][new_y] = Some(Particle::Fluid(fluid));
    } else {
        // If fluid can't move in any of these directions, it stays in place
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid));
    }
}
