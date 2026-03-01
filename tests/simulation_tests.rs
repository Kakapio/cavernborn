use cavernborn::particle::{Common, Direction, Liquid, Particle, Solid};
use cavernborn::testing::harness::SimScenario;

/// Water (viscosity=5) falls 4 cells in one tick in open air.
/// The fluid sim tries offsets 4,3,2,1,0 — offset 4 succeeds first.
#[test]
fn water_falls_max_distance() {
    let mut s = SimScenario::new(1, 1);
    s.water(5, 20, Direction::Left);

    let snap = s.tick();

    snap.assert_particle_at(5, 16, Particle::Liquid(Liquid::Water(Direction::Left)));
    snap.assert_empty_at(&[(5, 20)]);
}

/// Lava (viscosity=3) falls 2 cells in one tick — slower than water.
/// Offsets tried: 2,1,0 — offset 2 succeeds first.
#[test]
fn lava_falls_slower_than_water() {
    let mut s = SimScenario::new(1, 1);
    s.lava(5, 20, Direction::Left);

    let snap = s.tick();

    snap.assert_particle_at(5, 18, Particle::Liquid(Liquid::Lava(Direction::Left)));
    snap.assert_empty_at(&[(5, 20)]);
}

/// Solid particles (dirt) do not move during simulation.
#[test]
fn dirt_does_not_move() {
    let mut s = SimScenario::new(1, 1);
    s.dirt(5, 5);

    let snap = s.tick();

    snap.assert_particle_at(5, 5, Particle::Common(Common::Dirt));
}

/// When vertical movement is fully blocked, water moves diagonally.
/// The map edge at x=0 blocks one diagonal direction, making the result deterministic.
///
/// Setup: water at (0, 5), dirt column at x=0 from y=1 to y=4.
/// Vertical: all offsets hit dirt. Diagonal: "right" (x-offset) clamps to x=0 (dirt),
/// "left" (x+offset=4) is open at y=4. Water moves to (4, 4).
#[test]
fn water_blocked_below_goes_diagonal() {
    let mut s = SimScenario::new(1, 1);
    s.water(0, 5, Direction::Left);
    // Block all vertical movement with a dirt column
    for y in 1..=4 {
        s.dirt(0, y);
    }

    let snap = s.tick();

    snap.assert_particle_at(4, 4, Particle::Liquid(Liquid::Water(Direction::Left)));
    snap.assert_empty_at(&[(0, 5)]);
}

/// When vertical and diagonal movement are blocked, water spreads horizontally
/// in its current direction.
///
/// Setup: water at (15, 1) moving Left, full dirt floor at y=0.
/// All vertical and diagonal targets land on the floor. Horizontal: (14, 1) is open.
#[test]
fn water_spreads_horizontally_on_floor() {
    let mut s = SimScenario::new(1, 1);
    s.water(15, 1, Direction::Left);
    s.dirt_floor(0, 0, 31);

    let snap = s.tick();

    snap.assert_particle_at(14, 1, Particle::Liquid(Liquid::Water(Direction::Left)));
    snap.assert_empty_at(&[(15, 1)]);
}

/// When all movement is blocked (including horizontal), water flips its direction.
///
/// Setup: water at (0, 1) moving Left, full dirt floor at y=0.
/// Vertical, diagonal, and horizontal (x=-1 clamps to 0, current position) all blocked.
/// Water stays in place but direction flips to Right.
#[test]
fn water_flips_direction_at_wall() {
    let mut s = SimScenario::new(1, 1);
    s.water(0, 1, Direction::Left);
    s.dirt_floor(0, 0, 31);

    let snap = s.tick();

    // Water stays at same position
    snap.assert_particle_at(0, 1, Particle::Liquid(Liquid::Water(Direction::Right)));
    // Direction flipped from Left to Right
    snap.assert_liquid_direction(0, 1, Direction::Right);
}

/// Water falls 4 cells per tick over multiple ticks.
/// Tracks exact position at each of 5 ticks.
#[test]
fn water_falls_multiple_ticks() {
    let mut s = SimScenario::new(1, 1);
    s.water(5, 25, Direction::Left);

    let snaps = s.run_ticks(5);

    let expected_y = [21, 17, 13, 9, 5];
    for (i, snap) in snaps.iter().enumerate() {
        snap.assert_particle_at(
            5,
            expected_y[i],
            Particle::Liquid(Liquid::Water(Direction::Left)),
        );
    }
}

/// Water + Lava interaction produces Obsidian (Replace interaction).
/// Both water and lava are consumed; obsidian appears at lava's position.
///
/// Setup: water at (5, 5), lava at (5, 1) boxed in by dirt walls at (4,1) and (6,1),
/// dirt floor at y=0. Lava can't move, so when water falls to (5, 1) the interaction fires.
#[test]
fn water_lava_produces_obsidian() {
    let mut s = SimScenario::new(1, 1);
    s.water(5, 5, Direction::Left);
    s.lava(5, 1, Direction::Left);
    s.dirt_floor(0, 0, 31);
    // Box in the lava so it can't move horizontally
    s.dirt(4, 1);
    s.dirt(6, 1);

    let snap = s.tick();

    // Obsidian produced at lava's position
    snap.assert_particle_at(5, 1, Particle::Solid(Solid::Obsidian));
    // Water consumed
    snap.assert_empty_at(&[(5, 5)]);
}

/// Water crosses the chunk boundary at x=31/32 via horizontal movement.
///
/// Setup: 2×1 chunk map (64 wide), water at (31, 1) moving Right, full dirt floor at y=0.
/// Water moves horizontally from chunk (0,0) to chunk (1,0) via the interchunk queue.
#[test]
fn water_crosses_chunk_boundary() {
    let mut s = SimScenario::new(2, 1);
    s.water(31, 1, Direction::Right);
    s.dirt_floor(0, 0, 63);

    let snap = s.tick();

    // Water moved across chunk boundary to x=32
    snap.assert_particle_at(32, 1, Particle::Liquid(Liquid::Water(Direction::Right)));
    snap.assert_empty_at(&[(31, 1)]);
}

/// Multiple water particles settle into equilibrium on a floor inside a pit.
///
/// Setup: pit with dirt walls at x=2 and x=6 (y=0..6), FULL dirt floor at y=0 (x=0..31).
/// The full floor is essential — it blocks diagonal escape routes at y=0.
/// Three water particles at (3,3), (4,3), (5,3) fall to y=1 on tick 1,
/// then remain at (3,1), (4,1), (5,1) in subsequent ticks (spatially stable,
/// direction oscillates but positions don't change).
#[test]
fn multiple_water_particles_settle() {
    let mut s = SimScenario::new(1, 1);
    // Full floor to block all diagonal escapes at y=0
    s.dirt_floor(0, 0, 31);
    // Pit walls
    s.dirt_wall(2, 0, 6);
    s.dirt_wall(6, 0, 6);
    // Place three water particles above the floor
    s.water(3, 3, Direction::Left);
    s.water(4, 3, Direction::Left);
    s.water(5, 3, Direction::Left);

    let snaps = s.run_ticks(4);

    // After tick 1: all three land at y=1
    let water = Particle::Liquid(Liquid::Water(Direction::Left));
    snaps[0].assert_particles_at(&[(3, 1, water), (4, 1, water), (5, 1, water)]);

    // After 4 ticks: still at the same y positions (spatially settled)
    // Direction may have flipped but position is stable
    for snap in &snaps {
        for x in 3..=5 {
            snap.assert_particle_at(x, 1, water);
        }
    }
}
