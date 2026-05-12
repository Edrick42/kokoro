//! Neuromuscular actuation pipeline — one frame, one step.
//!
//! The pipeline per joint per frame:
//!
//! 1. The **mind** issues an `intent` for each nerve (flexor + extensor).
//! 2. Each [`Nerve`] delivers a delayed, attenuated signal to its muscle.
//! 3. Each [`Muscle`] ramps its activation toward that signal at its own
//!    `contraction_rate`, updating fatigue.
//! 4. The [`MusclePair`] computes its net muscle torque.
//! 5. The caller feeds that torque (plus gravity, contact, etc.) into
//!    `kokoro_rig::Skeleton::step_physics` which integrates the joint.
//!
//! This module owns step 1-4. Step 5 lives in `kokoro-rig`. The mind that
//! produces intents lives in the consumer (Kokoro itself).

use crate::nerve::Nerve;
use crate::pair::MusclePair;

/// Mind's per-joint command: how strongly the flexor and extensor should
/// be driven this frame. Values are 0..1; values outside that range get
/// clamped by the nerve.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PairIntent {
    pub flexor: f32,
    pub extensor: f32,
}

impl PairIntent {
    pub const fn new(flexor: f32, extensor: f32) -> Self {
        Self { flexor, extensor }
    }

    /// "Rest" — both muscles silent.
    pub const fn rest() -> Self {
        Self {
            flexor: 0.0,
            extensor: 0.0,
        }
    }
}

/// Wire-up: a [`PairIntent`] enters two [`Nerve`]s, then two [`Muscle`]s.
/// Owns no state — pass it the pieces from your body.
pub fn actuate(
    intent: PairIntent,
    nerve_flexor: &mut Nerve,
    nerve_extensor: &mut Nerve,
    pair: &mut MusclePair,
    dt: f32,
) -> f32 {
    let f_sig = nerve_flexor.deliver(intent.flexor, dt);
    let e_sig = nerve_extensor.deliver(intent.extensor, dt);
    pair.flexor.step(f_sig, dt);
    pair.extensor.step(e_sig, dt);
    pair.net_torque()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::muscle::{Muscle, MuscleAttachment};
    use kokoro_rig::BoneId;

    fn fresh_pair() -> MusclePair {
        let mk = |name: &'static str| {
            Muscle::new(
                name,
                MuscleAttachment::new(BoneId(0), 1.0),
                MuscleAttachment::new(BoneId(1), 2.0),
                10.0,
            )
        };
        let mut p = MusclePair::new(mk("flexor"), mk("extensor"));
        p.flexor.contraction_rate = 20.0;
        p.extensor.contraction_rate = 20.0;
        p
    }

    #[test]
    fn rest_intent_produces_zero_torque() {
        let mut nf = Nerve::new("nf", 1.0, 1.0);
        let mut ne = Nerve::new("ne", 1.0, 1.0);
        let mut p = fresh_pair();
        let t = actuate(PairIntent::rest(), &mut nf, &mut ne, &mut p, 0.01);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn driving_extensor_produces_eventual_positive_torque() {
        let mut nf = Nerve::new("nf", 5.0, 1.0);
        let mut ne = Nerve::new("ne", 5.0, 1.0);
        let mut p = fresh_pair();
        let dt = 0.01;
        // Push extensor intent for many frames; signal needs latency +
        // muscle ramp time before torque emerges.
        let mut last = 0.0;
        for _ in 0..30 {
            last = actuate(
                PairIntent::new(0.0, 1.0),
                &mut nf,
                &mut ne,
                &mut p,
                dt,
            );
        }
        assert!(last > 5.0, "expected sustained positive torque, got {last}");
    }

    #[test]
    fn co_contraction_cancels() {
        let mut nf = Nerve::new("nf", 1.0, 1.0);
        let mut ne = Nerve::new("ne", 1.0, 1.0);
        let mut p = fresh_pair();
        let dt = 0.01;
        let mut last = 1.0;
        for _ in 0..50 {
            last = actuate(
                PairIntent::new(1.0, 1.0),
                &mut nf,
                &mut ne,
                &mut p,
                dt,
            );
        }
        // Both muscles equally strong → torques cancel.
        assert!(last.abs() < 0.5, "expected near-zero co-contraction torque, got {last}");
    }
}
