//! Pose layer system — composable per-bone angle deltas.
//!
//! ## Mental model
//!
//! A static creature is a skeleton at its `rest_angle`s. To make it look
//! alive we want several **independent** sources of motion to *add up*:
//!
//! 1. **Mood**: a tired creature has its head pendulous, a happy one has
//!    ears alert. Mood-driven deltas are sampled from a fixed table.
//! 2. **Tics**: autonomous micro-anims that always run — blink, breathing
//!    pulse, occasional ear-twitch. Each tic is a small periodic function.
//! 3. **Reactions**: short overrides (a lick, a startle) that fade out.
//! 4. **Soft body drift**: physics layer, applied separately by the bridge
//!    in `softbody.rs` (next module).
//!
//! Each of these wants to contribute *deltas* to specific bones, not
//! *replace* the bone's pose. So we use an accumulator: every layer adds
//! its delta on top, the final per-bone angle is `rest_angle + sum(deltas)`,
//! applied to the skeleton in one pass.
//!
//! ## Design choices
//!
//! - **Deltas, not absolute angles.** Layers compose; the last layer
//!   doesn't win, they all sum. Order of contribution doesn't matter (sum
//!   is commutative), which keeps the system intuitive.
//! - **Lookup by bone name, not id.** Layers are species-agnostic data —
//!   the `Tics` for "blink the eyes" should work for any creature whose
//!   skeleton has a bone called `eye_l`. Names are the stable interface.
//! - **Time is `f32` seconds.** Caller passes whatever clock it wants
//!   (Bevy's `Time::elapsed_secs`, a manual counter in tests, etc.).

use crate::bone::BoneId;
use crate::skeleton::Skeleton;

/// Accumulates per-bone angle deltas. Reset and refilled every frame.
///
/// Indexed by `BoneId`, so it's tied to a specific skeleton's bone count.
/// Keep it allocated across frames and `reset()` instead of recreating —
/// avoids allocation churn.
#[derive(Clone, Debug)]
pub struct PoseAccumulator {
    deltas: Vec<f32>,
}

impl PoseAccumulator {
    /// Allocate space for a skeleton with `bone_count` bones.
    pub fn new(bone_count: usize) -> Self {
        Self { deltas: vec![0.0; bone_count] }
    }

    /// Build an accumulator sized for the given skeleton.
    pub fn for_skeleton(skeleton: &Skeleton) -> Self {
        Self::new(skeleton.len())
    }

    /// Zero every delta. Call once at the start of each frame, before any
    /// layer contributions.
    pub fn reset(&mut self) {
        for d in &mut self.deltas {
            *d = 0.0;
        }
    }

    /// Add a delta to a specific bone. Multiple `add` calls on the same
    /// bone accumulate.
    pub fn add(&mut self, id: BoneId, delta_radians: f32) {
        self.deltas[id.index()] += delta_radians;
    }

    /// Read back the accumulated delta for a bone.
    pub fn delta(&self, id: BoneId) -> f32 {
        self.deltas[id.index()]
    }

    /// Apply accumulated deltas to the skeleton: each bone's pose angle
    /// becomes `rest_angle + delta`. Marks the skeleton dirty for the next
    /// `forward()` call.
    pub fn apply_to(&self, skeleton: &mut Skeleton) {
        for (i, &delta) in self.deltas.iter().enumerate() {
            let id = BoneId(i as u16);
            let rest = skeleton.bone(id).rest_angle;
            skeleton.set_angle(id, rest + delta);
        }
    }
}

/// Anything that contributes per-bone deltas to a `PoseAccumulator`. Layers
/// don't read from the accumulator — they only add — so any number of
/// independent sources can be composed without ordering bugs.
pub trait PoseLayer {
    fn contribute(&self, accumulator: &mut PoseAccumulator, skeleton: &Skeleton, time_seconds: f32);
}

/// A simple table-driven layer: each entry is `(bone_name, delta_radians)`.
/// The same bone name can appear multiple times; deltas sum.
///
/// Used for mood offsets (e.g., when sad: head_neck = +0.2, ear_l = -0.3,
/// ear_r = -0.3). The caller picks the right table for the current state
/// and keeps a single `BoneAngleOffsets` per state in memory.
#[derive(Clone, Debug)]
pub struct BoneAngleOffsets {
    pub entries: Vec<(&'static str, f32)>,
}

impl BoneAngleOffsets {
    pub const fn new(entries: Vec<(&'static str, f32)>) -> Self {
        Self { entries }
    }

    pub const fn empty() -> Self {
        Self { entries: Vec::new() }
    }
}

impl PoseLayer for BoneAngleOffsets {
    fn contribute(&self, accumulator: &mut PoseAccumulator, skeleton: &Skeleton, _time: f32) {
        for &(name, delta) in &self.entries {
            if let Some(id) = skeleton.id_of(name) {
                accumulator.add(id, delta);
            }
            // Silently ignore unknown bones: lets us share offset tables
            // across species that have different skeletons.
        }
    }
}

/// A single autonomous micro-animation on a bone. Compiles to one of two
/// shapes — a smooth sin wave (breathing, slow swaying) or a brief impulse
/// firing on a period (blink, ear-twitch).
#[derive(Clone, Debug)]
pub struct Tic {
    pub bone_name: &'static str,
    pub kind: TicKind,
}

#[derive(Copy, Clone, Debug)]
pub enum TicKind {
    /// `delta = amplitude · sin(2π · frequency · time + phase)`.
    /// Use for breathing, idle swaying, gentle ambient motion.
    Sin {
        amplitude: f32,
        frequency: f32, // Hz
        phase: f32,
    },
    /// Brief square impulse: every `period` seconds the delta jumps to
    /// `amplitude` for `duration` seconds, otherwise 0. Use for blinks
    /// and twitches.
    Pulse {
        amplitude: f32,
        period: f32,   // seconds
        duration: f32, // seconds the impulse stays high
        phase: f32,    // seconds offset (so multiple pulses can stagger)
    },
}

impl Tic {
    pub fn delta_at(&self, time_seconds: f32) -> f32 {
        match self.kind {
            TicKind::Sin { amplitude, frequency, phase } => {
                amplitude * (std::f32::consts::TAU * frequency * time_seconds + phase).sin()
            }
            TicKind::Pulse { amplitude, period, duration, phase } => {
                let t = ((time_seconds + phase) % period.max(1e-6)).max(0.0);
                if t < duration {
                    amplitude
                } else {
                    0.0
                }
            }
        }
    }
}

/// A bag of `Tic`s. Stays applied every frame regardless of mood.
#[derive(Clone, Debug, Default)]
pub struct TicLayer {
    pub tics: Vec<Tic>,
}

impl PoseLayer for TicLayer {
    fn contribute(&self, accumulator: &mut PoseAccumulator, skeleton: &Skeleton, time: f32) {
        for tic in &self.tics {
            if let Some(id) = skeleton.id_of(tic.bone_name) {
                accumulator.add(id, tic.delta_at(time));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::{Bone, Vec2};

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn skeleton_with_bones(names: &[&'static str]) -> Skeleton {
        let mut bones = vec![Bone::root(names[0], 1.0, 1.0)];
        for &n in &names[1..] {
            bones.push(Bone::child(n, BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0));
        }
        Skeleton::new(bones)
    }

    #[test]
    fn accumulator_starts_at_zero_and_resets() {
        let mut acc = PoseAccumulator::new(3);
        acc.add(BoneId(1), 0.5);
        assert_eq!(acc.delta(BoneId(1)), 0.5);
        acc.reset();
        assert_eq!(acc.delta(BoneId(0)), 0.0);
        assert_eq!(acc.delta(BoneId(1)), 0.0);
    }

    #[test]
    fn deltas_sum_when_added_multiple_times() {
        let mut acc = PoseAccumulator::new(2);
        acc.add(BoneId(0), 0.3);
        acc.add(BoneId(0), 0.4);
        acc.add(BoneId(1), -0.1);
        assert!(approx(acc.delta(BoneId(0)), 0.7, 1e-5));
        assert!(approx(acc.delta(BoneId(1)), -0.1, 1e-5));
    }

    #[test]
    fn apply_writes_rest_plus_delta_into_skeleton() {
        let mut sk = skeleton_with_bones(&["root", "arm"]);
        let arm = sk.id_of("arm").unwrap();
        let mut acc = PoseAccumulator::for_skeleton(&sk);
        acc.add(arm, 0.5);
        acc.apply_to(&mut sk);
        // rest_angle is 0 (default), so effective angle should be 0.5.
        assert!(approx(sk.effective_angle(arm), 0.5, 1e-5));
    }

    #[test]
    fn bone_angle_offsets_layer_lookup_by_name() {
        let sk = skeleton_with_bones(&["root", "ear_l", "ear_r"]);
        let layer = BoneAngleOffsets::new(vec![("ear_l", -0.3), ("ear_r", -0.3)]);
        let mut acc = PoseAccumulator::for_skeleton(&sk);
        layer.contribute(&mut acc, &sk, 0.0);
        assert!(approx(acc.delta(sk.id_of("ear_l").unwrap()), -0.3, 1e-5));
        assert!(approx(acc.delta(sk.id_of("ear_r").unwrap()), -0.3, 1e-5));
    }

    #[test]
    fn unknown_bone_in_offsets_is_silently_ignored() {
        let sk = skeleton_with_bones(&["root"]);
        let layer = BoneAngleOffsets::new(vec![("nonexistent", 999.0)]);
        let mut acc = PoseAccumulator::for_skeleton(&sk);
        layer.contribute(&mut acc, &sk, 0.0);
        assert_eq!(acc.delta(BoneId(0)), 0.0);
    }

    #[test]
    fn sin_tic_oscillates_around_zero() {
        let tic = Tic {
            bone_name: "x",
            kind: TicKind::Sin { amplitude: 0.1, frequency: 1.0, phase: 0.0 },
        };
        // At t=0: sin(0) = 0.
        assert!(approx(tic.delta_at(0.0), 0.0, 1e-5));
        // At t=0.25 (quarter cycle): sin(π/2) = 1 → amplitude.
        assert!(approx(tic.delta_at(0.25), 0.1, 1e-5));
        // At t=0.5 (half cycle): sin(π) = 0.
        assert!(approx(tic.delta_at(0.5), 0.0, 1e-4));
    }

    #[test]
    fn pulse_tic_fires_on_period() {
        let tic = Tic {
            bone_name: "eye",
            kind: TicKind::Pulse {
                amplitude: -1.5,
                period: 4.0,
                duration: 0.2,
                phase: 0.0,
            },
        };
        // Inside the pulse window (0..0.2 seconds of every 4-second cycle).
        assert_eq!(tic.delta_at(0.0), -1.5);
        assert_eq!(tic.delta_at(0.1), -1.5);
        // Outside.
        assert_eq!(tic.delta_at(0.5), 0.0);
        assert_eq!(tic.delta_at(2.0), 0.0);
        // Next cycle.
        assert_eq!(tic.delta_at(4.05), -1.5);
    }

    #[test]
    fn multiple_layers_compose_by_summing() {
        let sk = skeleton_with_bones(&["root", "head"]);
        let head = sk.id_of("head").unwrap();
        let mood = BoneAngleOffsets::new(vec![("head", 0.2)]);
        let tics = TicLayer {
            tics: vec![Tic {
                bone_name: "head",
                kind: TicKind::Sin { amplitude: 0.05, frequency: 0.5, phase: 0.0 },
            }],
        };
        let mut acc = PoseAccumulator::for_skeleton(&sk);
        mood.contribute(&mut acc, &sk, 0.5);
        tics.contribute(&mut acc, &sk, 0.5); // sin(π/2) = 1
        // 0.2 + 0.05 = 0.25
        assert!(approx(acc.delta(head), 0.25, 1e-4));
    }

    #[test]
    fn applying_zero_deltas_does_not_change_rest() {
        let mut sk = skeleton_with_bones(&["root", "arm"]);
        let arm = sk.id_of("arm").unwrap();
        // Set a non-default rest angle.
        sk.set_angle(arm, 0.75);
        sk.clear_angle(arm); // back to rest
        // Empty accumulator → applies rest + 0 = rest.
        let acc = PoseAccumulator::for_skeleton(&sk);
        acc.apply_to(&mut sk);
        assert!(approx(sk.effective_angle(arm), 0.0, 1e-5));
    }
}
