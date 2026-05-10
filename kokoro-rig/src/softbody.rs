//! Bridge between the rigid skeleton and a flexible soft-body point cloud.
//!
//! ## The contract (Hybrid — option C)
//!
//! Each bone declares its `Stiffness`:
//!
//! - **Rigid** bones: skeleton drives. After FK, the bone's tip is *pushed*
//!   into the soft-body point cloud (overriding wherever physics had drifted
//!   it). Use for skull, torso, mantle — parts that must hold their shape.
//! - **Soft** bones: soft body drives. Before FK, the bone's angle is
//!   *derived* from the matching soft-body point so the bone tip lands on
//!   it. Use for tail tips, tentacles, ear tips — parts that should drift
//!   with simulation.
//!
//! Bone↔point matching is by **name**: a bone called `"tail_3"` syncs with
//! the soft-body point called `"tail_3"`. Bones whose name has no matching
//! point are silently skipped — lets you author the skeleton independently
//! of the soft-body rig and add points later.
//!
//! ## Why a trait, not a concrete type
//!
//! The soft-body simulation lives in the main `kokoro` crate (Bevy
//! resource), but `kokoro-rig` is Bevy-free. The `SoftBodyPoints` trait is
//! the seam: in production code, the Bevy `SoftBody` resource impls this
//! trait by name lookup; in unit tests, a tiny `HashMap` impl stands in.

use crate::bone::{BoneId, Stiffness, Vec2};
use crate::skeleton::Skeleton;

/// Read/write access to a named point cloud. The rig calls into this trait
/// once per `reconcile()` to exchange world positions with the simulator.
pub trait SoftBodyPoints {
    /// Look up a point's world-space position by name. Returns `None` if
    /// the simulator doesn't know about a point with that name.
    fn point(&self, name: &str) -> Option<Vec2>;

    /// Write a point's world-space position. No-op if the simulator
    /// doesn't track a point of that name (the rig won't crash if a Rigid
    /// bone is attached to a non-existent point).
    fn set_point(&mut self, name: &str, pos: Vec2);
}

/// Synchronise a skeleton with a soft-body point cloud per the Hybrid
/// contract. Call once per frame, after the soft-body simulation step and
/// after the pose layers have been applied.
///
/// ### Order of operations
///
/// 1. If the skeleton is dirty, run `forward()` so we have fresh world
///    positions to compare against.
/// 2. **First scan**: for each bone…
///    - `Rigid` → push its world tip into the soft body (skeleton wins).
///    - `Soft` → read the soft-body point, compute the local angle that
///      makes the bone reach it, queue the angle update.
/// 3. If any soft updates were queued, apply them and re-FK once so
///    children of soft bones see the new world positions.
///
/// The two-pass structure is deliberate: applying a soft update mid-loop
/// would invalidate the world positions of subsequent siblings without
/// re-computing FK every iteration — wasteful for typical rigs where most
/// bones are Rigid and only a handful are Soft.
pub fn reconcile(skeleton: &mut Skeleton, soft: &mut dyn SoftBodyPoints) {
    if skeleton.dirty() {
        skeleton.forward();
    }

    let n = skeleton.len();
    let mut updates: Vec<(BoneId, f32)> = Vec::new();

    for i in 0..n {
        let id = BoneId(i as u16);
        // Snapshot the bone's identity-relevant fields so we can release
        // the skeleton borrow before calling `soft.set_point` (which takes
        // `&mut self`) or other skeleton methods.
        let (name, stiffness, parent) = {
            let b = skeleton.bone(id);
            (b.name, b.stiffness, b.parent)
        };

        match stiffness {
            Stiffness::Rigid => {
                let tip = skeleton.world_tip(id);
                soft.set_point(name, tip);
            }
            Stiffness::Soft => {
                if let Some(target) = soft.point(name) {
                    let base = skeleton.world_base(id);
                    let world_angle = target.sub(base).angle();
                    let parent_world = parent
                        .map(|p| skeleton.world_angle(p))
                        .unwrap_or(0.0);
                    updates.push((id, world_angle - parent_world));
                }
            }
        }
    }

    if !updates.is_empty() {
        for (id, local_angle) in updates {
            skeleton.set_angle(id, local_angle);
        }
        skeleton.forward();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bone::Bone;
    use std::collections::HashMap;
    use std::f32::consts::PI;

    /// Tiny in-memory soft body: a bag of named points. Implements
    /// `SoftBodyPoints` so tests don't need Bevy.
    #[derive(Default)]
    struct MockSoftBody {
        points: HashMap<String, Vec2>,
    }

    impl MockSoftBody {
        fn with(&mut self, name: &str, pos: Vec2) -> &mut Self {
            self.points.insert(name.to_string(), pos);
            self
        }
    }

    impl SoftBodyPoints for MockSoftBody {
        fn point(&self, name: &str) -> Option<Vec2> {
            self.points.get(name).copied()
        }
        fn set_point(&mut self, name: &str, pos: Vec2) {
            self.points.insert(name.to_string(), pos);
        }
    }

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn approx_vec(a: Vec2, b: Vec2, eps: f32) -> bool {
        approx(a.x, b.x, eps) && approx(a.y, b.y, eps)
    }

    #[test]
    fn rigid_bone_pushes_tip_into_softbody() {
        let bones = vec![Bone::root("root", 5.0, 1.0)]; // default Rigid
        let mut sk = Skeleton::new(bones);
        sk.set_root(Vec2::new(10.0, 10.0));
        let mut soft = MockSoftBody::default();
        reconcile(&mut sk, &mut soft);
        // Tip = root_pos + (length, 0) = (15, 10).
        assert_eq!(soft.point("root"), Some(Vec2::new(15.0, 10.0)));
    }

    #[test]
    fn soft_bone_reads_softbody_and_rotates_to_match() {
        // Two-bone chain: rigid root + soft child. The child's tip should
        // land on the soft-body target after reconcile.
        let bones = vec![
            Bone::root("root", 5.0, 1.0),
            Bone::child("tip", BoneId(0), Vec2::ZERO, 0.0, 4.0, 1.0)
                .with_stiffness(Stiffness::Soft),
        ];
        let mut sk = Skeleton::new(bones);
        // Target is straight up from child base (5,0): rotate 90° (PI/2)
        // ⇒ tip goes from (9,0) to (5, 4).
        let mut soft = MockSoftBody::default();
        soft.with("tip", Vec2::new(5.0, 4.0));

        reconcile(&mut sk, &mut soft);

        let tip_id = sk.id_of("tip").unwrap();
        let tip_world = sk.world_tip(tip_id);
        assert!(
            approx_vec(tip_world, Vec2::new(5.0, 4.0), 1e-3),
            "tip should land on soft-body target, got {tip_world:?}"
        );
        // The soft bone's world angle should be PI/2 (pointing +y).
        assert!(approx(sk.world_angle(tip_id), PI / 2.0, 1e-3));
    }

    #[test]
    fn mixed_rigid_and_soft_in_one_skeleton() {
        // Rigid head, soft ear. The ear tip is in the soft body; the head
        // tip is pushed into the soft body.
        let bones = vec![
            Bone::root("head", 6.0, 1.0),
            Bone::child("ear", BoneId(0), Vec2::ZERO, 0.0, 3.0, 1.0)
                .with_stiffness(Stiffness::Soft),
        ];
        let mut sk = Skeleton::new(bones);
        sk.set_root(Vec2::new(20.0, 20.0));
        let mut soft = MockSoftBody::default();
        // Wherever the ear goes (-x from head tip), reconcile must produce
        // an ear angle that lands the tip there.
        soft.with("ear", Vec2::new(23.0, 20.0));

        reconcile(&mut sk, &mut soft);

        // Rigid head pushed its tip: head tip = root + length = (26, 20).
        assert_eq!(soft.point("head"), Some(Vec2::new(26.0, 20.0)));
        // Ear was soft → tip should be at the soft-body target.
        let ear_id = sk.id_of("ear").unwrap();
        assert!(approx_vec(sk.world_tip(ear_id), Vec2::new(23.0, 20.0), 1e-3));
    }

    #[test]
    fn bone_without_matching_softbody_point_is_skipped() {
        let bones = vec![
            Bone::root("root", 4.0, 1.0),
            Bone::child("phantom", BoneId(0), Vec2::ZERO, 0.0, 2.0, 1.0)
                .with_stiffness(Stiffness::Soft),
        ];
        let mut sk = Skeleton::new(bones);
        let mut soft = MockSoftBody::default(); // no "phantom" entry
        reconcile(&mut sk, &mut soft);

        // The Soft bone with no matching point keeps its rest angle (0).
        let phantom = sk.id_of("phantom").unwrap();
        assert!(approx(sk.effective_angle(phantom), 0.0, 1e-5));
        // Rigid root still pushed its tip.
        assert_eq!(soft.point("root"), Some(Vec2::new(4.0, 0.0)));
    }

    #[test]
    fn reconcile_fks_dirty_skeleton_first() {
        // If the skeleton hasn't been forward-kinematicked yet, reconcile
        // should do it itself rather than read stale (zero) world cache.
        let bones = vec![Bone::root("root", 7.0, 1.0)];
        let mut sk = Skeleton::new(bones);
        sk.set_root(Vec2::new(3.0, 3.0));
        // Note: we DON'T call sk.forward(); reconcile must handle dirty.
        let mut soft = MockSoftBody::default();
        reconcile(&mut sk, &mut soft);
        assert_eq!(soft.point("root"), Some(Vec2::new(10.0, 3.0)));
    }
}
