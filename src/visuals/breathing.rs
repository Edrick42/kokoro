//! Breathing and heartbeat animation.
//!
//! Adds organic life to creatures through two rhythmic systems:
//!
//! - **Breathing**: rhythmic scale oscillation on the body slot.
//!   Rate is tied to mood (calm = slow, playful = fast, sleeping = very slow).
//!   Amplitude is tied to energy level.
//!
//! - **Heartbeat**: periodic scale "thump" on the body. Rate tied to health
//!   and mood. Sick creatures have irregular timing.

use std::f32::consts::TAU;
use bevy::prelude::*;
use rand::Rng;

use crate::creature::species::{BodyPartSlot, CreatureRoot};
use crate::mind::{Mind, MoodState};

/// Breathing state attached to the creature root.
#[derive(Component)]
pub struct BreathingState {
    pub phase: f32,
    pub rate: f32,
    pub amplitude: f32,
    target_rate: f32,
    target_amplitude: f32,
}

impl BreathingState {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            rate: 0.22,
            amplitude: 0.012,
            target_rate: 0.22,
            target_amplitude: 0.012,
        }
    }
}

/// Heartbeat state attached to the creature root.
#[derive(Component)]
pub struct HeartbeatState {
    pub bpm: f32,
    pub irregularity: f32,
    pub pulse_timer: f32,
    pub pulse_active: f32,
    target_bpm: f32,
}

impl HeartbeatState {
    pub fn new() -> Self {
        Self {
            bpm: 72.0,
            irregularity: 0.0,
            pulse_timer: 0.83,
            pulse_active: 0.0,
            target_bpm: 72.0,
        }
    }
}

/// Stores the genome-derived base body scale so breathing can compose with it.
#[derive(Component)]
pub struct BaseBodyScale(pub Vec2);

pub struct BreathingPlugin;

impl Plugin for BreathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_breathing_params,
            breathing_system,
            heartbeat_system,
        ).chain());
    }
}

/// Updates breathing and heartbeat parameters based on mood and stats.
fn update_breathing_params(
    mind: Res<Mind>,
    mut query: Query<(&mut BreathingState, &mut HeartbeatState), With<CreatureRoot>>,
) {
    if !mind.is_changed() {
        // Only update targets on mood/stat changes
        // (the actual values smoothly interpolate each frame)
    }

    for (mut breathing, mut heartbeat) in query.iter_mut() {
        // Breathing rate by mood — slow and gentle, like a real animal
        breathing.target_rate = match mind.mood {
            MoodState::Sleeping => 0.12,
            MoodState::Tired | MoodState::Lonely => 0.18,
            MoodState::Happy => 0.22,
            MoodState::Hungry => 0.25,
            MoodState::Sick => 0.30,
            MoodState::Playful => 0.40,
        };

        // Breathing amplitude — subtle, just enough to feel alive
        breathing.target_amplitude = 0.008 + (mind.stats.energy / 100.0) * 0.012;

        // Heartbeat BPM — resting range, varies with health and mood
        let base_bpm = 50.0 + (100.0 - mind.stats.health) * 0.3;
        heartbeat.target_bpm = base_bpm + match mind.mood {
            MoodState::Playful => 15.0,
            MoodState::Sleeping => -15.0,
            MoodState::Sick => 8.0,
            _ => 0.0,
        };

        heartbeat.irregularity = if mind.mood == MoodState::Sick { 0.4 } else { 0.0 };
    }
}

/// Applies breathing scale oscillation to the body slot.
fn breathing_system(
    time: Res<Time>,
    mut root_q: Query<(&mut BreathingState, &Children), With<CreatureRoot>>,
    mut body_q: Query<(&BodyPartSlot, &mut Transform, &BaseBodyScale), Without<CreatureRoot>>,
) {
    let dt = time.delta_secs();

    for (mut breathing, children) in root_q.iter_mut() {
        // Smooth interpolation of breathing params
        let lerp_speed = 2.0 * dt;
        breathing.rate += (breathing.target_rate - breathing.rate) * lerp_speed;
        breathing.amplitude += (breathing.target_amplitude - breathing.amplitude) * lerp_speed;
        breathing.phase += breathing.rate * TAU * dt;
        if breathing.phase > TAU {
            breathing.phase -= TAU;
        }

        // Apply scale to body slot children
        let breath_factor_x = 1.0 + breathing.phase.sin() * breathing.amplitude;
        let breath_factor_y = 1.0 + breathing.phase.sin() * breathing.amplitude * 0.7;

        for child in children.iter() {
            if let Ok((slot, mut transform, base_scale)) = body_q.get_mut(child) {
                if slot.0 == "body" {
                    transform.scale.x = base_scale.0.x * breath_factor_x;
                    transform.scale.y = base_scale.0.y * breath_factor_y;
                }
            }
        }
    }
}

/// Applies heartbeat pulse to the body slot.
fn heartbeat_system(
    time: Res<Time>,
    mut query: Query<(&mut HeartbeatState, &Children), With<CreatureRoot>>,
    mut body_q: Query<(&BodyPartSlot, &mut Transform), Without<CreatureRoot>>,
) {
    let dt = time.delta_secs();

    for (mut heartbeat, children) in query.iter_mut() {
        // Smooth BPM interpolation
        let lerp_speed = 2.0 * dt;
        heartbeat.bpm += (heartbeat.target_bpm - heartbeat.bpm) * lerp_speed;

        // Count down to next beat
        heartbeat.pulse_timer -= dt;

        if heartbeat.pulse_timer <= 0.0 {
            // Trigger a beat
            let beat_interval = 60.0 / heartbeat.bpm.max(30.0);
            let jitter = if heartbeat.irregularity > 0.0 {
                let mut rng = rand::rng();
                rng.random_range(-1.0..1.0) * heartbeat.irregularity * 0.3
            } else {
                0.0
            };
            heartbeat.pulse_timer = beat_interval + jitter;
            heartbeat.pulse_active = 0.08; // 80ms pulse
        }

        // Apply pulse scale to body
        if heartbeat.pulse_active > 0.0 {
            heartbeat.pulse_active -= dt;
            let pulse_strength = (heartbeat.pulse_active / 0.08).max(0.0) * 0.01;

            for child in children.iter() {
                if let Ok((slot, mut transform)) = body_q.get_mut(child) {
                    if slot.0 == "body" {
                        // Additive on top of whatever breathing set
                        transform.scale.x += pulse_strength;
                        transform.scale.y += pulse_strength;
                    }
                }
            }
        }
    }
}
