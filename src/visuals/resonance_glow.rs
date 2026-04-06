//! Kokoro-sac resonance glow visualization.
//!
//! Every Kobara has a kokoro-sac — a resonance organ that vibrates with
//! emotional state. This system renders it as a soft, pulsing glow behind
//! the creature's body.
//!
//! Frequencies are derived from the lore:
//! - Happy:   1.5 Hz — warm steady pulse
//! - Hungry:  3.0 Hz — urgent fluttering
//! - Tired:   0.8 Hz — slow deep wave
//! - Lonely:  4.0 Hz — sharp piercing
//! - Playful: 2.0 Hz — quick rhythmic bursts
//! - Sick:    5.0 Hz — discordant arrhythmic
//! - Sleeping: 0.3 Hz — near-imperceptible hum

use std::f32::consts::TAU;
use bevy::prelude::*;

use crate::config;
use crate::mind::{Mind, MoodState};

/// The kokoro-sac glow, attached as a child entity of CreatureRoot.
#[derive(Component)]
pub struct ResonanceGlow {
    pub frequency: f32,
    pub intensity: f32,
    pub phase: f32,
    target_frequency: f32,
    target_intensity: f32,
}

impl ResonanceGlow {
    pub fn new() -> Self {
        Self {
            frequency: config::resonance::FREQ_HAPPY,
            intensity: config::resonance::INTENSITY_HAPPY,
            phase: 0.0,
            target_frequency: config::resonance::FREQ_HAPPY,
            target_intensity: config::resonance::INTENSITY_HAPPY,
        }
    }
}

pub struct ResonanceGlowPlugin;

impl Plugin for ResonanceGlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_resonance_glow);
    }
}

/// Updates the resonance glow based on mood and animates it.
fn update_resonance_glow(
    time: Res<Time>,
    mind: Res<Mind>,
    mut glow_q: Query<(&mut ResonanceGlow, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (mut glow, mut transform) in glow_q.iter_mut() {
        // Update targets from mood
        glow.target_frequency = match mind.mood {
            MoodState::Sleeping => config::resonance::FREQ_SLEEPING,
            MoodState::Tired    => config::resonance::FREQ_TIRED,
            MoodState::Happy    => config::resonance::FREQ_HAPPY,
            MoodState::Playful  => config::resonance::FREQ_PLAYFUL,
            MoodState::Hungry   => config::resonance::FREQ_HUNGRY,
            MoodState::Lonely   => config::resonance::FREQ_LONELY,
            MoodState::Sick     => config::resonance::FREQ_SICK,
        };

        glow.target_intensity = match mind.mood {
            MoodState::Sleeping => config::resonance::INTENSITY_SLEEPING,
            MoodState::Happy    => config::resonance::INTENSITY_HAPPY,
            MoodState::Tired    => config::resonance::INTENSITY_TIRED,
            MoodState::Hungry   => config::resonance::INTENSITY_HUNGRY,
            MoodState::Lonely   => config::resonance::INTENSITY_LONELY,
            MoodState::Playful  => config::resonance::INTENSITY_PLAYFUL,
            MoodState::Sick     => config::resonance::INTENSITY_SICK,
        };

        // Smooth interpolation
        let lerp_speed = 1.5 * dt;
        glow.frequency += (glow.target_frequency - glow.frequency) * lerp_speed;
        glow.intensity += (glow.target_intensity - glow.intensity) * lerp_speed;

        // Advance phase
        glow.phase += glow.frequency * TAU * dt;
        if glow.phase > TAU {
            glow.phase -= TAU;
        }

        // Sick irregularity: add jitter to create discordant feel
        let phase_val = if mind.mood == MoodState::Sick {
            let jitter = (glow.phase * config::resonance::SICK_JITTER_FREQ).sin()
                * config::resonance::SICK_JITTER_AMP;
            (glow.phase + jitter).sin()
        } else {
            glow.phase.sin()
        };

        // Drive scale — glow expands and contracts with the resonance
        let scale_factor = 1.0 + phase_val * config::resonance::SCALE_AMP * glow.intensity;
        transform.scale = Vec3::splat(scale_factor);
    }
}
