//! FABRIK — Forward And Backward Reaching Inverse Kinematics.
//!
//! Iterative IK solver for many-segment chains where an analytical solution
//! is impractical: tails, tentacles, antennae, anything that bends as a
//! flexible string instead of a hinged limb. Converges in 2-10 iterations
//! for typical pixel-art chains (3-7 joints).
//!
//! Reference: Aristidou & Lasenby (2011), "FABRIK: A fast, iterative solver
//! for the Inverse Kinematics problem." We implement the basic version
//! without joint-angle constraints — for an N-segment tail bending freely
//! that's enough; if tentacle joint limits become a problem later we can
//! layer constraints on top.
//!
//! ## Geometry model
//!
//! - `joints` = positions in world space. There are `N` joints.
//! - `segments[i]` = distance between joint `i` and joint `i+1`. There are
//!   `N-1` segments.
//! - `root_anchor` = fixed world position the root joint snaps back to on
//!   every iteration (the chain "hangs from" this point).
//!
//! After `solve()`, every adjacent pair `(joints[i], joints[i+1])` will be
//! at distance `segments[i]`, joint `0` will be at `root_anchor`, and the
//! tip will be at the target (or as close as the chain can reach).

use crate::bone::Vec2;

/// A flexible chain solved by FABRIK.
#[derive(Clone, Debug)]
pub struct FabrikChain {
    /// Joint positions in world space. `joints[0]` is the root, `joints[len-1]` the tip.
    pub joints: Vec<Vec2>,

    /// Per-segment rest lengths. Always `joints.len() - 1` long.
    pub segments: Vec<f32>,

    /// World position the root snaps to on every backward pass.
    pub root_anchor: Vec2,
}

impl FabrikChain {
    /// Create a chain. Panics in debug builds if `segments.len() != joints.len() - 1`.
    pub fn new(joints: Vec<Vec2>, segments: Vec<f32>, root_anchor: Vec2) -> Self {
        debug_assert_eq!(
            segments.len() + 1,
            joints.len(),
            "FabrikChain expects exactly N-1 segments for N joints"
        );
        Self { joints, segments, root_anchor }
    }

    /// Build a straight chain pointing along +x from the root, with the
    /// given per-segment lengths. Convenience for skeletons that haven't
    /// been solved yet.
    pub fn straight(root_anchor: Vec2, segments: Vec<f32>) -> Self {
        let mut joints = Vec::with_capacity(segments.len() + 1);
        joints.push(root_anchor);
        let mut cursor = root_anchor;
        for &len in &segments {
            cursor = Vec2::new(cursor.x + len, cursor.y);
            joints.push(cursor);
        }
        Self { joints, segments, root_anchor }
    }

    /// Total reach when fully extended.
    pub fn total_length(&self) -> f32 {
        self.segments.iter().sum()
    }

    /// Solve the chain so that `joints.last()` lands at `target`. Returns
    /// `true` if the tip reached the target within `tolerance`, `false`
    /// otherwise (target out of reach, or converged short of tolerance
    /// within `max_iters`).
    ///
    /// Guarantees on return:
    /// - `joints[0] == root_anchor`
    /// - segment lengths are preserved (within float epsilon)
    pub fn solve(&mut self, target: Vec2, tolerance: f32, max_iters: u32) -> bool {
        let n = self.joints.len();
        if n < 2 {
            return true;
        }

        let total = self.total_length();
        let to_target = self.root_anchor.distance(target);

        // Out of reach: stretch the chain straight from root toward target,
        // preserving segment lengths but accepting that the tip stops short.
        if to_target > total {
            self.joints[0] = self.root_anchor;
            let dir = target.sub(self.root_anchor);
            let dir_len = dir.length().max(1e-6);
            let unit = Vec2::new(dir.x / dir_len, dir.y / dir_len);
            for i in 1..n {
                let prev = self.joints[i - 1];
                self.joints[i] = Vec2::new(
                    prev.x + unit.x * self.segments[i - 1],
                    prev.y + unit.y * self.segments[i - 1],
                );
            }
            return false;
        }

        // Reachable: alternate forward and backward passes until the tip
        // lands within tolerance of the target or we hit max_iters. The
        // `iter` index is mixed into the degenerate-direction fallback so
        // collapsed joints don't lock into a fixed direction across passes.
        for iter in 0..max_iters {
            // Forward pass: pin the tip on the target, walk back toward root
            // re-projecting each joint to its rest distance from the next.
            self.joints[n - 1] = target;
            for i in (0..n - 1).rev() {
                let next = self.joints[i + 1];
                let cur = self.joints[i];
                let unit = unit_or_fallback(cur.sub(next), iter, i);
                self.joints[i] = Vec2::new(
                    next.x + unit.x * self.segments[i],
                    next.y + unit.y * self.segments[i],
                );
            }

            // Backward pass: pin the root, walk out toward the tip
            // re-projecting each joint to its rest distance from the previous.
            self.joints[0] = self.root_anchor;
            for i in 0..n - 1 {
                let cur = self.joints[i];
                let next = self.joints[i + 1];
                let unit = unit_or_fallback(next.sub(cur), iter, i);
                self.joints[i + 1] = Vec2::new(
                    cur.x + unit.x * self.segments[i],
                    cur.y + unit.y * self.segments[i],
                );
            }

            if self.joints[n - 1].distance(target) < tolerance {
                return true;
            }
        }

        false
    }
}

/// Normalise `delta` to unit length, falling back to a deterministic but
/// non-degenerate direction when the input is essentially zero. This keeps
/// FABRIK from dividing by zero (and from oscillating in place) when two
/// adjacent joints momentarily collapse onto the same point.
///
/// The fallback rotates by `iter + i` so successive passes try slightly
/// different directions, breaking any local-minimum lockup.
#[inline]
fn unit_or_fallback(delta: Vec2, iter: u32, i: usize) -> Vec2 {
    let len = delta.length();
    if len > 1e-5 {
        return Vec2::new(delta.x / len, delta.y / len);
    }
    // Cycle through 4 cardinal directions over iterations + joint index.
    let phase = (iter as usize + i) & 0b11;
    match phase {
        0 => Vec2::new(1.0, 0.0),
        1 => Vec2::new(0.0, 1.0),
        2 => Vec2::new(-1.0, 0.0),
        _ => Vec2::new(0.0, -1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn segments_preserved(chain: &FabrikChain, eps: f32) -> bool {
        chain.joints.windows(2).enumerate().all(|(i, pair)| {
            approx(pair[0].distance(pair[1]), chain.segments[i], eps)
        })
    }

    #[test]
    fn straight_chain_constructor_lays_joints_along_x() {
        let chain = FabrikChain::straight(Vec2::new(10.0, 20.0), vec![3.0, 4.0, 2.0]);
        assert_eq!(chain.joints.len(), 4);
        assert_eq!(chain.joints[0], Vec2::new(10.0, 20.0));
        assert_eq!(chain.joints[1], Vec2::new(13.0, 20.0));
        assert_eq!(chain.joints[2], Vec2::new(17.0, 20.0));
        assert_eq!(chain.joints[3], Vec2::new(19.0, 20.0));
        assert!(segments_preserved(&chain, 1e-4));
    }

    #[test]
    fn three_segment_chain_reaches_reachable_target() {
        let mut chain = FabrikChain::straight(Vec2::ZERO, vec![5.0, 5.0, 5.0]);
        let target = Vec2::new(8.0, 6.0);
        let ok = chain.solve(target, 1e-3, 20);
        assert!(ok, "should converge");
        assert!(approx(chain.joints[3].distance(target), 0.0, 1e-3));
        assert!(segments_preserved(&chain, 1e-3));
        // Root stays pinned.
        assert_eq!(chain.joints[0], Vec2::ZERO);
    }

    #[test]
    fn five_segment_chain_wraps_around_close_target() {
        // 5 segments × 3px = 15px reach. Target very close (2px from root)
        // → chain has to fold/wrap. The initial state is intentionally a
        // gentle arc rather than perfectly colinear: a perfectly straight
        // chain with the target ON its axis is a degenerate FABRIK case
        // (no gradient to break symmetry, oscillates forever). In practice
        // soft-body simulation never feeds the rig a perfectly colinear
        // chain so this is a realistic starting configuration.
        let root = Vec2::new(20.0, 20.0);
        let segments = vec![3.0; 5];
        let joints = vec![
            root,
            Vec2::new(23.0, 20.0),
            Vec2::new(25.5, 21.5),  // bend up
            Vec2::new(27.0, 24.0),
            Vec2::new(28.0, 26.5),
            Vec2::new(28.5, 29.0),
        ];
        let mut chain = FabrikChain::new(joints, segments, root);
        let target = Vec2::new(22.0, 20.0);
        let ok = chain.solve(target, 1e-3, 50);
        assert!(ok, "wrapping chain should converge");
        assert!(approx(chain.joints[5].distance(target), 0.0, 1e-3));
        assert!(segments_preserved(&chain, 1e-3));
        assert_eq!(chain.joints[0], root);
    }

    #[test]
    fn out_of_range_target_returns_false_but_stays_straight() {
        let mut chain = FabrikChain::straight(Vec2::ZERO, vec![2.0, 2.0, 2.0]);
        let target = Vec2::new(100.0, 0.0);
        let ok = chain.solve(target, 1e-3, 20);
        assert!(!ok);
        // Chain points straight at target along +x.
        for joint in &chain.joints {
            assert!(approx(joint.y, 0.0, 1e-4), "y should be 0, got {}", joint.y);
        }
        assert!(segments_preserved(&chain, 1e-4));
        // Tip ends up at distance == total_length from root, NOT at target.
        let total = chain.total_length();
        assert!(approx(chain.joints[3].x, total, 1e-4));
    }

    #[test]
    fn diagonal_target_reachable() {
        let mut chain = FabrikChain::straight(Vec2::new(32.0, 32.0), vec![4.0, 4.0, 4.0, 4.0]);
        let target = Vec2::new(40.0, 40.0);
        let ok = chain.solve(target, 1e-3, 30);
        assert!(ok);
        assert!(approx(chain.joints[4].distance(target), 0.0, 1e-3));
        assert!(segments_preserved(&chain, 1e-3));
    }

    #[test]
    fn root_anchor_pulls_root_back_after_solve() {
        // Even if the initial joints[0] is somewhere else, after solve()
        // joints[0] must equal root_anchor.
        let chain = FabrikChain::new(
            vec![Vec2::new(99.0, 99.0), Vec2::new(99.0, 99.0), Vec2::new(99.0, 99.0)],
            vec![3.0, 3.0],
            Vec2::new(5.0, 5.0),
        );
        let mut chain = chain;
        let _ = chain.solve(Vec2::new(8.0, 6.0), 1e-3, 20);
        assert_eq!(chain.joints[0], Vec2::new(5.0, 5.0));
    }

    #[test]
    fn two_joint_chain_is_a_no_op_when_target_at_tip() {
        let mut chain = FabrikChain::straight(Vec2::ZERO, vec![5.0]);
        let original_tip = chain.joints[1];
        let ok = chain.solve(original_tip, 1e-3, 5);
        assert!(ok);
        assert_eq!(chain.joints[1], original_tip);
    }

    #[test]
    fn very_close_target_at_root_does_not_crash() {
        // Edge: target sits ON the root. A 3-segment chain can only fold
        // back; tip distance won't reach 0 (segments don't allow), so we
        // expect non-convergence but stable output.
        let mut chain = FabrikChain::straight(Vec2::new(10.0, 10.0), vec![3.0, 3.0, 3.0]);
        let _ = chain.solve(Vec2::new(10.0, 10.0), 1e-3, 30);
        // Root pinned, segments still consistent.
        assert_eq!(chain.joints[0], Vec2::new(10.0, 10.0));
        assert!(segments_preserved(&chain, 1e-3));
    }
}
