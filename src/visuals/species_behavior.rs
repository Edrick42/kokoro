//! Per-species idle behaviors.
//!
//! Replaces the old generic IdleSway with unique animations for each species:
//! - **Moluun**: ear twitches
//! - **Pylum**: wing flutter, head bob, tail sway
//! - **Skael**: slow tail sway
//! - **Nyxal**: tentacle undulation, mantle rotation

use std::f32::consts::TAU;
use bevy::prelude::*;

use crate::creature::species::{BodyPartSlot, CreatureRoot};
use crate::genome::Species;
use crate::mind::absence::AbsenceState;

/// Preserves the rig-resolved position of a body part so animations can
/// apply deltas on top without losing the original offset.
#[derive(Component, Clone)]
pub struct BasePosition(pub Vec3);

/// Drives per-species idle animation on a creature.
#[derive(Component)]
pub struct SpeciesBehavior {
    pub species: Species,
    pub elapsed: f32,
}

pub struct SpeciesBehaviorPlugin;

impl Plugin for SpeciesBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, species_idle_system);
    }
}

/// Reunion intensity multiplier — amplifies animations when the player just returned.
/// Returns 1.0 normally, up to 3.0 during reunion.
fn reunion_intensity(absence: &AbsenceState) -> f32 {
    if absence.acknowledged || absence.reunion_ticks == 0 {
        return 1.0;
    }
    // Stronger reunion for longer absences
    let base = if absence.seconds_away > 14400 { 3.0 }
        else if absence.seconds_away > 1800 { 2.0 }
        else { 1.5 };
    // Decay over the reunion period
    let remaining = absence.reunion_ticks as f32 / 5.0;
    1.0 + (base - 1.0) * remaining
}

fn species_idle_system(
    time: Res<Time>,
    absence: Option<Res<AbsenceState>>,
    mut root_q: Query<(&mut SpeciesBehavior, &Children), With<CreatureRoot>>,
    mut part_q: Query<(&BodyPartSlot, &mut Transform, &BasePosition), Without<CreatureRoot>>,
) {
    let dt = time.delta_secs();
    let intensity = absence
        .as_ref()
        .map(|a| reunion_intensity(a))
        .unwrap_or(1.0);

    for (mut behavior, children) in root_q.iter_mut() {
        behavior.elapsed += dt;
        let t = behavior.elapsed;

        for child in children.iter() {
            let Ok((slot, mut transform, base)) = part_q.get_mut(child) else {
                continue;
            };

            match behavior.species {
                Species::Moluun => animate_moluun(&slot.0, t, &mut transform, base, intensity),
                Species::Pylum  => animate_pylum(&slot.0, t, &mut transform, base, intensity),
                Species::Skael  => animate_skael(&slot.0, t, &mut transform, base, intensity),
                Species::Nyxal  => animate_nyxal(&slot.0, t, &mut transform, base, intensity),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Moluun — gentle ear twitches
// ---------------------------------------------------------------------------

fn animate_moluun(slot: &str, t: f32, transform: &mut Transform, base: &BasePosition, intensity: f32) {
    transform.translation = base.0;

    if slot.contains("ear") {
        // Twitch every ~4 seconds for 0.2 seconds — faster during reunion
        let period = 4.0 / intensity;
        let cycle = t % period;
        let twitch = if cycle < 0.2 {
            (cycle / 0.2 * TAU).sin() * 0.09 * intensity
        } else {
            0.0
        };
        let sign = if slot.contains("left") { 1.0 } else { -1.0 };
        transform.rotation = Quat::from_rotation_z(twitch * sign);
    } else if slot == "body" && intensity > 1.1 {
        // Reunion bounce — Moluun bounces excitedly
        let bounce = (t * 4.0 * intensity * TAU).sin().abs() * 3.0 * (intensity - 1.0);
        transform.translation = base.0 + Vec3::new(0.0, bounce, 0.0);
    }
}

// ---------------------------------------------------------------------------
// Pylum — wing flutter, head bob, tail sway
// ---------------------------------------------------------------------------

fn animate_pylum(slot: &str, t: f32, transform: &mut Transform, base: &BasePosition, intensity: f32) {
    if slot.contains("wing") {
        // Flutter at ~3Hz, ±8 degrees — more intense during reunion (excited flutter)
        let angle = (t * 3.0 * intensity * TAU).sin() * 0.14 * intensity;
        let sign = if slot.contains("left") { 1.0 } else { -1.0 };
        transform.rotation = Quat::from_rotation_z(angle * sign);
        transform.translation = base.0;
    } else if slot == "body" {
        // Subtle vertical bob
        let bob = (t * 1.5 * TAU).sin() * 3.0;
        transform.translation = base.0 + Vec3::new(0.0, bob, 0.0);
    } else if slot == "tail" {
        // Slow rotation sway
        let angle = (t * 0.8 * TAU).sin() * 0.09;
        transform.rotation = Quat::from_rotation_z(angle);
        transform.translation = base.0;
    } else {
        transform.translation = base.0;
    }
}

// ---------------------------------------------------------------------------
// Skael — slow tail sway, rigid crests
// ---------------------------------------------------------------------------

fn animate_skael(slot: &str, t: f32, transform: &mut Transform, base: &BasePosition, intensity: f32) {
    transform.translation = base.0;

    if slot == "tail" {
        // Slow sway ±4 degrees at 0.5Hz
        let angle = (t * 0.5 * TAU).sin() * 0.07;
        transform.rotation = Quat::from_rotation_z(angle);
    }

    // Skael reunion: no visible excitement — but positions slightly closer (body shifts forward)
    if slot == "body" && intensity > 1.1 {
        let shift = (intensity - 1.0) * 5.0;
        transform.translation = base.0 + Vec3::new(0.0, shift, 0.0);
    }
}

// ---------------------------------------------------------------------------
// Nyxal — tentacle undulation, mantle rotation
// ---------------------------------------------------------------------------

fn animate_nyxal(slot: &str, t: f32, transform: &mut Transform, base: &BasePosition, intensity: f32) {
    if slot.contains("tentacle") {
        // Each tentacle gets a different phase offset
        let phase = match slot {
            "tentacle_front_left"  => 0.0,
            "tentacle_front_right" => TAU * 0.25,
            "tentacle_back_left"   => TAU * 0.50,
            "tentacle_back_right"  => TAU * 0.75,
            _ => 0.0,
        };

        // Rotation undulation ±12 degrees at 1.2Hz — pulses faster during reunion
        let angle = ((t * 1.2 * intensity * TAU) + phase).sin() * 0.21 * intensity;
        transform.rotation = Quat::from_rotation_z(angle);

        // Small vertical waviness
        let wave = ((t * 0.8 * TAU) + phase).sin() * 2.5;
        transform.translation = base.0 + Vec3::new(0.0, wave, 0.0);
    } else if slot == "mantle" {
        // Gentle rotation oscillation ±3 degrees at 0.6Hz
        let angle = (t * 0.6 * TAU).sin() * 0.05;
        transform.rotation = Quat::from_rotation_z(angle);
        transform.translation = base.0;
    } else {
        transform.translation = base.0;
    }
}
