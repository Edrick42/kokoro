//! Sprite rendering system for the Kobara.
//!
//! Loads mood-specific PNGs from `assets/sprites/kobara/` and swaps the
//! displayed image whenever the creature's mood changes. A tint derived
//! from the genome's `hue` gene is applied on top.
//!
//! ## Fallback behaviour
//!
//! If the sprite files are not present, the procedural meshes from
//! `creature_form.rs` remain visible. A 2-second grace timer checks
//! whether the idle sprite resolved; if it did, the meshes are despawned
//! and the sprite takes over.

use bevy::prelude::*;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::systems::creature_form::{CreatureBody, CreatureEyes, CreatureMouth};

/// Marker component for the sprite entity.
#[derive(Component)]
pub struct KobaraSprite;

/// Holds all mood sprite handles loaded at startup.
#[derive(Resource)]
struct SpriteHandles {
    idle:     Handle<Image>,
    hungry:   Handle<Image>,
    tired:    Handle<Image>,
    lonely:   Handle<Image>,
    playful:  Handle<Image>,
    sick:     Handle<Image>,
    sleeping: Handle<Image>,
}

/// Tracks whether we already tried to switch from meshes to sprites.
#[derive(Resource)]
struct SpriteFallback {
    timer: Timer,
    resolved: bool,
}

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sprites)
           .add_systems(Update, (check_fallback, sync_mood_sprite).chain());
    }
}

/// Loads all sprite assets at startup. Bevy's AssetServer does not panic
/// on missing files — it logs a warning and returns a handle that stays
/// in the `Loading` state forever. This is what makes the fallback safe.
fn load_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    genome: Res<Genome>,
) {
    let handles = SpriteHandles {
        idle:     asset_server.load("sprites/kobara/idle.png"),
        hungry:   asset_server.load("sprites/kobara/hungry.png"),
        tired:    asset_server.load("sprites/kobara/tired.png"),
        lonely:   asset_server.load("sprites/kobara/lonely.png"),
        playful:  asset_server.load("sprites/kobara/playful.png"),
        sick:     asset_server.load("sprites/kobara/sick.png"),
        sleeping: asset_server.load("sprites/kobara/sleeping.png"),
    };

    // Spawn the sprite entity (invisible until the fallback check passes)
    commands.spawn((
        Sprite {
            image: handles.idle.clone(),
            color: genome.tint_color(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.5),
        Visibility::Hidden,
        KobaraSprite,
    ));

    commands.insert_resource(handles);
    commands.insert_resource(SpriteFallback {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
        resolved: false,
    });
}

/// After the grace period, checks if the idle sprite loaded successfully.
/// If yes: makes the sprite visible and despawns procedural meshes.
/// If no: keeps the meshes and hides the sprite entity.
fn check_fallback(
    time: Res<Time>,
    mut fallback: ResMut<SpriteFallback>,
    handles: Res<SpriteHandles>,
    images: Res<Assets<Image>>,
    mut sprite_q: Query<&mut Visibility, With<KobaraSprite>>,
    mesh_body:  Query<Entity, With<CreatureBody>>,
    mesh_eyes:  Query<Entity, With<CreatureEyes>>,
    mesh_mouth: Query<Entity, With<CreatureMouth>>,
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

    // Check if the idle sprite actually loaded
    if images.get(&handles.idle).is_some() {
        info!("Sprites loaded — switching from procedural meshes to sprite rendering");

        // Show the sprite
        for mut vis in sprite_q.iter_mut() {
            *vis = Visibility::Visible;
        }

        // Despawn all procedural mesh entities
        for entity in mesh_body.iter().chain(mesh_eyes.iter()).chain(mesh_mouth.iter()) {
            commands.entity(entity).despawn();
        }
    } else {
        info!("Sprites not found — keeping procedural mesh fallback");
    }
}

/// Swaps the sprite image whenever the creature's mood changes.
fn sync_mood_sprite(
    mind: Res<Mind>,
    handles: Option<Res<SpriteHandles>>,
    fallback: Option<Res<SpriteFallback>>,
    mut sprite_q: Query<&mut Sprite, With<KobaraSprite>>,
) {
    // Only run if sprites were loaded and the fallback resolved to sprites
    let Some(handles) = handles else { return };
    let Some(fb) = fallback else { return };
    if !fb.resolved || !mind.is_changed() {
        return;
    }

    let new_handle = match mind.mood.sprite_name() {
        "idle.png"     => handles.idle.clone(),
        "hungry.png"   => handles.hungry.clone(),
        "tired.png"    => handles.tired.clone(),
        "lonely.png"   => handles.lonely.clone(),
        "playful.png"  => handles.playful.clone(),
        "sick.png"     => handles.sick.clone(),
        "sleeping.png" => handles.sleeping.clone(),
        _              => handles.idle.clone(),
    };

    for mut sprite in sprite_q.iter_mut() {
        sprite.image = new_handle.clone();
    }
}
