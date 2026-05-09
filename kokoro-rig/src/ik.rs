//! Inverse kinematics for the rig.
//!
//! ## What lives here
//!
//! - `solve_two_bone` — analytical IK for a 2-bone chain (hip → knee → foot,
//!   or shoulder → elbow → hand). Single-pass, deterministic, ~30 lines of
//!   trig.
//!
//! Future modules (separate commits) will add a FABRIK iterative solver for
//! many-segment chains like tails and tentacles, where an analytical
//! solution is impractical.
//!
//! ## Why IK matters here
//!
//! Without IK, when the soft-body simulation drags a foot point sideways,
//! the leg in the sprite stays straight and the foot detaches from the body
//! visually. With IK we say "given that the foot is *here*, what hip + knee
//! angles make a connected leg reach it?" — answer those, render bones
//! anchored at the hip, and the leg looks intentionally posed.

use crate::bone::Vec2;

/// Result of solving a 2-bone chain.
///
/// `upper_world_angle` and `lower_local_angle` go straight into
/// `Skeleton::set_angle` for the upper and lower bones respectively. The
/// upper bone's angle is *world*-relative because, by convention, the hip
/// is the chain's root in the rig (its parent doesn't apply rotation to it
/// for IK purposes); the lower bone's angle is *parent-relative*, matching
/// `Bone::rest_angle` semantics.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TwoBoneResult {
    pub upper_world_angle: f32,
    pub lower_local_angle: f32,
    /// `true` if the chain can actually reach the target. `false` when the
    /// target is too far (overflow) or too close (underflow); in both
    /// fallback cases the chain is laid out straight toward the target so
    /// the foot lands as close as possible without leaving the body broken.
    pub reached: bool,
}

/// Which side the knee/elbow should bend to. Positive bends one way, negative
/// the other. The choice is purely cosmetic for a single solve, but staying
/// consistent across frames prevents the leg from "popping" between
/// configurations when the target wobbles past the straight-leg line.
///
/// Convention: with `Positive`, looking from hip toward target, the knee
/// bends to the **right** of the hip→target line in image-space (where +y
/// is down). For a creature standing upright with feet below the hip and
/// knees pointing forward, `Positive` is "knee bends toward viewer".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BendDirection {
    Positive,
    Negative,
}

impl BendDirection {
    #[inline]
    fn sign(self) -> f32 {
        match self {
            BendDirection::Positive => 1.0,
            BendDirection::Negative => -1.0,
        }
    }
}

/// Analytical 2-bone IK using the law of cosines.
///
/// Given the hip (root) world position, an end-effector target, and the two
/// bone lengths, compute the upper bone's world rotation and the lower
/// bone's local rotation that place the chain's tip exactly at the target.
///
/// Math:
/// 1. `d = distance(hip, target)`
/// 2. If `d ≥ upper + lower`, the leg is too short — lay it straight toward
///    target, mark `reached = false`.
/// 3. If `d ≤ |upper - lower|`, the leg is too folded to reach — same
///    fallback (straight pose, unreached).
/// 4. Otherwise `cosα = (upper² + d² - lower²) / (2·upper·d)` gives the
///    angle at the hip between the upper bone and the hip→target line; the
///    upper bone's world angle is `direction(hip→target) ± α` per `bend`.
/// 5. `cosβ = (upper² + lower² - d²) / (2·upper·lower)` gives the interior
///    knee angle; the lower bone's local angle is `±(π - β)` so that
///    `lower_local = 0` corresponds to "fully extended" and `±π` to
///    "folded back on itself".
pub fn solve_two_bone(
    hip: Vec2,
    target: Vec2,
    upper: f32,
    lower: f32,
    bend: BendDirection,
) -> TwoBoneResult {
    let delta = target.sub(hip);
    let d = delta.length();
    let to_target = delta.angle(); // direction hip → target in world

    let max_reach = upper + lower;
    let min_reach = (upper - lower).abs();

    // Out-of-range fallbacks: lay the chain straight toward the target.
    // The foot won't land where the caller asked, but the visual chain
    // stays connected and pointing the right way.
    if d >= max_reach || d <= min_reach || upper <= 0.0 || lower <= 0.0 {
        return TwoBoneResult {
            upper_world_angle: to_target,
            lower_local_angle: 0.0,
            reached: false,
        };
    }

    // Cosine law for the hip-side angle (between upper bone and hip→target).
    let cos_alpha = ((upper * upper + d * d - lower * lower) / (2.0 * upper * d))
        .clamp(-1.0, 1.0);
    let alpha = cos_alpha.acos();

    // Cosine law for the interior knee angle.
    let cos_beta = ((upper * upper + lower * lower - d * d) / (2.0 * upper * lower))
        .clamp(-1.0, 1.0);
    let beta = cos_beta.acos();

    let s = bend.sign();
    let upper_world_angle = to_target + s * alpha;
    // Local lower angle: 0 = fully extended; sign(π - β) bends the knee
    // opposite to alpha so the chain folds toward the same side either way.
    let lower_local_angle = -s * (std::f32::consts::PI - beta);

    TwoBoneResult {
        upper_world_angle,
        lower_local_angle,
        reached: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    /// Reconstruct where the foot ends up given the IK angles, so we can
    /// verify the chain actually closes on the target. Mirrors what the
    /// `Skeleton::forward` pass would compute for this two-bone subtree.
    fn forward_kinematics(
        hip: Vec2,
        upper: f32,
        lower: f32,
        upper_world: f32,
        lower_local: f32,
    ) -> Vec2 {
        let knee = Vec2::new(
            hip.x + upper * upper_world.cos(),
            hip.y + upper * upper_world.sin(),
        );
        let lower_world = upper_world + lower_local;
        Vec2::new(
            knee.x + lower * lower_world.cos(),
            knee.y + lower * lower_world.sin(),
        )
    }

    #[test]
    fn reachable_target_lands_exactly() {
        let hip = Vec2::new(10.0, 10.0);
        let target = Vec2::new(15.0, 18.0);
        let r = solve_two_bone(hip, target, 6.0, 6.0, BendDirection::Positive);
        assert!(r.reached);

        let foot = forward_kinematics(hip, 6.0, 6.0, r.upper_world_angle, r.lower_local_angle);
        assert!(
            approx(foot.x, target.x, 1e-3) && approx(foot.y, target.y, 1e-3),
            "foot {foot:?} should match target {target:?}"
        );
    }

    #[test]
    fn bend_direction_chooses_opposite_knee() {
        // Same target, opposite bends → knees on opposite sides of the
        // hip→target line, but BOTH foot positions still land on target.
        let hip = Vec2::new(0.0, 0.0);
        let target = Vec2::new(8.0, 0.0);
        let pos = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Positive);
        let neg = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Negative);

        assert!(pos.reached && neg.reached);
        // Upper world angles mirror across the hip→target line (= 0 here).
        assert!(approx(pos.upper_world_angle, -neg.upper_world_angle, 1e-3));
        // Both reach the target.
        let foot_pos = forward_kinematics(hip, 5.0, 5.0, pos.upper_world_angle, pos.lower_local_angle);
        let foot_neg = forward_kinematics(hip, 5.0, 5.0, neg.upper_world_angle, neg.lower_local_angle);
        assert!(approx(foot_pos.x, 8.0, 1e-3) && approx(foot_pos.y, 0.0, 1e-3));
        assert!(approx(foot_neg.x, 8.0, 1e-3) && approx(foot_neg.y, 0.0, 1e-3));
    }

    #[test]
    fn target_out_of_reach_falls_back_straight() {
        let hip = Vec2::ZERO;
        // Target 30 units away but chain only reaches 10.
        let target = Vec2::new(30.0, 0.0);
        let r = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Positive);
        assert!(!r.reached);
        // Straight chain pointing at target ⇒ knee local 0, hip world 0.
        assert!(approx(r.upper_world_angle, 0.0, 1e-4));
        assert!(approx(r.lower_local_angle, 0.0, 1e-4));
    }

    #[test]
    fn target_too_close_falls_back_straight() {
        let hip = Vec2::ZERO;
        // a=5, b=8 → min reach = 3. Target at d=2 is inside the dead zone.
        let target = Vec2::new(2.0, 0.0);
        let r = solve_two_bone(hip, target, 5.0, 8.0, BendDirection::Positive);
        assert!(!r.reached);
    }

    #[test]
    fn straight_extension_at_max_reach() {
        // Right at the edge: chain fully extended toward the target.
        let hip = Vec2::ZERO;
        let target = Vec2::new(10.0, 0.0);
        let r = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Positive);
        // d == max_reach is treated as out-of-range to keep math safe.
        assert!(!r.reached);
        assert!(approx(r.upper_world_angle, 0.0, 1e-4));
        assert!(approx(r.lower_local_angle, 0.0, 1e-4));
    }

    #[test]
    fn known_symmetric_case() {
        // a=b=5, target (6, 0). By symmetry the knee is directly above or
        // below the midpoint at (3, ±4) — a 3-4-5 triangle on each side.
        let hip = Vec2::ZERO;
        let target = Vec2::new(6.0, 0.0);
        let r = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Positive);
        assert!(r.reached);
        // Hip angle α = acos((25 + 36 - 25) / (2·5·6)) = acos(0.6) ≈ 0.9273.
        let expected_alpha = (0.6_f32).acos();
        assert!(approx(r.upper_world_angle, expected_alpha, 1e-3));
        // Closes on the target.
        let foot = forward_kinematics(hip, 5.0, 5.0, r.upper_world_angle, r.lower_local_angle);
        assert!(approx(foot.x, 6.0, 1e-3) && approx(foot.y, 0.0, 1e-3));
    }

    #[test]
    fn full_extension_when_target_at_max_minus_epsilon() {
        // Just inside max reach — solver should produce a nearly-straight
        // leg with tiny bend.
        let hip = Vec2::ZERO;
        let target = Vec2::new(9.99, 0.0);
        let r = solve_two_bone(hip, target, 5.0, 5.0, BendDirection::Positive);
        assert!(r.reached);
        // Hip angle and knee local should both be small.
        assert!(r.upper_world_angle.abs() < 0.1);
        assert!(r.lower_local_angle.abs() < 0.2);
    }

    #[test]
    fn zero_length_bone_returns_unreached() {
        // Degenerate input: a 0-length upper bone. Don't divide by zero;
        // return the straight-fallback result.
        let r = solve_two_bone(Vec2::ZERO, Vec2::new(3.0, 0.0), 0.0, 5.0, BendDirection::Positive);
        assert!(!r.reached);
        assert!(approx(r.upper_world_angle, 0.0, 1e-4));
    }

    #[test]
    fn diagonal_target_reachable() {
        let hip = Vec2::new(32.0, 32.0);
        let target = Vec2::new(38.0, 41.0);
        let r = solve_two_bone(hip, target, 7.0, 7.0, BendDirection::Negative);
        assert!(r.reached);
        let foot = forward_kinematics(hip, 7.0, 7.0, r.upper_world_angle, r.lower_local_angle);
        assert!(
            approx(foot.x, target.x, 1e-3) && approx(foot.y, target.y, 1e-3),
            "foot {foot:?} should match target {target:?}"
        );
    }

}
