use crate::{
    particle::{Particle, ParticleType},
    world::chunk::{Chunk, CHUNK_SIZE},
};

pub mod fluid;

/// A struct that contains the neighbors of a chunk.
#[derive(Clone)]
pub struct NeighborChunks {
    pub top: Option<Chunk>,
    pub bottom: Option<Chunk>,
    pub left: Option<Chunk>,
    pub right: Option<Chunk>,
}

/// A trait for types that can simulate particles.
pub trait Simulator<P: ParticleType> {
    fn simulate(
        &mut self,
        neighbors: NeighborChunks,
        original_cells: &[[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        new_cells: &mut [[Option<Particle>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
        particle: P,
        x: u32,
        y: u32,
    );
}

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
