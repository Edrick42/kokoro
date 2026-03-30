//! Basic creature animations.
//!
//! Adds life to the creature with two simple animations:
//!
//! 1. **Body sway** — gentle side-to-side rocking on the root entity,
//!    giving the creature a breathing/idle feel.
//!
//! 2. **Eye blink** — periodically closes the eyes for a brief moment
//!    by swapping to the sleeping eye sprite, then back to the current mood.
//!
//! Both animations are driven by timers and sine waves, keeping CPU cost
//! negligible even on low-end hardware.

use bevy::prelude::*;
use crate::mind::Mind;
use crate::creature::species::{BodyPartSlot, CreatureRoot, MoodReactive};
use crate::creature::spawn::PartSpriteHandles;

/// Controls the idle sway animation on the creature root.
#[derive(Component)]
pub struct IdleSway {
    elapsed: f32,
}

/// Controls the blink cycle for eyes.
#[derive(Resource)]
pub struct BlinkTimer {
    /// Time until next blink
    cooldown: Timer,
    /// How long the blink lasts (eyes closed)
    blink_duration: Timer,
    /// Are we currently mid-blink?
    blinking: bool,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BlinkTimer {
                cooldown: Timer::from_seconds(4.0, TimerMode::Repeating),
                blink_duration: Timer::from_seconds(0.15, TimerMode::Once),
                blinking: false,
            })
           .add_systems(Update, (attach_idle_sway, idle_sway_system, blink_system).chain());
    }
}

/// Attaches the `IdleSway` component to creature roots that don't have one yet.
fn attach_idle_sway(
    mut commands: Commands,
    query: Query<Entity, (With<CreatureRoot>, Without<IdleSway>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(IdleSway { elapsed: 0.0 });
    }
}

/// Applies a gentle sine-wave rotation and vertical bob to the creature.
fn idle_sway_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut IdleSway), With<CreatureRoot>>,
) {
    for (mut transform, mut sway) in query.iter_mut() {
        sway.elapsed += time.delta_secs();

        // Gentle rotation sway (±2 degrees)
        let angle = (sway.elapsed * 1.2).sin() * 0.035;
        transform.rotation = Quat::from_rotation_z(angle);

        // Subtle vertical bob (±2 pixels)
        let bob = (sway.elapsed * 1.8).sin() * 2.0;
        transform.translation.y = bob;
    }
}

/// Periodically blinks the creature's eyes.
///
/// Works by temporarily swapping eye sprites to the "sleeping" variant
/// (closed eyes), then swapping back after a short duration.
fn blink_system(
    time: Res<Time>,
    mut blink: ResMut<BlinkTimer>,
    mind: Res<Mind>,
    handles: Option<Res<PartSpriteHandles>>,
    mut eye_q: Query<(&BodyPartSlot, &mut Sprite), With<MoodReactive>>,
) {
    let Some(handles) = handles else { return };

    // Don't blink while sleeping (eyes are already closed)
    if mind.mood == crate::mind::MoodState::Sleeping {
        blink.blinking = false;
        blink.cooldown.reset();
        return;
    }

    if blink.blinking {
        // Currently blinking — check if blink duration is over
        blink.blink_duration.tick(time.delta());
        if blink.blink_duration.finished() {
            blink.blinking = false;
            blink.blink_duration.reset();

            // Restore eyes to current mood
            let mood_key = mind.mood.mood_key();
            for (slot, mut sprite) in eye_q.iter_mut() {
                if slot.0.starts_with("eye_") {
                    let new_handle = handles.handles
                        .get(&(slot.0.clone(), mood_key.to_string()))
                        .or_else(|| handles.handles.get(&(slot.0.clone(), "idle".to_string())));
                    if let Some(handle) = new_handle {
                        sprite.image = handle.clone();
                    }
                }
            }
        }
    } else {
        // Waiting for next blink
        blink.cooldown.tick(time.delta());
        if blink.cooldown.finished() {
            blink.blinking = true;
            blink.cooldown.reset();

            // Randomize next blink interval (3-6 seconds)
            let next_interval = 3.0 + rand::random::<f32>() * 3.0;
            blink.cooldown.set_duration(std::time::Duration::from_secs_f32(next_interval));

            // Swap eyes to sleeping (closed)
            for (slot, mut sprite) in eye_q.iter_mut() {
                if slot.0.starts_with("eye_") {
                    if let Some(handle) = handles.handles.get(&(slot.0.clone(), "sleeping".to_string())) {
                        sprite.image = handle.clone();
                    }
                }
            }
        }
    }
}
