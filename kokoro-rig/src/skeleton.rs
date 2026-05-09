//! `Skeleton` — collection of bones in a tree, with forward kinematics.
//!
//! Storage layout: bones live in a flat `Vec<Bone>`, indexed by `BoneId`.
//! **Children must appear after their parents** in the vector. This is the
//! one structural invariant the FK pass relies on (single-pass topological
//! walk instead of recursion). `Skeleton::new` validates this in debug
//! builds via `debug_assert!`.

use crate::bone::{Bone, BoneId, Vec2};

/// A skeleton is a tree of bones plus the per-frame state derived from
/// applying a pose to those bones (world positions and angles).
///
/// The skeleton itself is **mutable but pose-aware**: callers set per-bone
/// pose deltas via `set_angle`, then call `forward()` to recompute world
/// positions. This separation keeps FK explicit and cheap to skip when
/// nothing has changed.
#[derive(Clone, Debug)]
pub struct Skeleton {
    bones: Vec<Bone>,

    /// Per-bone pose override. `None` means "use rest_angle". `Some(a)`
    /// replaces (does NOT add to) the rest angle. Pose layers (next module)
    /// are responsible for deciding what value to write here.
    pose_angles: Vec<Option<f32>>,

    /// World-space base position of each bone, populated by `forward()`.
    world_positions: Vec<Vec2>,

    /// World-space rotation of each bone, populated by `forward()`.
    world_angles: Vec<f32>,

    /// World-space position the root bone sits at. Used to place the whole
    /// creature on the canvas.
    pub root_position: Vec2,

    /// True when the world cache hasn't been recomputed since the last
    /// pose mutation. `forward()` clears it; `set_angle`/`set_root` set it.
    dirty: bool,
}

impl Skeleton {
    /// Build a skeleton from a flat list of bones. The order matters:
    /// every bone's parent (if any) must come before it. Cheap to enforce
    /// since rigs are tiny (≤ ~30 bones for a single creature).
    pub fn new(bones: Vec<Bone>) -> Self {
        debug_assert!(
            bones.first().is_some_and(|b| b.parent.is_none()),
            "skeleton must start with a root bone (parent = None)"
        );
        for (i, bone) in bones.iter().enumerate() {
            if let Some(parent) = bone.parent {
                debug_assert!(
                    parent.index() < i,
                    "bone '{}' at index {} references parent index {} that comes later — \
                     bones must be stored in topological order",
                    bone.name,
                    i,
                    parent.index(),
                );
            }
        }

        let n = bones.len();
        Self {
            bones,
            pose_angles: vec![None; n],
            world_positions: vec![Vec2::ZERO; n],
            world_angles: vec![0.0; n],
            root_position: Vec2::ZERO,
            dirty: true,
        }
    }

    pub fn len(&self) -> usize {
        self.bones.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }

    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }

    pub fn bone(&self, id: BoneId) -> &Bone {
        &self.bones[id.index()]
    }

    /// Linear-scan name lookup. Skeletons are small enough (< 30 bones) that
    /// a hash map is more overhead than payoff. Returns the first match.
    pub fn id_of(&self, name: &str) -> Option<BoneId> {
        self.bones
            .iter()
            .position(|b| b.name == name)
            .map(|i| BoneId(i as u16))
    }

    /// Move the skeleton's root in world space. Marks the world cache dirty
    /// so the next `forward()` recomputes from the new origin.
    pub fn set_root(&mut self, position: Vec2) {
        self.root_position = position;
        self.dirty = true;
    }

    /// Override a bone's pose angle. Replaces the rest angle until cleared.
    pub fn set_angle(&mut self, id: BoneId, angle_radians: f32) {
        self.pose_angles[id.index()] = Some(angle_radians);
        self.dirty = true;
    }

    /// Drop the pose override and revert this bone to its rest angle.
    pub fn clear_angle(&mut self, id: BoneId) {
        self.pose_angles[id.index()] = None;
        self.dirty = true;
    }

    /// Returns whichever angle is "in effect" for this bone (override or rest).
    pub fn effective_angle(&self, id: BoneId) -> f32 {
        self.pose_angles[id.index()].unwrap_or(self.bones[id.index()].rest_angle)
    }

    /// Forward kinematics. Single linear pass over bones (no recursion)
    /// because we enforced parents-before-children at construction.
    ///
    /// For each bone:
    /// 1. Take parent's tip position + parent's world angle.
    /// 2. Add this bone's `rest_offset` rotated into the parent's frame.
    /// 3. World angle = parent's angle + this bone's effective angle.
    pub fn forward(&mut self) {
        for i in 0..self.bones.len() {
            let bone = &self.bones[i];
            let local_angle = self.pose_angles[i].unwrap_or(bone.rest_angle);

            let (parent_tip, parent_angle) = match bone.parent {
                None => (self.root_position, 0.0),
                Some(parent_id) => {
                    let pi = parent_id.index();
                    let parent_bone = &self.bones[pi];
                    let parent_base = self.world_positions[pi];
                    let p_angle = self.world_angles[pi];
                    // Tip = base + parent.length × direction(p_angle).
                    let len = parent_bone.effective_length();
                    let tip = Vec2::new(
                        parent_base.x + len * p_angle.cos(),
                        parent_base.y + len * p_angle.sin(),
                    );
                    (tip, p_angle)
                }
            };

            let offset_world = bone.rest_offset.rotated(parent_angle);
            self.world_positions[i] = parent_tip.add(offset_world);
            self.world_angles[i] = parent_angle + local_angle;
        }
        self.dirty = false;
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// World-space base of the bone. Call after `forward()`. In debug builds
    /// asserts the cache is fresh — silent stale reads are a common rig bug.
    pub fn world_base(&self, id: BoneId) -> Vec2 {
        debug_assert!(!self.dirty, "world_base read before forward()");
        self.world_positions[id.index()]
    }

    pub fn world_angle(&self, id: BoneId) -> f32 {
        debug_assert!(!self.dirty, "world_angle read before forward()");
        self.world_angles[id.index()]
    }

    /// World-space tip of the bone (= base + length × direction(angle)).
    pub fn world_tip(&self, id: BoneId) -> Vec2 {
        debug_assert!(!self.dirty, "world_tip read before forward()");
        let i = id.index();
        let base = self.world_positions[i];
        let angle = self.world_angles[i];
        let len = self.bones[i].effective_length();
        Vec2::new(base.x + len * angle.cos(), base.y + len * angle.sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn approx_vec(a: Vec2, b: Vec2) -> bool {
        approx(a.x, b.x) && approx(a.y, b.y)
    }

    #[test]
    fn root_only_skeleton_sits_at_origin_by_default() {
        let bones = vec![Bone::root("root", 10.0, 1.0)];
        let mut sk = Skeleton::new(bones);
        sk.forward();
        assert_eq!(sk.world_base(BoneId(0)), Vec2::ZERO);
        assert!(approx(sk.world_angle(BoneId(0)), 0.0));
        assert_eq!(sk.world_tip(BoneId(0)), Vec2::new(10.0, 0.0));
    }

    #[test]
    fn root_position_translates_whole_skeleton() {
        let bones = vec![Bone::root("root", 5.0, 1.0)];
        let mut sk = Skeleton::new(bones);
        sk.set_root(Vec2::new(32.0, 32.0));
        sk.forward();
        assert_eq!(sk.world_base(BoneId(0)), Vec2::new(32.0, 32.0));
        assert_eq!(sk.world_tip(BoneId(0)), Vec2::new(37.0, 32.0));
    }

    #[test]
    fn child_bone_attaches_to_parent_tip_with_offset() {
        let bones = vec![
            Bone::root("spine", 10.0, 2.0),
            // Child sits 0px past parent's tip, no extra offset, length 4
            Bone::child("head", BoneId(0), Vec2::ZERO, 0.0, 4.0, 2.0),
        ];
        let mut sk = Skeleton::new(bones);
        sk.forward();
        // Spine tip is at (10, 0); head sits at the same point.
        assert_eq!(sk.world_base(BoneId(1)), Vec2::new(10.0, 0.0));
        assert_eq!(sk.world_tip(BoneId(1)), Vec2::new(14.0, 0.0));
    }

    #[test]
    fn child_offset_is_rotated_into_parent_frame() {
        // Parent rotated 90° (PI/2) — its local +x becomes world +y.
        // A child with rest_offset (3, 0) should land at (parent_tip + (0, 3)).
        let bones = vec![
            Bone {
                rest_angle: PI / 2.0,
                ..Bone::root("root", 6.0, 1.0)
            },
            Bone::child("child", BoneId(0), Vec2::new(3.0, 0.0), 0.0, 2.0, 1.0),
        ];
        let mut sk = Skeleton::new(bones);
        sk.forward();
        // Root world: base (0,0), angle PI/2, tip (0, 6).
        assert!(approx_vec(sk.world_tip(BoneId(0)), Vec2::new(0.0, 6.0)));
        // Child base = parent_tip (0,6) + offset (3,0) rotated PI/2 = (0,3) ⇒ (0, 9).
        assert!(approx_vec(sk.world_base(BoneId(1)), Vec2::new(0.0, 9.0)));
    }

    #[test]
    fn pose_override_replaces_rest_angle() {
        let bones = vec![
            Bone::root("root", 10.0, 1.0),
            Bone::child("arm", BoneId(0), Vec2::ZERO, 0.0, 5.0, 1.0),
        ];
        let mut sk = Skeleton::new(bones);
        let arm = sk.id_of("arm").unwrap();

        // Bend the arm 90° relative to the root.
        sk.set_angle(arm, PI / 2.0);
        sk.forward();
        // Arm base sits at root tip (10, 0); arm rotated 90° points to (10, 5).
        assert!(approx_vec(sk.world_tip(arm), Vec2::new(10.0, 5.0)));

        // Clear the override → arm should snap back to rest angle (0).
        sk.clear_angle(arm);
        sk.forward();
        assert!(approx_vec(sk.world_tip(arm), Vec2::new(15.0, 0.0)));
    }

    #[test]
    fn id_of_finds_named_bones() {
        let bones = vec![
            Bone::root("pelvis", 1.0, 1.0),
            Bone::child("spine", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0),
            Bone::child("head", BoneId(1), Vec2::ZERO, 0.0, 1.0, 1.0),
        ];
        let sk = Skeleton::new(bones);
        assert_eq!(sk.id_of("pelvis"), Some(BoneId(0)));
        assert_eq!(sk.id_of("head"), Some(BoneId(2)));
        assert_eq!(sk.id_of("nope"), None);
    }

    #[test]
    fn genome_length_scale_changes_tip_distance() {
        let bones = vec![
            Bone::root("root", 10.0, 1.0).with_genome_scales(1.20, 1.0),
        ];
        let mut sk = Skeleton::new(bones);
        sk.forward();
        // Effective length 12, so tip at (12, 0) instead of (10, 0).
        assert_eq!(sk.world_tip(BoneId(0)), Vec2::new(12.0, 0.0));
    }

    #[test]
    fn dirty_flag_tracks_pose_mutations() {
        let bones = vec![Bone::root("root", 1.0, 1.0)];
        let mut sk = Skeleton::new(bones);
        assert!(sk.dirty());
        sk.forward();
        assert!(!sk.dirty());
        sk.set_root(Vec2::new(1.0, 1.0));
        assert!(sk.dirty());
    }

    #[test]
    #[should_panic(expected = "must be stored in topological order")]
    fn child_before_parent_panics_in_debug() {
        // Index 1 references parent at index 2 — order invalid even though
        // the root at index 0 is fine.
        let bones = vec![
            Bone::root("root", 1.0, 1.0),
            Bone::child("early_child", BoneId(2), Vec2::ZERO, 0.0, 1.0, 1.0),
            Bone::child("late_parent", BoneId(0), Vec2::ZERO, 0.0, 1.0, 1.0),
        ];
        let _ = Skeleton::new(bones);
    }
}
