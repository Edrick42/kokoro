//! Creature spawning and lifecycle.
//!
//! Spawns creature root entities with all gameplay components.
//! Visual rendering is handled by PixelCreaturePlugin (runtime pixel art).
//! When the player switches creatures, the old entity is despawned
//! and a new one is created.

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::RgbaImage;

use crate::genome::{Genome, Species};
use crate::mind::Mind;
use crate::creature::lifecycle::collection::CreatureCollection;
use crate::creature::lifecycle::egg::EggEntity;
use crate::creature::interaction::physics::{PhysicsBody, GROUND_Y};
use crate::visuals::species_behavior::SpeciesBehavior;
use crate::visuals::skin;
use crate::audio::VocalRepertoire;
use crate::mind::lifecycle::LifecycleState;
use crate::mind::nutrition::NutrientState;
use crate::mind::preferences::PreferenceMemory;
use crate::visuals::breathing::{BreathingState, HeartbeatState};
use crate::game::state::{AppState, GameplayEntity};
use crate::creature::identity::species::*;

pub struct CreatureVisualsPlugin;

impl Plugin for CreatureVisualsPlugin {
    fn build(&self, app: &mut App) {
        let registry = SpeciesRegistry::new();
        app.insert_resource(registry)
           .add_systems(OnEnter(AppState::Gameplay), spawn_creature)
           .add_systems(Update, respawn_on_switch.run_if(in_state(AppState::Gameplay)));
    }
}

/// Spawns the creature (or egg) at startup.
fn spawn_creature(
    commands: Commands,
    mut images: ResMut<Assets<Image>>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    collection: Res<CreatureCollection>,
) {
    let is_egg = collection.creatures
        .get(collection.active_index)
        .map(|c| !c.egg.hatched)
        .unwrap_or(false);

    if is_egg {
        do_spawn_egg(commands, &mut images, &genome);
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
    mut images: ResMut<Assets<Image>>,
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
        do_spawn_egg(cmds, &mut images, &genome);
    } else {
        do_spawn_creature(cmds, &genome, &mind);
    }
}

/// Spawns an egg entity with pixel art rendering.
fn do_spawn_egg(
    mut commands: Commands,
    images: &mut Assets<Image>,
    genome: &Genome,
) {
    // Create pixel art egg
    let mut buf = RgbaImage::new(64, 64);
    skin::draw_egg(&mut buf, &genome.species);

    let mut image = Image::new_fill(
        Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    image.sampler = ImageSampler::nearest();
    if let Some(ref mut data) = image.data {
        data.copy_from_slice(buf.as_raw());
    }
    let handle = images.add(image);

    commands.spawn((
        EggEntity,
        GameplayEntity,
        Sprite {
            image: handle,
            custom_size: Some(Vec2::new(64.0 * 5.0, 64.0 * 5.0)),
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
    mind: &Mind,
) {
    // Physics body: land creatures fall, aquatic creatures float
    let physics = match genome.species {
        Species::Nyxal => PhysicsBody::aquatic_creature(GROUND_Y + 80.0),
        _ => PhysicsBody::land_creature(GROUND_Y),
    };

    // Initialize soft body physics for this species + growth stage
    use crate::creature::interaction::soft_body;
    use crate::visuals::evolution::GrowthStage;
    let stage = GrowthStage::from_age_pub(mind.age_ticks);
    let soft_body = match (&genome.species, stage) {
        (Species::Moluun, GrowthStage::Cub) => soft_body::moluun_cub(),
        (Species::Moluun, _) => soft_body::moluun_adult(),
        _ => soft_body::moluun_adult(), // TODO: other species
    };
    commands.insert_resource(soft_body);

    // Spawn root entity — SkinPlugin will attach the visual sprite
    commands.spawn((
        CreatureRoot,
        GameplayEntity,
        physics,
        NutrientState::default(),
        PreferenceMemory::default(),
        VocalRepertoire::new(&genome.species, &genome),
        LifecycleState::new(&genome.species),
        SpeciesBehavior { species: genome.species.clone(), elapsed: 0.0 },
        BreathingState::new(),
        HeartbeatState::new(),
        Transform::default(),
        Visibility::Inherited,
    ));
}
