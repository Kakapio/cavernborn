use rand::Rng;

use crate::{
    chunk::CHUNK_SIZE,
    particle::{Fluid, Particle},
};

/// Checks if the given coordinates are within the bounds of a chunk
fn within_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < CHUNK_SIZE as i32 && y >= 0 && y < CHUNK_SIZE as i32
}

/// Checks if a position is valid and available in both original and new cells.
fn is_valid_cell(
    original_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    new_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    x: i32,
    y: i32,
) -> bool {
    // First check bounds to avoid invalid conversions to usize
    if !within_bounds(x, y) {
        return false;
    }

    // Convert to usize only after bounds check
    let x_usize = x as usize;
    let y_usize = y as usize;

    // Check if cell is available
    original_cells[x_usize][y_usize].is_none() && new_cells[x_usize][y_usize].is_none()
}

/// Calculates the new position for a fluid particle, reading from original_cells and writing to new_cells
pub fn simulate_fluid(
    original_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    fluid: Fluid,
    x: u32,
    y: u32,
) {
    let buoyancy = fluid.get_buoyancy();
    let viscosity = fluid.get_viscosity();

    // Move vertically down, checking if the space(s) below are available.
    // Check farthest space first, then move closer.
    for offset in (0..viscosity).rev() {
        // Lowest index we can have is 0.
        let new_y = (y as i32 + buoyancy * offset).max(0) as u32;

        // The farthest space below is available, so we can move there...
        if is_valid_cell(original_cells, new_cells, x as i32, new_y as i32) {
            new_cells[x as usize][new_y as usize] = Some(Particle::Fluid(fluid));
            return;
        }
    }

    // Diagonal movement.
    for offset in (0..viscosity).rev() {
        // Only check 1 space below for diagonal movement.
        let new_y = (y as i32 + buoyancy).max(0) as u32;
        let new_x_right = (x as i32 + offset * buoyancy).max(0) as u32;
        let new_x_left = (x as i32 - offset * buoyancy).max(0) as u32;

        // If both spaces are available, pick one randomly.
        if is_valid_cell(original_cells, new_cells, new_x_right as i32, new_y as i32)
            && is_valid_cell(original_cells, new_cells, new_x_left as i32, new_y as i32)
        {
            let mut rng = rand::rng();
            let random_direction = rng.random_range(0..2);
            if random_direction == 0 {
                new_cells[new_x_right as usize][new_y as usize] = Some(Particle::Fluid(fluid));
            } else {
                new_cells[new_x_left as usize][new_y as usize] = Some(Particle::Fluid(fluid));
            }

            return;
        }
        // Check if the right space is available.
        else if is_valid_cell(original_cells, new_cells, new_x_right as i32, new_y as i32) {
            new_cells[new_x_right as usize][new_y as usize] = Some(Particle::Fluid(fluid));
            return;
        }
        // Check if the left space is available.
        else if is_valid_cell(original_cells, new_cells, new_x_left as i32, new_y as i32) {
            new_cells[new_x_left as usize][new_y as usize] = Some(Particle::Fluid(fluid));
            return;
        }
    }

    // If we've checked all spaces and still haven't moved, move one unit.
    let new_x = (x as i32 + fluid.get_direction().as_int()).max(0) as u32;

    // Try to move in the direction of the fluid.
    if is_valid_cell(original_cells, new_cells, new_x as i32, y as i32) {
        new_cells[new_x as usize][y as usize] = Some(Particle::Fluid(fluid));
    }
    // If the space is not available, flip the direction.
    else {
        new_cells[x as usize][y as usize] = Some(Particle::Fluid(fluid.get_flipped_direction()));
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
