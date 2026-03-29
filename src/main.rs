mod genome;
mod mind;
mod persistence;
mod systems;
mod world;

use bevy::prelude::*;

use persistence::plugin::PersistencePlugin;
use systems::{
    creature_form::setup_creature,
    stats::StatsPlugin,
    time_tick::TimeTickPlugin,
};
use world::setup_world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Persistence runs first — loads (or creates) Genome and Mind resources
        .add_plugins(PersistencePlugin)
        // Startup systems
        .add_systems(Startup, (setup_world, setup_creature))
        // Gameplay plugins
        .add_plugins((TimeTickPlugin, StatsPlugin))
        .run();
}
