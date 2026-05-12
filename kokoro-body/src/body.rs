//! `Body` — bundles a skeleton with its neuromuscular system.
//!
//! Holds the skeleton (bones + joints + physics state), and per joint a
//! pair of nerves + a muscle pair. Drives one whole-body update per
//! frame:
//!
//! 1. For every actuated joint, call `actuate` with the mind's intent
//!    → gets a net muscle torque.
//! 2. Combine that with the joint's gravity / contact / external torques
//!    (caller-supplied).
//! 3. Step the skeleton's physics by `dt`.
//! 4. Forward kinematics (optional — caller can defer).
//!
//! The body owns no opinion about *what* intents to issue; that's the
//! mind's job. The body just runs the physiology.

use crate::actuation::{actuate, PairIntent};
use crate::nerve::Nerve;
use crate::pair::MusclePair;
use kokoro_rig::{AppliedTorques, BoneId, Skeleton};

/// Per-joint neuromuscular unit. Each actuated joint has exactly one of
/// these, mapped by `BoneId`.
pub struct Actuator {
    pub bone: BoneId,
    pub nerve_flexor: Nerve,
    pub nerve_extensor: Nerve,
    pub muscles: MusclePair,
}

impl Actuator {
    pub fn new(
        bone: BoneId,
        nerve_flexor: Nerve,
        nerve_extensor: Nerve,
        muscles: MusclePair,
    ) -> Self {
        Self {
            bone,
            nerve_flexor,
            nerve_extensor,
            muscles,
        }
    }
}

/// A complete kobara body: skeleton (bones + joints + physics state) plus
/// a list of per-joint actuators. The mind issues intents; the body runs
/// the physiology.
pub struct Body {
    pub skeleton: Skeleton,
    pub actuators: Vec<Actuator>,
}

impl Body {
    pub fn new(skeleton: Skeleton) -> Self {
        Self {
            skeleton,
            actuators: Vec::new(),
        }
    }

    /// Attach an actuator to one of the skeleton's joints. The bone must
    /// already have a joint installed (`Skeleton::install_joint`) so the
    /// integrator has something to drive.
    pub fn attach_actuator(&mut self, actuator: Actuator) {
        debug_assert!(
            self.skeleton.joint(actuator.bone).is_some(),
            "attach_actuator: bone {:?} has no installed joint",
            actuator.bone
        );
        self.actuators.push(actuator);
    }

    /// One full physiology step.
    ///
    /// `intent_for(bone)` is called once per actuator and should return
    /// the mind's per-frame command. `external_for(bone)` is called once
    /// per *all* physics-driven joints and supplies non-muscle torques
    /// (gravity, contact). Both default to rest / zero when omitted by
    /// the caller's closures.
    pub fn step(
        &mut self,
        dt: f32,
        mut intent_for: impl FnMut(BoneId) -> PairIntent,
        mut external_for: impl FnMut(BoneId) -> AppliedTorques,
    ) {
        // 1-3: actuator pipeline → compute muscle torques per joint.
        let mut muscle_torques: Vec<(BoneId, f32)> = Vec::with_capacity(self.actuators.len());
        for a in &mut self.actuators {
            let intent = intent_for(a.bone);
            let torque = actuate(
                intent,
                &mut a.nerve_flexor,
                &mut a.nerve_extensor,
                &mut a.muscles,
                dt,
            );
            muscle_torques.push((a.bone, torque));
        }

        // 4: step physics on the skeleton, merging muscle + external.
        let muscle_by_bone = muscle_torques;
        self.skeleton.step_physics(dt, |bone| {
            let muscle = muscle_by_bone
                .iter()
                .find(|(b, _)| *b == bone)
                .map(|(_, t)| *t)
                .unwrap_or(0.0);
            let mut applied = external_for(bone);
            applied.muscle += muscle;
            applied
        });
    }
}
