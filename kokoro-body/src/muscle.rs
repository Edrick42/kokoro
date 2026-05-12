//! Muscles — the active force producers between bones.
//!
//! A muscle has two attachment points (origin and insertion) on bones,
//! receives an activation signal in 0..1 from its nerve, contracts with
//! a force proportional to `max_force × activation × (1 − fatigue)`, and
//! applies that force at the insertion point. The resulting torque on the
//! joint depends on the lever arm (distance from the joint axis to the
//! insertion).
//!
//! Today muscles are modelled as a single aggregate force; the
//! [`MuscleComposition`] enum holds a forward-compatibility slot for a
//! `Fibers` variant that would track individual fascicles.

use kokoro_rig::BoneId;

/// Where a muscle attaches to a bone.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MuscleAttachment {
    /// Bone the muscle is fastened to.
    pub bone: BoneId,
    /// Distance from the joint axis (= the bone's base) to the attachment,
    /// along the bone's local direction. This becomes the lever arm when
    /// computing torque about the joint. In pixels.
    pub lever_arm: f32,
}

impl MuscleAttachment {
    pub const fn new(bone: BoneId, lever_arm: f32) -> Self {
        Self { bone, lever_arm }
    }
}

/// Internal composition of a muscle. Today only the aggregate force
/// model exists; the `Fibers` variant is reserved for finer-grained
/// modelling without breaking the [`Muscle`] API.
#[derive(Clone, Debug, PartialEq)]
pub enum MuscleComposition {
    /// Lumped model: one peak force, no internal structure.
    Aggregate { max_force: f32 },
    // future:
    // Fibers(Vec<Fiber>),
}

impl MuscleComposition {
    /// Peak force this muscle can deliver at full activation, ignoring
    /// fatigue. In Newtons (or "force units" if you keep things
    /// dimensionless for now).
    pub fn max_force(&self) -> f32 {
        match self {
            MuscleComposition::Aggregate { max_force } => *max_force,
        }
    }
}

/// A skeletal muscle bridging two bones.
#[derive(Clone, Debug)]
pub struct Muscle {
    pub name: &'static str,
    pub origin: MuscleAttachment,
    pub insertion: MuscleAttachment,

    /// Internal composition (aggregate today, fibre-level later).
    pub composition: MuscleComposition,

    /// Current activation level in 0..=1. Driven by the nerve.
    pub activation: f32,

    /// Current fatigue in 0..=1. Subtracts from effective output. Climbs
    /// while activation is high, recovers while it is low. The rates live
    /// here so the genome can vary muscle endurance per creature.
    pub fatigue: f32,
    pub fatigue_rate: f32,    // per second at activation = 1
    pub recovery_rate: f32,   // per second at activation = 0

    /// Maximum 1/s rate at which activation can ramp toward the nerve's
    /// signal. Models myosin cross-bridge cycling: muscles can't go from
    /// 0 to full contraction instantly.
    pub contraction_rate: f32,

    /// Resting lateral cross-section, in canvas pixels. Real physical
    /// state — the muscle's volume, distributed over its rest length,
    /// translates to this perpendicular thickness at activation = 0.
    /// Volume conservation (cf. [`MUSCLE_BULGE_FACTOR`]) then says the
    /// muscle gets thicker when it contracts — see
    /// [`Self::current_thickness`].
    pub rest_thickness: f32,
}

/// Volume-conservation analog: a contracting muscle bulges laterally so
/// its cross-section grows even though its longitudinal length doesn't
/// change in this rigid-hinge model. Real muscles fatten roughly 30–50%
/// at peak contraction; we pick 0.4 as the midpoint.
pub const MUSCLE_BULGE_FACTOR: f32 = 0.4;

impl Muscle {
    pub fn new(
        name: &'static str,
        origin: MuscleAttachment,
        insertion: MuscleAttachment,
        max_force: f32,
    ) -> Self {
        Self {
            name,
            origin,
            insertion,
            composition: MuscleComposition::Aggregate { max_force },
            activation: 0.0,
            fatigue: 0.0,
            fatigue_rate: 0.10,
            recovery_rate: 0.20,
            contraction_rate: 8.0,
            rest_thickness: 1.0,
        }
    }

    /// Effective force this muscle currently produces. Caps at
    /// `max_force × activation × (1 − fatigue)`; never negative.
    pub fn current_force(&self) -> f32 {
        let max = self.composition.max_force();
        let eff = max * self.activation.clamp(0.0, 1.0) * (1.0 - self.fatigue).max(0.0);
        eff.max(0.0)
    }

    /// Current lateral thickness, in the same units as `rest_thickness`.
    /// Approximates volume conservation: when the muscle activates, it
    /// bulges out by `rest_thickness × MUSCLE_BULGE_FACTOR × activation`.
    /// At full activation a muscle is ~40% thicker than at rest.
    pub fn current_thickness(&self) -> f32 {
        self.rest_thickness * (1.0 + MUSCLE_BULGE_FACTOR * self.activation.clamp(0.0, 1.0))
    }

    /// Advance fatigue and activation over `dt` seconds, given the nerve
    /// signal (0..1) that arrived at this muscle this frame.
    ///
    /// Activation chases the signal at `contraction_rate` per second.
    /// Fatigue climbs proportional to activation and recovers proportional
    /// to (1 − activation), both clamped to 0..1.
    pub fn step(&mut self, signal: f32, dt: f32) {
        let target = signal.clamp(0.0, 1.0);
        let max_step = self.contraction_rate * dt;
        let delta = (target - self.activation).clamp(-max_step, max_step);
        self.activation = (self.activation + delta).clamp(0.0, 1.0);

        let fatigue_delta =
            self.fatigue_rate * self.activation * dt - self.recovery_rate * (1.0 - self.activation) * dt;
        self.fatigue = (self.fatigue + fatigue_delta).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kokoro_rig::BoneId;

    fn test_muscle(max_force: f32) -> Muscle {
        Muscle::new(
            "test",
            MuscleAttachment::new(BoneId(0), 1.0),
            MuscleAttachment::new(BoneId(1), 3.0),
            max_force,
        )
    }

    #[test]
    fn fresh_muscle_produces_no_force() {
        let m = test_muscle(10.0);
        assert_eq!(m.current_force(), 0.0);
    }

    #[test]
    fn activation_ramps_toward_signal_at_contraction_rate() {
        let mut m = test_muscle(10.0);
        m.contraction_rate = 4.0;
        m.step(1.0, 0.1);
        // At rate 4/s, after 0.1s activation rose by 0.4.
        assert!((m.activation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn activation_clamps_at_one() {
        let mut m = test_muscle(10.0);
        for _ in 0..100 {
            m.step(1.0, 0.05);
        }
        assert!((m.activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn fatigue_accumulates_under_sustained_activation() {
        let mut m = test_muscle(10.0);
        m.fatigue_rate = 0.5;
        m.recovery_rate = 0.0;
        // Force activation high quickly.
        for _ in 0..50 {
            m.step(1.0, 0.05);
        }
        assert!(m.fatigue > 0.1, "fatigue should climb, got {}", m.fatigue);
    }

    #[test]
    fn fatigue_caps_force_output() {
        let mut m = test_muscle(10.0);
        m.activation = 1.0;
        m.fatigue = 0.5;
        // 10 × 1.0 × (1 − 0.5) = 5.
        assert!((m.current_force() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn recovery_drops_fatigue_when_resting() {
        let mut m = test_muscle(10.0);
        m.fatigue = 0.6;
        m.recovery_rate = 0.5;
        // Drive activation to 0 and let it sit.
        for _ in 0..100 {
            m.step(0.0, 0.05);
        }
        assert!(m.fatigue < 0.1, "fatigue should recover, got {}", m.fatigue);
    }
}
