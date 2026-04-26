//! Breathing and heartbeat systems.
//!
//! - **Breathing**: subtle body scale oscillation (inhale/exhale).
//! - **Heartbeat**: BPM tracking tied to health and mood.
//!
//! The visual signs panel (BPM, breathing rate) is rendered by `ui/vitals.rs`.

use std::f32::consts::TAU;
use bevy::prelude::*;
use rand::Rng;

use crate::game::state::AppState;

use crate::config;
use crate::creature::identity::species::CreatureRoot;
use crate::genome::{Genome, Species};
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
#[allow(dead_code)]
pub struct BaseBodyScale(pub Vec2);

pub struct BreathingPlugin;

impl Plugin for BreathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_breathing_params,
            breathing_system,
            heartbeat_timer_system,
        ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

fn update_breathing_params(
    mind: Res<Mind>,
    ans: Option<Res<crate::mind::autonomic::AutonomicState>>,
    mut query: Query<(&mut BreathingState, &mut HeartbeatState), With<CreatureRoot>>,
) {
    // ANS arousal drives breathing/heartbeat instead of mood lookup tables
    let arousal = ans.as_ref().map(|a| a.level).unwrap_or(0.4);
    let arousal_mult = 0.5 + arousal; // 0.5 (calm) to 1.5 (alert)

    for (mut breathing, mut heartbeat) in query.iter_mut() {
        // Breathing rate: base rate scaled by arousal
        let base_rate = config::breathing::DEFAULT_RATE;
        breathing.target_rate = base_rate * arousal_mult;

        // Override: sleeping forces very slow breathing
        if mind.mood == MoodState::Sleeping {
            breathing.target_rate = config::breathing::RATE_SLEEPING;
        }

        breathing.target_amplitude = config::breathing::AMPLITUDE_BASE
            + (mind.stats.energy / 100.0) * config::breathing::AMPLITUDE_ENERGY_FACTOR;

        // Heartbeat: base BPM + arousal scaling
        let base_bpm = config::heartbeat::BASE_BPM
            + (100.0 - mind.stats.health) * config::heartbeat::HEALTH_BPM_FACTOR;
        heartbeat.target_bpm = base_bpm * arousal_mult;

        // Irregularity from sympathetic stress
        heartbeat.irregularity = if arousal > 0.7 {
            (arousal - 0.7) * config::heartbeat::SICK_IRREGULARITY * 3.0
        } else {
            0.0
        };
    }
}

fn breathing_system(
    time: Res<Time>,
    mut root_q: Query<&mut BreathingState, With<CreatureRoot>>,
    genome: Res<Genome>,
    mut soft_body: Option<ResMut<crate::creature::interaction::soft_body::SoftBody>>,
) {
    let dt = time.delta_secs();

    for mut breathing in root_q.iter_mut() {
        let lerp_speed = 2.0 * dt;
        breathing.rate += (breathing.target_rate - breathing.rate) * lerp_speed;
        breathing.amplitude += (breathing.target_amplitude - breathing.amplitude) * lerp_speed;
        breathing.phase += breathing.rate * TAU * dt;
        if breathing.phase > TAU {
            breathing.phase -= TAU;
        }

        let Some(ref mut body) = soft_body else { continue };
        let pulse = breathing.phase.sin() * breathing.amplitude;

        // Species-specific breathing signature.
        // Positive y = downward in soft-body space.
        match genome.species {
            Species::Moluun => {
                // Mammal: belly rises/falls, head subtly lifts on inhale.
                body.impulse("belly", Vec2::new(0.0, pulse * 80.0));
                body.impulse("head", Vec2::new(0.0, -pulse * 8.0));
            }
            Species::Pylum => {
                // Bird: rapid shallow breathing — chest + tail counterbalance.
                body.impulse("belly", Vec2::new(0.0, pulse * 70.0));
                body.impulse("tail", Vec2::new(0.0, -pulse * 10.0));
            }
            Species::Skael => {
                // Reptile: slow deep belly expansion; tail tip drifts with breath.
                body.impulse("belly", Vec2::new(0.0, pulse * 90.0));
                body.impulse("tail_3", Vec2::new(0.0, -pulse * 5.0));
            }
            Species::Nyxal => {
                // Cephalopod: mantle pulsates dramatically — the *entire* breathing pattern.
                // The belly pulse is dominant here because the mantle IS the body.
                body.impulse("belly", Vec2::new(0.0, pulse * 60.0));
                body.impulse("mantle_top", Vec2::new(0.0, -pulse * 40.0));
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
