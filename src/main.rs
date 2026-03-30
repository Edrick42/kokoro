mod creature;
mod genome;
mod mind;
mod persistence;
mod ui;
mod visuals;
mod world;

use bevy::prelude::*;

use creature::{
    collection::MultiCreaturePlugin,
    reproduction::ReproductionPlugin,
    spawn::CreatureVisualsPlugin,
};
use mind::plugin::NeuralMindPlugin;
use persistence::plugin::PersistencePlugin;
use ui::{
    actions::ActionsPlugin,
    creature_selector::CreatureSelectorPlugin,
    hud::StatsPlugin,
};
use visuals::{
    accessories::AccessoriesPlugin,
    animation::AnimationPlugin,
    effects::EffectsPlugin,
    evolution::EvolutionPlugin,
    genome_visuals::apply_genome_visuals,
    mood_sync::sync_mood_sprites,
};
use world::{
    daycycle::DayCyclePlugin,
    setup_world,
    time_tick::TimeTickPlugin,
};

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
        .add_plugins((DayCyclePlugin, TimeTickPlugin))
        // Neural mind — learns owner interaction patterns
        .add_plugins(NeuralMindPlugin)
        // UI plugins
        .add_plugins((StatsPlugin, ActionsPlugin, CreatureSelectorPlugin))
        // Creature lifecycle — reproduction, collection
        .add_plugins((ReproductionPlugin, MultiCreaturePlugin))
        // Visual plugins — effects, animation, evolution, accessories
        .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin))
        // Visual update systems
        .add_systems(Update, (sync_mood_sprites, apply_genome_visuals))
        .run();
}
