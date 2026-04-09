//! Creature spawning and lifecycle.
//!
//! Spawns creature root entities with all gameplay components.
//! Visual rendering is handled by PixelCreaturePlugin (runtime pixel art).
//! When the player switches creatures, the old entity is despawned
//! and a new one is created.

use bevy::prelude::*;

use crate::genome::{Genome, Species};
use crate::mind::Mind;
use crate::creature::collection::CreatureCollection;
use crate::creature::egg::EggEntity;
use crate::creature::physics::{PhysicsBody, GROUND_Y};
use crate::visuals::species_behavior::SpeciesBehavior;
use crate::audio::VocalRepertoire;
use crate::mind::lifecycle::LifecycleState;
use crate::mind::nutrition::NutrientState;
use crate::mind::preferences::PreferenceMemory;
use crate::visuals::breathing::{BreathingState, HeartbeatState};
use super::species::*;

pub struct CreatureVisualsPlugin;

impl Plugin for CreatureVisualsPlugin {
    fn build(&self, app: &mut App) {
        let registry = SpeciesRegistry::new();
        app.insert_resource(registry)
           .add_systems(Startup, spawn_creature)
           .add_systems(Update, respawn_on_switch);
    }
}

/// Spawns the creature (or egg) at startup.
fn spawn_creature(
    commands: Commands,
    genome: Res<Genome>,
    mind: Res<Mind>,
    collection: Res<CreatureCollection>,
) {
    let is_egg = collection.creatures
        .get(collection.active_index)
        .map(|c| !c.egg.hatched)
        .unwrap_or(false);

    if is_egg {
        do_spawn_egg(commands, &genome);
    } else {
        do_spawn_creature(commands, &genome, &mind);
    }
}

/// Checks if creature visuals need respawning (after switch or hatch).
fn respawn_on_switch(
    mut collection: ResMut<CreatureCollection>,
    root_q: Query<Entity, With<CreatureRoot>>,
    egg_q: Query<Entity, With<EggEntity>>,
    commands: Commands,
    genome: Res<Genome>,
    mind: Res<Mind>,
) {
    if !collection.visuals_dirty {
        return;
    }
    collection.visuals_dirty = false;

    // Despawn old entities
    let mut cmds = commands;
    for entity in root_q.iter() {
        cmds.entity(entity).despawn();
    }
    for entity in egg_q.iter() {
        cmds.entity(entity).despawn();
    }

    let is_egg = collection.creatures
        .get(collection.active_index)
        .map(|c| !c.egg.hatched)
        .unwrap_or(false);

    if is_egg {
        do_spawn_egg(cmds, &genome);
    } else {
        do_spawn_creature(cmds, &genome, &mind);
    }
}

/// Spawns an egg entity (simple colored ellipse).
fn do_spawn_egg(
    mut commands: Commands,
    genome: &Genome,
) {
    let color = crate::creature::egg::egg_color(&genome.species);
    commands.spawn((
        EggEntity,
        Sprite {
            color,
            custom_size: Some(Vec2::new(60.0, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
    ));
}

/// Spawns the creature root entity with all gameplay components.
/// Visual rendering (pixel art) is attached by PixelCreaturePlugin.
fn do_spawn_creature(
    mut commands: Commands,
    genome: &Genome,
    _mind: &Mind,
) {
    // Physics body: land creatures fall, aquatic creatures float
    let physics = match genome.species {
        Species::Nyxal => PhysicsBody::aquatic_creature(GROUND_Y + 80.0),
        _ => PhysicsBody::land_creature(GROUND_Y),
    };

    // Spawn root entity — PixelCreaturePlugin will attach the visual sprite
    commands.spawn((
        CreatureRoot,
        physics,
        NutrientState::default(),
        PreferenceMemory::default(),
        VocalRepertoire::new(&genome.species),
        LifecycleState::new(&genome.species),
        SpeciesBehavior { species: genome.species.clone(), elapsed: 0.0 },
        BreathingState::new(),
        HeartbeatState::new(),
        Transform::default(),
        Visibility::Inherited,
    ));
}
