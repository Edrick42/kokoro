//! Creature visual spawning and lifecycle.
//!
//! Replaces the old `creature_form.rs` (procedural meshes) and `sprite.rs`
//! (single full-character PNG) with a modular composition system.
//!
//! Each creature is spawned as a root entity with child entities for each
//! body part. Parts can be either sprite-based (if PNGs exist) or procedural
//! mesh-based (as a fallback). The two can coexist: some parts might have
//! sprites while others fall back to meshes.
//!
//! ## Asset path convention
//!
//! ```text
//! assets/sprites/{species_dir}/{slot}_{mood_key}.png
//! ```
//!
//! For example: `assets/sprites/kobara/eye_left_hungry.png`
//!
//! If a mood-specific sprite is missing, the system tries `{slot}_idle.png`.
//! If that is also missing, the procedural mesh fallback is used.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::genome::Genome;
use crate::mind::Mind;
use super::body_parts::*;

/// Preloaded sprite handles for all (slot, mood_key) combinations.
/// Used by the mood_sync system for fast sprite swaps without disk I/O.
#[derive(Resource)]
pub struct PartSpriteHandles {
    /// Map of (slot, mood_key) → image handle.
    pub handles: HashMap<(String, String), Handle<Image>>,
}

/// Tracks whether we checked if sprites loaded successfully.
/// After a 2-second grace period, we check which parts got real sprites
/// and despawn their fallback meshes.
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
           .add_systems(Update, check_sprite_fallback);
    }
}

/// Spawns the creature as a root entity with child body parts.
///
/// For each part defined in the species template, this system:
/// 1. Tries to load a sprite from disk
/// 2. Always spawns a procedural mesh fallback alongside it
/// 3. After 2 seconds, the fallback check will hide whichever is not needed
fn spawn_creature(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    registry: Res<SpeciesRegistry>,
) {
    let template = registry.get(&genome.species);
    let mood_key = mind.mood.mood_key();
    let body_color = genome.body_color();
    let tint = genome.tint_color();

    let mut sprite_handles = PartSpriteHandles {
        handles: HashMap::new(),
    };

    // Preload all mood variants for all mood-reactive parts
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
            // Non-reactive parts only need the idle variant
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

    // Spawn the root entity with all parts as children
    commands.spawn((
        CreatureRoot,
        Transform::default(),
        Visibility::Inherited,
    )).with_children(|parent| {
        for part_def in &template.parts {
            let slot = BodyPartSlot(part_def.slot.clone());

            // Determine which sprite handle to use (mood-specific or idle)
            let sprite_handle = sprite_handles.handles
                .get(&(part_def.slot.clone(), mood_key.to_string()))
                .or_else(|| sprite_handles.handles.get(&(part_def.slot.clone(), "idle".to_string())))
                .cloned();

            // Spawn sprite entity (hidden until fallback check confirms it loaded)
            if let Some(handle) = sprite_handle {
                let color = if part_def.tinted { tint } else { Color::WHITE };
                let mut entity = parent.spawn((
                    Sprite {
                        image: handle,
                        color,
                        ..default()
                    },
                    Transform::from_xyz(part_def.offset.x, part_def.offset.y, part_def.z_depth)
                        .with_scale(part_def.base_scale.extend(1.0)),
                    Visibility::Hidden,
                    slot.clone(),
                ));
                if part_def.mood_reactive {
                    entity.insert(MoodReactive);
                }
                if part_def.tinted {
                    entity.insert(Tinted);
                }
            }

            // Always spawn procedural mesh fallback
            let mesh_color = part_def.fallback_color.unwrap_or(body_color);
            match &part_def.fallback_shape {
                FallbackShape::Circle { radius } => {
                    parent.spawn((
                        Mesh2d(meshes.add(Circle::new(*radius))),
                        MeshMaterial2d(materials.add(mesh_color)),
                        Transform::from_xyz(part_def.offset.x, part_def.offset.y, part_def.z_depth),
                        slot,
                        FallbackMesh,
                    ));
                }
                FallbackShape::Rect { width, height } => {
                    parent.spawn((
                        Mesh2d(meshes.add(Rectangle::new(*width, *height))),
                        MeshMaterial2d(materials.add(mesh_color)),
                        Transform::from_xyz(part_def.offset.x, part_def.offset.y, part_def.z_depth),
                        slot,
                        FallbackMesh,
                    ));
                }
            }
        }
    });

    commands.insert_resource(sprite_handles);
    commands.insert_resource(SpriteFallback {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
        resolved: false,
    });
}

/// After 2 seconds, checks which sprite parts actually loaded.
/// For parts that have sprites: show the sprite, despawn the fallback mesh.
/// For parts without sprites: the fallback mesh stays visible.
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

    // Check which slots have at least one sprite that loaded
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

    // Show sprite entities for loaded slots
    for (slot, mut vis) in sprite_q.iter_mut() {
        if loaded_slots.contains(&slot.0) {
            *vis = Visibility::Visible;
        }
    }

    // Despawn fallback meshes for slots that have sprites
    for (entity, slot) in mesh_q.iter() {
        if loaded_slots.contains(&slot.0) {
            commands.entity(entity).despawn();
        }
    }
}
