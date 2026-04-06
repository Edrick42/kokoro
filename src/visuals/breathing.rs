//! Breathing and heartbeat systems.
//!
//! - **Breathing**: subtle body scale oscillation (inhale/exhale).
//! - **Heartbeat**: BPM tracking tied to health and mood.
//!
//! The visual signs panel (BPM, breathing rate) is rendered by `ui/vitals.rs`.

use std::f32::consts::TAU;
use bevy::prelude::*;
use rand::Rng;

use crate::config;
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
            rate: config::breathing::DEFAULT_RATE,
            amplitude: config::breathing::DEFAULT_AMPLITUDE,
            target_rate: config::breathing::DEFAULT_RATE,
            target_amplitude: config::breathing::DEFAULT_AMPLITUDE,
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
            bpm: config::heartbeat::DEFAULT_BPM,
            irregularity: 0.0,
            pulse_timer: 60.0 / config::heartbeat::DEFAULT_BPM,
            pulse_active: 0.0,
            target_bpm: config::heartbeat::DEFAULT_BPM,
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
            heartbeat_timer_system,
        ).chain());
    }
}

fn update_breathing_params(
    mind: Res<Mind>,
    mut query: Query<(&mut BreathingState, &mut HeartbeatState), With<CreatureRoot>>,
) {
    for (mut breathing, mut heartbeat) in query.iter_mut() {
        breathing.target_rate = match mind.mood {
            MoodState::Sleeping => config::breathing::RATE_SLEEPING,
            MoodState::Tired | MoodState::Lonely => config::breathing::RATE_TIRED,
            MoodState::Happy => config::breathing::RATE_HAPPY,
            MoodState::Hungry => config::breathing::RATE_HUNGRY,
            MoodState::Sick => config::breathing::RATE_SICK,
            MoodState::Playful => config::breathing::RATE_PLAYFUL,
        };

        breathing.target_amplitude = config::breathing::AMPLITUDE_BASE
            + (mind.stats.energy / 100.0) * config::breathing::AMPLITUDE_ENERGY_FACTOR;

        let base_bpm = config::heartbeat::BASE_BPM
            + (100.0 - mind.stats.health) * config::heartbeat::HEALTH_BPM_FACTOR;
        heartbeat.target_bpm = base_bpm + match mind.mood {
            MoodState::Playful => config::heartbeat::BPM_PLAYFUL,
            MoodState::Sleeping => config::heartbeat::BPM_SLEEPING,
            MoodState::Sick => config::heartbeat::BPM_SICK,
            _ => 0.0,
        };

        heartbeat.irregularity = if mind.mood == MoodState::Sick {
            config::heartbeat::SICK_IRREGULARITY
        } else {
            0.0
        };
    }
}

fn breathing_system(
    time: Res<Time>,
    mut root_q: Query<(&mut BreathingState, &Children), With<CreatureRoot>>,
    mut body_q: Query<(&BodyPartSlot, &mut Transform, &BaseBodyScale), Without<CreatureRoot>>,
) {
    let dt = time.delta_secs();

    for (mut breathing, children) in root_q.iter_mut() {
        let lerp_speed = 2.0 * dt;
        breathing.rate += (breathing.target_rate - breathing.rate) * lerp_speed;
        breathing.amplitude += (breathing.target_amplitude - breathing.amplitude) * lerp_speed;
        breathing.phase += breathing.rate * TAU * dt;
        if breathing.phase > TAU {
            breathing.phase -= TAU;
        }

        let breath_factor_x = 1.0 + breathing.phase.sin() * breathing.amplitude;
        let breath_factor_y = 1.0 + breathing.phase.sin() * breathing.amplitude
            * config::breathing::Y_AMPLITUDE_RATIO;

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

fn heartbeat_timer_system(
    time: Res<Time>,
    mut query: Query<&mut HeartbeatState, With<CreatureRoot>>,
) {
    let dt = time.delta_secs();

    for mut heartbeat in query.iter_mut() {
        let lerp_speed = 2.0 * dt;
        heartbeat.bpm += (heartbeat.target_bpm - heartbeat.bpm) * lerp_speed;

        heartbeat.pulse_timer -= dt;

        if heartbeat.pulse_timer <= 0.0 {
            let beat_interval = 60.0 / heartbeat.bpm.max(config::heartbeat::MIN_BPM);
            let jitter = if heartbeat.irregularity > 0.0 {
                let mut rng = rand::rng();
                rng.random_range(-1.0..1.0) * heartbeat.irregularity
                    * config::heartbeat::IRREGULARITY_JITTER
            } else {
                0.0
            };
            heartbeat.pulse_timer = beat_interval + jitter;
            heartbeat.pulse_active = config::heartbeat::PULSE_DURATION;
        }

        if heartbeat.pulse_active > 0.0 {
            heartbeat.pulse_active -= dt;
        }
    }
}
