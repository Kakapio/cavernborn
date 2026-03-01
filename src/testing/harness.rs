use std::collections::BTreeMap;

use bevy::math::UVec2;

use crate::{
    particle::{Common, Direction, Liquid, Particle, Solid},
    world::{chunk::CHUNK_SIZE, Map},
};

/// Builder for setting up particle simulation test scenarios.
pub struct SimScenario {
    pub map: Map,
}

impl SimScenario {
    /// Creates a new scenario with the given dimensions in chunk units.
    /// All chunks are marked active. Most tests use `new(1, 1)` for a single 32x32 grid.
    pub fn new(chunks_x: u32, chunks_y: u32) -> Self {
        let width = chunks_x * CHUNK_SIZE;
        let height = chunks_y * CHUNK_SIZE;
        let mut map = Map::empty(width, height);

        for cx in 0..chunks_x {
            for cy in 0..chunks_y {
                map.active_chunks.insert(UVec2::new(cx, cy));
            }
        }

        Self { map }
    }

    /// Place a particle at the given world coordinates.
    pub fn place(&mut self, x: u32, y: u32, particle: Particle) -> &mut Self {
        self.map.set_particle_at(UVec2::new(x, y), Some(particle));
        self
    }

    /// Place a water particle with the given direction.
    pub fn water(&mut self, x: u32, y: u32, dir: Direction) -> &mut Self {
        self.place(x, y, Particle::Liquid(Liquid::Water(dir)))
    }

    /// Place a lava particle with the given direction.
    pub fn lava(&mut self, x: u32, y: u32, dir: Direction) -> &mut Self {
        self.place(x, y, Particle::Liquid(Liquid::Lava(dir)))
    }

    /// Place a dirt particle.
    pub fn dirt(&mut self, x: u32, y: u32) -> &mut Self {
        self.place(x, y, Particle::Common(Common::Dirt))
    }

    /// Place a row of particles from x_start to x_end (inclusive).
    pub fn place_row(&mut self, y: u32, x_start: u32, x_end: u32, particle: Particle) -> &mut Self {
        for x in x_start..=x_end {
            self.place(x, y, particle);
        }
        self
    }

    /// Place a row of dirt from x_start to x_end (inclusive).
    pub fn dirt_floor(&mut self, y: u32, x_start: u32, x_end: u32) -> &mut Self {
        self.place_row(y, x_start, x_end, Particle::Common(Common::Dirt))
    }

    /// Place a column of dirt from y_start to y_end (inclusive).
    pub fn dirt_wall(&mut self, x: u32, y_start: u32, y_end: u32) -> &mut Self {
        for y in y_start..=y_end {
            self.dirt(x, y);
        }
        self
    }

    /// Run one simulation tick: update dirty chunks then simulate active chunks.
    /// Returns a snapshot of the map state after the tick.
    pub fn tick(&mut self) -> Snapshot {
        self.map.update_dirty_chunks();
        self.map.simulate_active_chunks();
        self.snapshot()
    }

    /// Run N simulation ticks, returning a snapshot after each tick.
    pub fn run_ticks(&mut self, n: usize) -> Vec<Snapshot> {
        (0..n).map(|_| self.tick()).collect()
    }

    /// Capture the current map state as a snapshot.
    pub fn snapshot(&self) -> Snapshot {
        let mut cells = BTreeMap::new();
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                if let Some(p) = self.map.get_particle_at(UVec2::new(x, y)) {
                    cells.insert((x, y), p);
                }
            }
        }
        Snapshot {
            cells,
            width: self.map.width,
            height: self.map.height,
        }
    }
}

/// A snapshot of the map state at a point in time.
pub struct Snapshot {
    pub cells: BTreeMap<(u32, u32), Particle>,
    pub width: u32,
    pub height: u32,
}

impl Snapshot {
    /// Assert that a specific particle exists at the given position.
    pub fn assert_particle_at(&self, x: u32, y: u32, expected: Particle) {
        let actual = self.cells.get(&(x, y));
        assert_eq!(
            actual,
            Some(&expected),
            "Expected {:?} at ({}, {}), got {:?}\n{}",
            expected,
            x,
            y,
            actual,
            self.render_grid(x, y, 8)
        );
    }

    /// Assert multiple particle positions at once.
    pub fn assert_particles_at(&self, expected: &[(u32, u32, Particle)]) {
        for &(x, y, particle) in expected {
            self.assert_particle_at(x, y, particle);
        }
    }

    /// Assert that the given positions are empty.
    pub fn assert_empty_at(&self, positions: &[(u32, u32)]) {
        for &(x, y) in positions {
            assert!(
                !self.cells.contains_key(&(x, y)),
                "Expected empty at ({}, {}), got {:?}\n{}",
                x,
                y,
                self.cells.get(&(x, y)),
                self.render_grid(x, y, 8)
            );
        }
    }

    /// Assert that two snapshots are identical, showing a diff on failure.
    pub fn assert_eq(&self, other: &Snapshot) {
        if self.cells == other.cells {
            return;
        }
        let mut diff = String::new();
        let all_keys: std::collections::BTreeSet<_> =
            self.cells.keys().chain(other.cells.keys()).collect();
        for key in all_keys {
            let a = self.cells.get(key);
            let b = other.cells.get(key);
            if a != b {
                diff.push_str(&format!(
                    "  ({}, {}): actual={:?}, expected={:?}\n",
                    key.0, key.1, a, b
                ));
            }
        }
        panic!(
            "Snapshots differ:\n{}\nActual:\n{}\nExpected:\n{}",
            diff,
            self.render(),
            other.render()
        );
    }

    /// Assert the direction of a liquid particle at the given position.
    /// Needed because `Liquid`'s `PartialEq` ignores direction.
    pub fn assert_liquid_direction(&self, x: u32, y: u32, expected_dir: Direction) {
        match self.cells.get(&(x, y)) {
            Some(Particle::Liquid(liquid)) => {
                assert_eq!(
                    *liquid.get_direction(),
                    expected_dir,
                    "Expected direction {:?} at ({}, {}), got {:?}\n{}",
                    expected_dir,
                    x,
                    y,
                    liquid.get_direction(),
                    self.render_grid(x, y, 8)
                );
            }
            other => {
                panic!(
                    "Expected liquid at ({}, {}), got {:?}\n{}",
                    x,
                    y,
                    other,
                    self.render_grid(x, y, 8)
                );
            }
        }
    }

    /// Render the entire map as ASCII art.
    pub fn render(&self) -> String {
        self.render_grid(self.width / 2, self.height / 2, self.width.max(self.height))
    }

    /// Render a region of the map as ASCII art, centered on (cx, cy) with the given radius.
    /// y=0 is at the bottom (matching the physics coordinate system).
    pub fn render_grid(&self, cx: u32, cy: u32, radius: u32) -> String {
        let x_min = cx.saturating_sub(radius);
        let x_max = (cx + radius).min(self.width.saturating_sub(1));
        let y_min = cy.saturating_sub(radius);
        let y_max = (cy + radius).min(self.height.saturating_sub(1));

        let mut result = String::new();

        // Header row with x coordinates
        result.push_str("    ");
        for x in x_min..=x_max {
            result.push_str(&format!("{:>3}", x));
        }
        result.push('\n');

        // Render from top to bottom (highest y first, since y=0 is the bottom)
        for y in (y_min..=y_max).rev() {
            result.push_str(&format!("{:>3} ", y));
            for x in x_min..=x_max {
                let ch = match self.cells.get(&(x, y)) {
                    None => " . ",
                    Some(Particle::Common(Common::Dirt)) => " D ",
                    Some(Particle::Common(Common::Stone)) => " S ",
                    Some(Particle::Liquid(Liquid::Water(d))) => match d {
                        Direction::Left => " w<",
                        Direction::Right => " w>",
                        Direction::Still => " w~",
                    },
                    Some(Particle::Liquid(Liquid::Lava(d))) => match d {
                        Direction::Left => " L<",
                        Direction::Right => " L>",
                        Direction::Still => " L~",
                    },
                    Some(Particle::Liquid(Liquid::Acid(d))) => match d {
                        Direction::Left => " A<",
                        Direction::Right => " A>",
                        Direction::Still => " A~",
                    },
                    Some(Particle::Solid(Solid::Obsidian)) => " O ",
                    Some(Particle::Special(_)) => " * ",
                };
                result.push_str(ch);
            }
            result.push('\n');
        }
        result
    }
}
