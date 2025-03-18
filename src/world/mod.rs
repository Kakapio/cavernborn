pub mod camera;
pub mod chunk;
pub mod generator;
pub mod map;
use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    time::{Fixed, Time},
};
use generator::setup_map;
use map::{
    simulate_active_particles, update_active_chunks, update_map_dirty_chunks, SIMULATION_RATE,
};

pub use self::map::Map;

/// Plugin that handles the map systems
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(SIMULATION_RATE))
            .add_systems(Startup, setup_map)
            .add_systems(Update, (update_active_chunks, update_map_dirty_chunks))
            .add_systems(FixedUpdate, simulate_active_particles);
    }
}
