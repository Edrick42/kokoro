//! Mood-driven visual effects.
//!
//! Spawns floating effect sprites above the creature based on its current mood:
//! - **Sleeping** → floating ZZZ bubbles
//! - **Happy / Playful** → hearts
//! - **Lonely** → rain cloud
//! - **Sick** → dizzy stars
//!
//! Effects are spawned as children of the `CreatureRoot` so they move with the
//! creature. Each effect entity is tagged with `MoodEffect` and gets despawned
//! when the mood changes to something that doesn't use that effect.

use bevy::prelude::*;
use crate::mind::{Mind, MoodState};
use super::body_parts::CreatureRoot;

/// Marker for effect sprite entities so we can find and despawn them.
#[derive(Component)]
pub struct MoodEffect;

/// Tracks which mood we last spawned effects for, so we only respawn
/// when the mood actually changes.
#[derive(Resource)]
struct CurrentEffectMood(Option<MoodState>);

/// Animates effects: floating upward, gentle oscillation.
#[derive(Component)]
pub struct EffectAnimation {
    /// Elapsed time for this effect's animation
    elapsed: f32,
    /// Base Y position (before animation offset)
    base_y: f32,
    /// Horizontal oscillation amplitude
    sway_amp: f32,
    /// Float speed (pixels per second upward)
    float_speed: f32,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentEffectMood(None))
           .add_systems(Update, (spawn_mood_effects, animate_effects).chain());
    }
}

/// Checks if the mood changed and spawns/despawns effect sprites accordingly.
fn spawn_mood_effects(
    mut commands: Commands,
    mind: Res<Mind>,
    asset_server: Res<AssetServer>,
    mut current: ResMut<CurrentEffectMood>,
    root_q: Query<Entity, With<CreatureRoot>>,
    effect_q: Query<Entity, With<MoodEffect>>,
) {
    if !mind.is_changed() {
        return;
    }

    // Check if mood actually changed
    if current.0.as_ref() == Some(&mind.mood) {
        return;
    }
    current.0 = Some(mind.mood.clone());

    // Despawn all existing effects
    for entity in effect_q.iter() {
        commands.entity(entity).despawn();
    }

    // Determine which effect sprite to show (if any)
    let effect_path = match &mind.mood {
        MoodState::Sleeping => Some("sprites/shared/effects/zzz.png"),
        MoodState::Happy    => None, // happy is the baseline, no effect
        MoodState::Playful  => Some("sprites/shared/effects/hearts.png"),
        MoodState::Lonely   => Some("sprites/shared/effects/rain_cloud.png"),
        MoodState::Sick     => Some("sprites/shared/effects/stars_dizzy.png"),
        MoodState::Hungry   => None,
        MoodState::Tired    => None,
    };

    let Some(path) = effect_path else { return };
    let Ok(root) = root_q.single() else { return };

    // Spawn the effect as a child of the creature root
    let handle: Handle<Image> = asset_server.load(path);

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: handle,
                color: Color::srgba(1.0, 1.0, 1.0, 0.85),
                ..default()
            },
            Transform::from_xyz(30.0, 80.0, 5.0)
                .with_scale(Vec3::splat(0.6)),
            MoodEffect,
            EffectAnimation {
                elapsed: 0.0,
                base_y: 80.0,
                sway_amp: 4.0,
                float_speed: 8.0,
            },
        ));
    });
}

/// Gently floats and sways effect sprites.
fn animate_effects(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EffectAnimation), With<MoodEffect>>,
) {
    for (mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        // Gentle float up and down (sine wave)
        let float_offset = (anim.elapsed * anim.float_speed * 0.3).sin() * 6.0;
        transform.translation.y = anim.base_y + float_offset;

        // Gentle horizontal sway
        let sway = (anim.elapsed * 1.5).sin() * anim.sway_amp;
        transform.translation.x = 30.0 + sway;

        // Subtle pulse
        let scale = 0.6 + (anim.elapsed * 2.0).sin() * 0.05;
        transform.scale = Vec3::splat(scale);
    }
}
