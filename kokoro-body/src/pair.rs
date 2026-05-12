//! Antagonist muscle pair — the smallest functional unit that can move a
//! joint in both directions.
//!
//! Biological joints are driven by **pairs** of muscles on opposite sides:
//! a flexor pulls one way, an extensor pulls the other. Both contracted
//! at once = co-contraction (stiffness without motion); only one
//! contracted = motion in that direction.
//!
//! Each muscle is independently controlled (its own activation, its own
//! nerve), so the pair is just a data wrapper. The net torque this pair
//! produces on the joint is:
//!
//! ```text
//! torque = (flexor.current_force × flexor.insertion.lever_arm)
//!        − (extensor.current_force × extensor.insertion.lever_arm)
//! ```
//!
//! Sign convention: positive torque rotates the joint in the +angle
//! direction (per kokoro-rig's right-hand-down convention, that's
//! clockwise in image space).

use crate::muscle::Muscle;

/// A flexor + extensor pair attached around the same joint. Either side
/// can be silently activated (no signal → no contraction), so the pair
/// naturally collapses to one-way motion when only one half is innervated.
#[derive(Clone, Debug)]
pub struct MusclePair {
    /// Pulls the joint toward `range_min` (positive torque convention:
    /// flexor produces *negative* torque to bend the joint "inward").
    pub flexor: Muscle,
    /// Pulls the joint toward `range_max` (positive torque).
    pub extensor: Muscle,
}

impl MusclePair {
    pub const fn new(flexor: Muscle, extensor: Muscle) -> Self {
        Self { flexor, extensor }
    }

    /// Net muscle torque this pair applies to the joint. Uses the
    /// insertion lever arm of each muscle.
    pub fn net_torque(&self) -> f32 {
        let extensor_t = self.extensor.current_force() * self.extensor.insertion.lever_arm;
        let flexor_t = self.flexor.current_force() * self.flexor.insertion.lever_arm;
        extensor_t - flexor_t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::muscle::{Muscle, MuscleAttachment};
    use kokoro_rig::BoneId;

    fn pair() -> MusclePair {
        let flexor = Muscle::new(
            "flexor",
            MuscleAttachment::new(BoneId(0), 1.0),
            MuscleAttachment::new(BoneId(1), 2.0),
            10.0,
        );
        let extensor = Muscle::new(
            "extensor",
            MuscleAttachment::new(BoneId(0), 1.0),
            MuscleAttachment::new(BoneId(1), 2.0),
            10.0,
        );
        MusclePair::new(flexor, extensor)
    }

    #[test]
    fn balanced_co_contraction_produces_zero_torque() {
        let mut p = pair();
        p.flexor.activation = 1.0;
        p.extensor.activation = 1.0;
        // Equal max_force, equal lever arms → cancels out.
        assert!(p.net_torque().abs() < 1e-6);
    }

    #[test]
    fn extensor_only_produces_positive_torque() {
        let mut p = pair();
        p.extensor.activation = 1.0;
        // 10 × 1.0 × 2.0 = 20.
        assert!((p.net_torque() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn flexor_only_produces_negative_torque() {
        let mut p = pair();
        p.flexor.activation = 1.0;
        assert!((p.net_torque() + 20.0).abs() < 1e-6);
    }

    #[test]
    fn unequal_strengths_skew_balance() {
        let mut p = pair();
        // Strong extensor vs weak flexor at equal activation.
        if let crate::muscle::MuscleComposition::Aggregate { max_force } =
            &mut p.extensor.composition
        {
            *max_force = 15.0;
        }
        p.flexor.activation = 1.0;
        p.extensor.activation = 1.0;
        // Net torque positive (extensor wins).
        assert!(p.net_torque() > 0.0);
    }
}
