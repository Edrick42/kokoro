//! Universal physics integrator for a [`Skeleton`](crate::Skeleton).
//!
//! Every joint of every creature in Kokoro obeys the same physical law.
//! This module is that law — a single pure function `integrate_joint`
//! plus a sweep `step_skeleton` that walks the joints in order.
//!
//! ## The law (per joint, per frame)
//!
//! Total torque on the joint is the sum of:
//!
//! - Externally applied torques (muscle, contact, gravity) — passed in
//!   by the caller; the rig itself never invents them.
//! - Passive ligament spring: `−k × (angle − rest_angle)` pulling back
//!   toward rest.
//! - Surface friction: `−d × angular_velocity` resisting motion.
//!
//! Angular acceleration `α = total_torque / I`, where `I` is the moment
//! of inertia of the hanging bone treated as a uniform rod about its end:
//! `I = (1/3) × m × L²`.
//!
//! Semi-implicit Euler step:
//!
//! ```text
//! velocity ← velocity + α × dt
//! angle    ← clamp(angle + velocity × dt, rest+min, rest+max)
//! ```
//!
//! When the ROM clamps an angle, the velocity is zeroed in the violating
//! direction so the joint doesn't keep accumulating torque against the
//! wall. Semi-implicit Euler is chosen over explicit Euler because it is
//! stable for high spring constants, which we need for stiff joints.

use crate::bone::Bone;
use crate::joint::{Joint, JointKind, JointState};

/// External per-frame torques applied to a joint by the higher layers
/// (muscle, contact, gravity). The integrator adds passive spring +
/// friction to this sum. All in N·m.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct AppliedTorques {
    pub muscle: f32,
    pub contact: f32,
    pub gravity: f32,
}

impl AppliedTorques {
    #[inline]
    pub fn sum(&self) -> f32 {
        self.muscle + self.contact + self.gravity
    }
}

/// Moment of inertia of a uniform rod about its end: `I = (1/3) × m × L²`.
/// Bones are treated as uniform rods hinged at the parent's tip; this is
/// the standard approximation for 2D rigid-body physics on segmented chains.
#[inline]
pub fn rod_inertia(mass: f32, length: f32) -> f32 {
    (mass * length * length) / 3.0
}

/// Integrate one timestep for one joint.
///
/// Pure function — no skeleton access, no allocation. The caller is
/// responsible for computing `moment_of_inertia` from the relevant bone
/// (typically via [`rod_inertia`]) and for gathering the externally
/// applied torques.
///
/// Returns the new [`JointState`] after `dt` seconds.
pub fn integrate_joint(
    joint: &Joint,
    state: JointState,
    applied: AppliedTorques,
    moment_of_inertia: f32,
    dt: f32,
) -> JointState {
    // Fused joints never move. Snap back to rest in case state was poked.
    if matches!(joint.kind, JointKind::Fused) {
        return JointState::at_rest(joint.rest_angle);
    }

    let spring = -joint.ligament_k * (state.angle - joint.rest_angle);
    let damping = -joint.surface.friction * state.angular_velocity;
    let total = applied.sum() + spring + damping;

    let i = moment_of_inertia.max(1e-6);
    let alpha = total / i;

    let mut velocity = state.angular_velocity + alpha * dt;
    let mut angle = state.angle + velocity * dt;

    let lo = joint.rest_angle + joint.range_min;
    let hi = joint.rest_angle + joint.range_max;
    if angle <= lo {
        angle = lo;
        if velocity < 0.0 {
            velocity = 0.0;
        }
    } else if angle >= hi {
        angle = hi;
        if velocity > 0.0 {
            velocity = 0.0;
        }
    }

    JointState {
        angle,
        angular_velocity: velocity,
    }
}

/// Convenience: derive the moment of inertia for the bone that hangs from
/// `joint`, using the bone's effective length and mass. Bone width is not
/// used — the rod approximation treats every bone as a slender beam.
#[inline]
pub fn bone_inertia(bone: &Bone) -> f32 {
    rod_inertia(bone.tissue.mass, bone.effective_length())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::{Bone, BoneId};
    use crate::joint::{Joint, JointKind, JointState, JointSurface};

    fn make_hinge(k: f32, friction: f32, range: f32) -> Joint {
        Joint {
            bone: BoneId(1),
            kind: JointKind::Hinge,
            rest_angle: 0.0,
            range_min: -range,
            range_max: range,
            ligament_k: k,
            surface: JointSurface { friction },
        }
    }

    #[test]
    fn rod_inertia_matches_textbook_formula() {
        let i = rod_inertia(3.0, 2.0);
        assert!((i - 4.0).abs() < 1e-6); // (1/3) × 3 × 4 = 4
    }

    #[test]
    fn fused_joint_snaps_back_to_rest() {
        let mut j = make_hinge(0.0, 0.0, 1.0);
        j.kind = JointKind::Fused;
        j.rest_angle = 0.5;
        let s = JointState { angle: 0.0, angular_velocity: 10.0 };
        let out = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.01);
        assert_eq!(out.angle, 0.5);
        assert_eq!(out.angular_velocity, 0.0);
    }

    #[test]
    fn spring_pulls_angle_back_toward_rest() {
        let j = make_hinge(10.0, 0.0, 5.0);
        let s = JointState { angle: 0.5, angular_velocity: 0.0 };
        // No external torque, no friction → only spring acts.
        let out = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.01);
        // Spring torque = -10 × 0.5 = -5 N·m. α = -5. After 0.01s:
        // velocity = -0.05, angle = 0.5 + -0.05 × 0.01 = 0.4995.
        assert!(out.angular_velocity < 0.0);
        assert!(out.angle < s.angle);
    }

    #[test]
    fn friction_resists_velocity() {
        let j = make_hinge(0.0, 2.0, 5.0);
        let s = JointState { angle: 0.0, angular_velocity: 1.0 };
        // No spring, only friction (=-2 × 1 = -2 N·m). α = -2.
        let out = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.01);
        assert!(out.angular_velocity < s.angular_velocity);
    }

    #[test]
    fn muscle_torque_accelerates_in_its_direction() {
        let j = make_hinge(0.0, 0.0, 5.0);
        let s = JointState { angle: 0.0, angular_velocity: 0.0 };
        let applied = AppliedTorques { muscle: 4.0, ..Default::default() };
        let out = integrate_joint(&j, s, applied, 1.0, 0.05);
        // α = 4. velocity = 0 + 4 × 0.05 = 0.2.
        assert!((out.angular_velocity - 0.2).abs() < 1e-6);
    }

    #[test]
    fn rom_clamp_stops_velocity_at_upper_bound() {
        let j = make_hinge(0.0, 0.0, 0.5);
        // Already at upper limit, moving outward.
        let s = JointState { angle: 0.5, angular_velocity: 1.0 };
        let out = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.1);
        assert!((out.angle - 0.5).abs() < 1e-6);
        assert_eq!(out.angular_velocity, 0.0);
    }

    #[test]
    fn rom_clamp_keeps_inward_velocity() {
        let j = make_hinge(0.0, 0.0, 0.5);
        // At upper limit but moving inward — the integrator must let it.
        let s = JointState { angle: 0.5, angular_velocity: -1.0 };
        let out = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.1);
        assert!(out.angle < 0.5);
        assert_eq!(out.angular_velocity, -1.0);
    }

    #[test]
    fn long_simulation_settles_to_rest_under_friction() {
        // Released from a deflected position with no muscle drive: should
        // damp toward rest_angle = 0 over time.
        let j = make_hinge(8.0, 0.5, 5.0);
        let mut s = JointState { angle: 0.4, angular_velocity: 0.0 };
        for _ in 0..2000 {
            s = integrate_joint(&j, s, AppliedTorques::default(), 1.0, 0.01);
        }
        assert!(s.angle.abs() < 0.01, "expected near-rest angle, got {}", s.angle);
        assert!(s.angular_velocity.abs() < 0.05);
    }

    #[test]
    fn bone_inertia_uses_effective_length_and_mass() {
        let mut b = Bone::root("test", 6.0, 1.0);
        b.tissue.mass = 2.0;
        // Effective length = 6 (scale 1.0). I = (1/3) × 2 × 36 = 24.
        assert!((bone_inertia(&b) - 24.0).abs() < 1e-6);
    }
}
