//! Z-order draw scheduling.
//!
//! Pixel-art frontal sprites need explicit occlusion: the right arm renders
//! in front of the body, the left arm behind, the head over both, the ears
//! above the head, etc. Each `Bone` carries a `z_layer` (`-2..=+2`); this
//! module turns that into a stable draw order.
//!
//! ## Why this is its own module
//!
//! `Skeleton` keeps no opinion about drawing — it only does kinematics. The
//! draw order is a presentation concern that lives next to but outside the
//! kinematic core, so this module stays small and easy to swap (e.g., for a
//! 3/4 view that needs y-position-based sorting later).

use crate::bone::BoneId;
use crate::skeleton::Skeleton;

/// Returns bone ids in the order they should be painted: ascending `z_layer`,
/// ties broken by the bone's index in the skeleton (which itself follows
/// topological order — parents before children, naturally adequate for
/// most sibling overlap when z_layer ties).
///
/// Allocates a `Vec` per call. For a skeleton with ~30 bones this is
/// trivial; if it ever shows up in profiles, the caller can reuse a
/// scratch buffer.
pub fn draw_order(skeleton: &Skeleton) -> Vec<BoneId> {
    let mut ids: Vec<(usize, i8)> = skeleton
        .bones()
        .iter()
        .enumerate()
        .map(|(i, b)| (i, b.z_layer))
        .collect();
    // Stable sort: lower z first; if z is equal, lower index first.
    ids.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    ids.into_iter().map(|(i, _)| BoneId(i as u16)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::{Bone, Vec2};

    #[test]
    fn equal_z_uses_skeleton_order() {
        let bones = vec![
            Bone::root("root", 1.0, 1.0),
            Bone::child("a", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0),
            Bone::child("b", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0),
        ];
        let sk = Skeleton::new(bones);
        let order = draw_order(&sk);
        assert_eq!(order, vec![BoneId(0), BoneId(1), BoneId(2)]);
    }

    #[test]
    fn negative_z_renders_first() {
        // arm_back z=-1 should paint before body z=0 should paint before
        // arm_front z=+1.
        let bones = vec![
            Bone::root("body", 1.0, 1.0).with_z(0),
            Bone::child("arm_back", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(-1),
            Bone::child("arm_front", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(1),
        ];
        let sk = Skeleton::new(bones);
        let order = draw_order(&sk);
        let names: Vec<&str> = order.iter().map(|id| sk.bone(*id).name).collect();
        assert_eq!(names, vec!["arm_back", "body", "arm_front"]);
    }

    #[test]
    fn full_creature_layering() {
        // Realistic example: pelvis (z=0), left arm behind (z=-1),
        // body (z=0), right arm in front (z=+1), head (z=+1), ears (z=+2).
        let bones = vec![
            Bone::root("pelvis", 1.0, 1.0),
            Bone::child("arm_l", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(-1),
            Bone::child("torso", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0),
            Bone::child("arm_r", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(1),
            Bone::child("head", BoneId(2), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(1),
            Bone::child("ear", BoneId(4), Vec2::ZERO, 0.0, 1.0, 1.0).with_z(2),
        ];
        let sk = Skeleton::new(bones);
        let order = draw_order(&sk);
        let names: Vec<&str> = order.iter().map(|id| sk.bone(*id).name).collect();
        assert_eq!(names, vec!["arm_l", "pelvis", "torso", "arm_r", "head", "ear"]);
    }

    #[test]
    fn empty_skeleton_returns_empty_order() {
        // Single root only — should still return a single-element list.
        let bones = vec![Bone::root("only", 1.0, 1.0)];
        let sk = Skeleton::new(bones);
        assert_eq!(draw_order(&sk), vec![BoneId(0)]);
    }
}
