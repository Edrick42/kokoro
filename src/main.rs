mod genome;
mod mind;
mod persistence;
mod systems;
mod world;

use bevy::prelude::*;

use persistence::plugin::PersistencePlugin;
use systems::{
    creature_form::setup_creature,
    sprite::SpritePlugin,
    stats::StatsPlugin,
    time_tick::TimeTickPlugin,
    ui::actions::ActionsPlugin,
};
use world::{daycycle::DayCyclePlugin, setup_world};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kokoro".into(),
                resolution: (400.0, 700.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        // Persistence runs first — loads (or creates) Genome and Mind resources
        .add_plugins(PersistencePlugin)
        // Startup systems — camera and procedural mesh fallback
        .add_systems(Startup, (setup_world, setup_creature))
        // Sprite system — replaces meshes if PNG assets are found
        .add_plugins(SpritePlugin)
        // World systems
        .add_plugins(DayCyclePlugin)
        // Gameplay plugins
        .add_plugins((TimeTickPlugin, StatsPlugin, ActionsPlugin))
        .run();
}
