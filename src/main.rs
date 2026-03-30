mod genome;
mod mind;
mod persistence;
mod systems;
mod world;

use bevy::prelude::*;

use persistence::plugin::PersistencePlugin;
use systems::{
    creature_spawn::CreatureVisualsPlugin,
    genome_visuals::apply_genome_visuals,
    mood_sync::sync_mood_sprites,
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
        // Camera
        .add_systems(Startup, setup_world)
        // Creature visuals — modular body part composition
        .add_plugins(CreatureVisualsPlugin)
        // World systems
        .add_plugins(DayCyclePlugin)
        // Gameplay plugins
        .add_plugins((TimeTickPlugin, StatsPlugin, ActionsPlugin))
        // Visual update systems
        .add_systems(Update, (sync_mood_sprites, apply_genome_visuals))
        .run();
}
