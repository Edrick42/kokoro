mod systems;

use bevy::{prelude::*};

use systems::creature::Creature;
use systems::creature_form::setup_creature;

use systems::world::setup_world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Creature::new()) 
        .add_systems(Startup, (setup_world, setup_creature))
        .run();
}