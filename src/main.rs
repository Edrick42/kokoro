mod creature;
#[cfg(feature = "dev")]
mod dev;
mod genome;
mod mind;
mod persistence;
mod ui;
mod visuals;
mod world;

use bevy::prelude::*;

use creature::{
    collection::MultiCreaturePlugin,
    physics::PhysicsPlugin,
    spawn::CreatureVisualsPlugin,
};
use mind::plugin::NeuralMindPlugin;
use persistence::plugin::PersistencePlugin;
use ui::{
    actions::ActionsPlugin,
    hud::StatsPlugin,
    vitals::VitalsPlugin,
};
use visuals::{
    accessories::AccessoriesPlugin,
    animation::AnimationPlugin,
    breathing::BreathingPlugin,
    effects::EffectsPlugin,
    evolution::EvolutionPlugin,
    genome_visuals::apply_genome_visuals,
    mood_sync::sync_mood_sprites,
    resonance_glow::ResonanceGlowPlugin,
    species_behavior::SpeciesBehaviorPlugin,
};
use world::{
    daycycle::DayCyclePlugin,
    setup_world,
    time_tick::TimeTickPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
        .add_plugins((StatsPlugin, ActionsPlugin, VitalsPlugin))
        // Creature lifecycle — collection management
        .add_plugins(MultiCreaturePlugin)
        // Physics — gravity, collision, buoyancy
        .add_plugins(PhysicsPlugin)
        // Visual plugins — effects, animation, evolution, accessories, organic behavior
        .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin))
        .add_plugins((BreathingPlugin, SpeciesBehaviorPlugin, ResonanceGlowPlugin))
        // Visual update systems
        .add_systems(Update, (sync_mood_sprites, apply_genome_visuals));

    // Dev mode — only compiled with `cargo run --features dev`
    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevPlugin);

    app.run();
}
