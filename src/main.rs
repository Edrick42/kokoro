mod audio;
pub mod config;
mod creature;
#[cfg(feature = "dev")]
mod dev;
mod game;
mod genome;
mod mind;
mod persistence;
mod ui;
mod visuals;
mod web;
mod world;

use bevy::prelude::*;

use audio::SoundPlugin;
use creature::{
    anatomy::AnatomyPlugin,
    lifecycle::collection::MultiCreaturePlugin,
    lifecycle::egg::EggPlugin,
    lifecycle::spawn::CreatureVisualsPlugin,
    interaction::physics::PhysicsPlugin,
    interaction::touch::TouchPlugin,
};
use game::state::{AppState, GameStatePlugin};
use web::WebPlugin;
use creature::abilities::AbilityPlugin;
use creature::behavior::idle::IdleBehaviorPlugin;
use creature::behavior::involuntary::InvoluntaryPlugin;
use creature::behavior::pose::PosePlugin;
use creature::behavior::reactions::ReactionPlugin;
use mind::autonomic::AutonomicPlugin;
use mind::disease::DiseasePlugin;
use mind::hygiene::HygienePlugin;
use mind::lifecycle::LifecyclePlugin;
use mind::nutrition::NutritionPlugin;
use mind::plugin::NeuralMindPlugin;
use mind::preferences::PreferencePlugin;
use persistence::plugin::PersistencePlugin;
use ui::{
    actions::ActionsPlugin,
    auth_screen::AuthScreenPlugin,
    death_screen::DeathScreenPlugin,
    hud::StatsPlugin,
    onboarding_screen::OnboardingScreenPlugin,
    side_menu::SideMenuPlugin,
    title_screen::TitleScreenPlugin,
    vitals::VitalsPlugin,
};
use visuals::{
    accessories::AccessoriesPlugin,
    animation::AnimationPlugin,
    background::BackgroundPlugin,
    breathing::BreathingPlugin,
    effects::EffectsPlugin,
    evolution::EvolutionPlugin,
    genome_visuals::apply_genome_visuals,
    mood_sync::sync_mood_sprites,
    skin::SkinPlugin,
    resonance_glow::ResonanceGlowPlugin,
    species_behavior::SpeciesBehaviorPlugin,
};
use world::{
    daycycle::DayCyclePlugin,
    environment::EnvironmentPlugin,
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
        // Game state machine (Loading → TitleScreen → Auth → Gameplay)
        .add_plugins(GameStatePlugin)
        // Retro font + palette (must load before UI plugins)
        .add_plugins(config::ui::RetroFontPlugin)
        // Web API client (auth + creature sync)
        .add_plugins(WebPlugin)
        // Persistence — loads (or creates) Genome and Mind resources
        .add_plugins(PersistencePlugin)
        // Camera
        .add_systems(Startup, setup_world)
        // Screen plugins (state-gated internally)
        .add_plugins((TitleScreenPlugin, AuthScreenPlugin, OnboardingScreenPlugin, DeathScreenPlugin))
        // Creature visuals — modular body part composition
        .add_plugins(CreatureVisualsPlugin)
        // World systems
        .add_plugins((DayCyclePlugin, TimeTickPlugin, EnvironmentPlugin))
        // Neural mind — learns owner interaction patterns
        .add_plugins((NeuralMindPlugin, NutritionPlugin, HygienePlugin, DiseasePlugin, AutonomicPlugin))
        // UI plugins (gameplay)
        .add_plugins((StatsPlugin, ActionsPlugin, VitalsPlugin, SideMenuPlugin))
        // Creature lifecycle — collection management + egg incubation + anatomy
        .add_plugins((MultiCreaturePlugin, EggPlugin, TouchPlugin, PreferencePlugin, SoundPlugin, LifecyclePlugin, AnatomyPlugin, AbilityPlugin, PosePlugin, ReactionPlugin, IdleBehaviorPlugin, InvoluntaryPlugin))
        // Physics — gravity, collision, buoyancy
        .add_plugins(PhysicsPlugin)
        // Visual plugins — effects, animation, evolution, accessories, organic behavior
        .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin, BackgroundPlugin))
        .add_plugins((BreathingPlugin, SpeciesBehaviorPlugin, ResonanceGlowPlugin, SkinPlugin))
        // Visual update systems
        .add_systems(Update, (sync_mood_sprites, apply_genome_visuals).run_if(in_state(AppState::Gameplay)));

    // Dev mode — only compiled with `cargo run --features dev`
    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevPlugin);

    app.run();
}
