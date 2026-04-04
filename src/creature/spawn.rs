//! Creature visual spawning and lifecycle.
//!
//! Each creature is spawned as a root entity with child entities for each
//! body part. Positions come from the **body rig** — a proportional landmark
//! system that resolves normalized coordinates into pixel offsets based on
//! the creature's genome.
//!
//! Parts can be either sprite-based (if PNGs exist) or procedural mesh-based
//! (as a fallback). The two can coexist.
//!
//! When the player switches creatures, the old visual entity is despawned
//! and a new one is spawned from the new species' template.
//!
//! ## Asset path convention
//!
//! ```text
//! assets/sprites/{species_dir}/{slot}_{mood_key}.png
//! ```

use bevy::prelude::*;
use std::collections::HashMap;

use crate::genome::{Genome, Species};
use crate::mind::Mind;
use crate::creature::collection::CreatureCollection;
use crate::creature::physics::{PhysicsBody, GROUND_Y};
use crate::visuals::species_behavior::{BasePosition, SpeciesBehavior};
use crate::visuals::breathing::{BreathingState, HeartbeatState, BaseBodyScale};
use super::species::*;
use super::rig::ResolvedAnchor;

/// Preloaded sprite handles for all (slot, mood_key) combinations.
/// Used by the mood_sync system for fast sprite swaps without disk I/O.
#[derive(Resource)]
pub struct PartSpriteHandles {
    /// Map of (slot, mood_key) → image handle.
    pub handles: HashMap<(String, String), Handle<Image>>,
}

/// Stores the resolved rig positions so other systems (like genome_visuals)
/// can reference the rig-computed positions without re-resolving.
#[derive(Resource)]
pub struct ResolvedRig {
    pub anchors: Vec<ResolvedAnchor>,
}

/// Tracks whether we checked if sprites loaded successfully.
#[derive(Resource)]
struct SpriteFallback {
    timer: Timer,
    resolved: bool,
}

/// Marker for fallback mesh entities (so we can despawn them selectively).
#[derive(Component)]
struct FallbackMesh;

pub struct CreatureVisualsPlugin;

impl Plugin for CreatureVisualsPlugin {
    fn build(&self, app: &mut App) {
        let registry = SpeciesRegistry::new();
        app.insert_resource(registry)
           .add_systems(Startup, spawn_creature)
           .add_systems(Update, (respawn_on_switch, check_sprite_fallback));
    }
}

/// Spawns the creature as a root entity with child body parts.
fn spawn_creature(
    commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    registry: Res<SpeciesRegistry>,
) {
    do_spawn_creature(commands, &asset_server, meshes, materials, &genome, &mind, &registry);
}

/// Checks if creature visuals need respawning (after a switch) and rebuilds them.
fn respawn_on_switch(
    mut collection: ResMut<CreatureCollection>,
    root_q: Query<Entity, With<CreatureRoot>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    registry: Res<SpeciesRegistry>,
) {
    if !collection.visuals_dirty {
        return;
    }
    collection.visuals_dirty = false;

    // Despawn old creature entity and all children
    for entity in root_q.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new creature with updated genome/mind
    do_spawn_creature(commands, &asset_server, meshes, materials, &genome, &mind, &registry);
}

/// Shared spawn logic used by both startup and respawn.
fn do_spawn_creature(
    mut commands: Commands,
    asset_server: &AssetServer,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    genome: &Genome,
    mind: &Mind,
    registry: &SpeciesRegistry,
) {
    let template = registry.get(&genome.species);
    let mood_key = mind.mood.mood_key();
    let body_color = genome.body_color();
    let tint = genome.tint_color();

    // Resolve the rig: convert normalized landmarks → pixel positions
    let resolved = template.rig.resolve(genome);

    // Build a lookup: slot name → resolved position + z_depth
    let anchor_map: HashMap<String, &ResolvedAnchor> = resolved
        .iter()
        .map(|a| (a.slot.clone(), a))
        .collect();

    let mut sprite_handles = PartSpriteHandles {
        handles: HashMap::new(),
    };

    // Preload all mood variants for mood-reactive parts
    let mood_keys = ["idle", "hungry", "tired", "lonely", "playful", "sick", "sleeping"];
    for part_def in &template.parts {
        if part_def.mood_reactive {
            for &mk in &mood_keys {
                let path = format!("sprites/{}/{}_{}.png", template.species_dir, part_def.slot, mk);
                if std::path::Path::new(&format!("assets/{path}")).exists() {
                    let handle = asset_server.load(&path);
                    sprite_handles.handles.insert(
                        (part_def.slot.clone(), mk.to_string()),
                        handle,
                    );
                }
            }
        } else {
            let path = format!("sprites/{}/{}_idle.png", template.species_dir, part_def.slot);
            if std::path::Path::new(&format!("assets/{path}")).exists() {
                let handle = asset_server.load(&path);
                sprite_handles.handles.insert(
                    (part_def.slot.clone(), "idle".to_string()),
                    handle,
                );
            }
        }
    }

    // Physics body: land creatures fall, aquatic creatures float
    let physics = match genome.species {
        Species::Nyxal => PhysicsBody::aquatic_creature(GROUND_Y + 80.0),
        _ => PhysicsBody::land_creature(GROUND_Y),
    };

    // Spawn the root entity with all parts as children
    commands.spawn((
        CreatureRoot,
        physics,
        SpeciesBehavior { species: genome.species.clone(), elapsed: 0.0 },
        BreathingState::new(),
        HeartbeatState::new(),
        Transform::default(),
        Visibility::Inherited,
    )).with_children(|parent| {
        for part_def in &template.parts {
            let slot = BodyPartSlot(part_def.slot.clone());

            // Get position from the resolved rig (or fall back to origin)
            let (offset, z_depth) = anchor_map
                .get(&part_def.slot)
                .map(|a| (a.position, a.z_depth))
                .unwrap_or((Vec2::ZERO, 0.0));

            // Determine which sprite handle to use
            let sprite_handle = sprite_handles.handles
                .get(&(part_def.slot.clone(), mood_key.to_string()))
                .or_else(|| sprite_handles.handles.get(&(part_def.slot.clone(), "idle".to_string())))
                .cloned();

            let base_pos = BasePosition(Vec3::new(offset.x, offset.y, z_depth));

            // Genome-derived body scale for breathing composition
            let body_scale_x = if part_def.slot == "body" {
                1.1 - genome.appetite * 0.2
            } else {
                1.0
            };

            // Spawn sprite entity (hidden until fallback check confirms it loaded)
            if let Some(handle) = sprite_handle {
                let color = if part_def.tinted { tint } else { Color::WHITE };
                let mut entity = parent.spawn((
                    Sprite {
                        image: handle,
                        color,
                        ..default()
                    },
                    Transform::from_xyz(offset.x, offset.y, z_depth)
                        .with_scale(part_def.base_scale.extend(1.0)),
                    Visibility::Hidden,
                    slot.clone(),
                    base_pos.clone(),
                ));
                if part_def.mood_reactive {
                    entity.insert(MoodReactive);
                }
                if part_def.tinted {
                    entity.insert(Tinted);
                }
                if part_def.slot == "body" {
                    entity.insert(BaseBodyScale(Vec2::new(body_scale_x, 1.0)));
                }
            }

            // Always spawn procedural mesh fallback
            let mesh_color = part_def.fallback_color.unwrap_or(body_color);
            let base_pos_fallback = BasePosition(Vec3::new(offset.x, offset.y, z_depth));
            match &part_def.fallback_shape {
                FallbackShape::Circle { radius } => {
                    let mut entity = parent.spawn((
                        Mesh2d(meshes.add(Circle::new(*radius))),
                        MeshMaterial2d(materials.add(mesh_color)),
                        Transform::from_xyz(offset.x, offset.y, z_depth),
                        slot,
                        FallbackMesh,
                        base_pos_fallback,
                    ));
                    if part_def.slot == "body" {
                        entity.insert(BaseBodyScale(Vec2::new(body_scale_x, 1.0)));
                    }
                }
                FallbackShape::Rect { width, height } => {
                    let mut entity = parent.spawn((
                        Mesh2d(meshes.add(Rectangle::new(*width, *height))),
                        MeshMaterial2d(materials.add(mesh_color)),
                        Transform::from_xyz(offset.x, offset.y, z_depth),
                        slot,
                        FallbackMesh,
                        base_pos_fallback,
                    ));
                    if part_def.slot == "body" {
                        entity.insert(BaseBodyScale(Vec2::new(body_scale_x, 1.0)));
                    }
                }
            }
        }
    });

    commands.insert_resource(sprite_handles);
    commands.insert_resource(ResolvedRig { anchors: resolved });
    commands.insert_resource(SpriteFallback {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
        resolved: false,
    });
}

/// After 2 seconds, checks which sprite parts actually loaded.
fn check_sprite_fallback(
    time: Res<Time>,
    mut fallback: ResMut<SpriteFallback>,
    sprite_handles: Res<PartSpriteHandles>,
    images: Res<Assets<Image>>,
    mut sprite_q: Query<(&BodyPartSlot, &mut Visibility), (With<Sprite>, Without<FallbackMesh>)>,
    mesh_q: Query<(Entity, &BodyPartSlot), With<FallbackMesh>>,
    mut commands: Commands,
) {
    if fallback.resolved {
        return;
    }
    fallback.timer.tick(time.delta());
    if !fallback.timer.finished() {
        return;
    }
    fallback.resolved = true;

    let mut loaded_slots = std::collections::HashSet::new();
    for ((slot, _mood), handle) in &sprite_handles.handles {
        if images.get(handle).is_some() {
            loaded_slots.insert(slot.clone());
        }
    }

    if loaded_slots.is_empty() {
        info!("No part sprites found — keeping procedural mesh fallback for all parts");
        return;
    }

    info!("Sprites loaded for parts: {:?} — switching those to sprite rendering", loaded_slots);

    for (slot, mut vis) in sprite_q.iter_mut() {
        if loaded_slots.contains(&slot.0) {
            *vis = Visibility::Visible;
        }
    }

    for (entity, slot) in mesh_q.iter() {
        if loaded_slots.contains(&slot.0) {
            commands.entity(entity).despawn();
        }
    }
}
