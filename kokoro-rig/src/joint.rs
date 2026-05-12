//! Joints — where two bones meet and physical motion happens.
//!
//! In Kokoro the **bone** is a rigid structural element; all flexibility
//! lives in the **joint** between bones. A joint owns the rest angle,
//! the range-of-motion limits, the passive ligament spring back to neutral,
//! and the surface mechanics (friction today, cartilage condition + fluid
//! later) that resist motion. The universal physics integrator in
//! `physics` reads these per-joint parameters and advances each joint's
//! [`JointState`] by `dt`.
//!
//! ## Why split Joint from Bone
//!
//! Real anatomy splits responsibility this way: the bone gives shape and
//! lever arm, the joint surface determines how the bones articulate, and
//! the surrounding ligaments / cartilage / synovial-fluid determine the
//! mechanical response. Folding all of these into [`Bone`](crate::Bone)
//! would conflate joint pathology (worn cartilage, torn ligaments) with
//! bone pathology (density, fracture).
//!
//! ## Forward-compatibility
//!
//! [`JointSurface`] and [`JointKind`] are narrow today (friction + ROM,
//! single hinge type in 2D) but the names hold space for cartilage
//! condition, synovial fluid pressure, ball-and-socket math, etc. Adding
//! fields to these structs is a forward-compatible change.

use crate::bone::BoneId;

/// Kind of articulation between two bones. Today only `Hinge` participates
/// in physics integration; `Ball` is reserved for future 3D rigs and
/// `Fused` short-circuits the integrator so rigid couplings cost nothing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JointKind {
    /// One rotational degree of freedom in the plane.
    Hinge,
    /// Reserved — 3-DoF rotation. Not implemented in the 2D integrator.
    Ball,
    /// Bones are rigidly fused — no relative motion, no integration cost.
    Fused,
}

/// Mechanical condition of the joint contact surface. Currently a single
/// dimensionless friction coefficient that resists angular velocity at the
/// joint. Future expansion will add `cartilage_condition` and
/// `synovial_fluid` fields here to model wear and lubrication —
/// downstream code accesses these via the struct so adding fields is a
/// forward-compatible change.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointSurface {
    /// Damping coefficient, N·m·s/rad. Multiplied by angular velocity to
    /// produce the friction torque each integration step.
    pub friction: f32,
}

impl JointSurface {
    pub const fn new(friction: f32) -> Self {
        Self { friction }
    }
}

impl Default for JointSurface {
    fn default() -> Self {
        Self { friction: 0.0 }
    }
}

/// Static description of a joint. Owns the mechanical parameters; the
/// time-varying `angle` and `angular_velocity` live in [`JointState`] so
/// the integrator can read constants and write state without aliasing.
#[derive(Clone, Debug)]
pub struct Joint {
    /// The child bone this joint controls. Its parent in the skeleton is
    /// the joint's "other side".
    pub bone: BoneId,

    /// Anatomical kind of articulation.
    pub kind: JointKind,

    /// Resting angle the ligaments pull toward, in radians, relative to
    /// the parent's world angle. Typically matches `Bone::rest_angle`;
    /// stored here so the joint can be reset/healed without touching
    /// the bone.
    pub rest_angle: f32,

    /// Hard ROM limit (radians, relative to `rest_angle`). The joint is
    /// clamped to `[rest_angle + range_min, rest_angle + range_max]` —
    /// past those it cannot move regardless of how strong the muscle is.
    pub range_min: f32,
    pub range_max: f32,

    /// Ligament spring constant (N·m/rad). Higher = stronger pull back
    /// to `rest_angle`. Zero = floppy (cat tail). Very high = rigid spine.
    pub ligament_k: f32,

    /// Surface mechanics — friction today, plus future fields.
    pub surface: JointSurface,
}

impl Joint {
    /// Hinge constructor — the 2D-rig default.
    pub const fn hinge(
        bone: BoneId,
        rest_angle: f32,
        range_min: f32,
        range_max: f32,
        ligament_k: f32,
        friction: f32,
    ) -> Self {
        Self {
            bone,
            kind: JointKind::Hinge,
            rest_angle,
            range_min,
            range_max,
            ligament_k,
            surface: JointSurface { friction },
        }
    }

    /// Fused joint — no degrees of freedom, no integration. Used when two
    /// bones should never move relative to each other (e.g., skull plates).
    pub const fn fused(bone: BoneId) -> Self {
        Self {
            bone,
            kind: JointKind::Fused,
            rest_angle: 0.0,
            range_min: 0.0,
            range_max: 0.0,
            ligament_k: 0.0,
            surface: JointSurface { friction: 0.0 },
        }
    }

    /// True iff this joint participates in physics integration.
    #[inline]
    pub fn is_active(&self) -> bool {
        matches!(self.kind, JointKind::Hinge | JointKind::Ball)
    }
}

/// Mutable per-joint physical state — updated each `dt` by the integrator.
/// Kept separate from [`Joint`] so the static description and the
/// time-varying state can be sliced independently.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointState {
    /// Current angle in radians, in the same frame as `Joint::rest_angle`.
    pub angle: f32,
    /// Current angular velocity in rad/s.
    pub angular_velocity: f32,
}

impl JointState {
    /// Construct a state parked at `angle` with zero velocity.
    pub const fn at_rest(angle: f32) -> Self {
        Self {
            angle,
            angular_velocity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::BoneId;

    #[test]
    fn hinge_constructor_sets_kind_and_surface() {
        let j = Joint::hinge(BoneId(1), 0.0, -1.0, 1.0, 5.0, 0.2);
        assert_eq!(j.kind, JointKind::Hinge);
        assert!(j.is_active());
        assert!((j.surface.friction - 0.2).abs() < 1e-6);
    }

    #[test]
    fn fused_joint_is_not_active() {
        let j = Joint::fused(BoneId(1));
        assert_eq!(j.kind, JointKind::Fused);
        assert!(!j.is_active());
    }

    #[test]
    fn at_rest_state_has_zero_velocity() {
        let s = JointState::at_rest(0.3);
        assert!((s.angle - 0.3).abs() < 1e-6);
        assert_eq!(s.angular_velocity, 0.0);
    }
}
