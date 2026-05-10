//! Moluun skeletons.
//!
//! Anatomy lean per `feedback_moluun_red_panda_lean.md`:
//! - **Primary**: red panda (long ringed tail, bipedal-capable, semi-
//!   retractile claws, plantigrade stance, triangular tufted ears).
//! - **Secondary**: wombat juvenile (cub body mass — plump, low-slung,
//!   head ~55% of total height).
//! - **Tertiary**: koala (ear shape — large rounded with perpendicular tuft).
//!
//! All canvas math assumes the standard 64×64 sprite buffer. Pelvis sits
//! at canvas (32, 32) by default; callers can `set_root` to move the rig.
//!
//! ## Conventions used here
//!
//! - **Pelvis is a 0-length virtual pivot** at the rig's root. Its
//!   `rest_angle = -π/2` so its local +x = world up, which means children
//!   with `rest_angle = 0` continue *up* the body. This makes the spine →
//!   neck → head chain easy to read.
//! - **Local frames vs world**: a child's `rest_offset` is in the parent's
//!   local frame. With pelvis facing up, a parent-local offset of `(a, b)`
//!   shifts the child by `(b, -a)` in world space (b on world-x = right,
//!   `-a` on world-y = up). Comments show the resulting world delta where
//!   it's not obvious.
//! - **Length-0 pivots**: shoulders and hips are 0-length bones acting as
//!   pure rotation pivots. Their children (arm, thigh) attach at the same
//!   world point as the pivot itself.
//! - **Stiffness = Soft** for parts that should drift with the soft-body
//!   simulation: ear tips, paws, foot tips, the tail beyond `tail_2`.
//!   Everything load-bearing (skull, torso, hips, shoulders) stays Rigid.

use kokoro_rig::{Bone, BoneId, Skeleton, Stiffness, Vec2};
use std::f32::consts::PI;

/// Build the Moluun cub skeleton.
///
/// Default rest pose: 26 bones, root at canvas centre, total visible
/// extent y ≈ 9..47 (≈ 38px) with head/total ≈ 0.55. Genome scale-factors
/// all default to 1.0; future code can `with_genome_scales(...)` per bone
/// to give individual creatures unique proportions.
//
// `dead_code` allow: cub_skeleton is the consumer-facing API but the
// consumer (a rewritten moluun::draw_cub) lands in a separate commit.
// Until then the unit tests in this module are the only callers.
#[allow(dead_code)]
pub fn cub_skeleton() -> Skeleton {
    let mut bones = Vec::with_capacity(26);

    // ============================================================
    // ROOT — pelvis as a 0-length virtual pivot pointing "up".
    // ============================================================
    bones.push(Bone {
        rest_angle: -PI / 2.0,
        ..Bone::root("pelvis", 0.0, 1.0)
    });
    let pelvis = BoneId(0);

    // ============================================================
    // SPINE → NECK → HEAD — vertical column rising from pelvis.
    // ============================================================
    // Each child's local rest_angle is 0 → continues parent's direction
    // (= up in world). Lengths design the head/body ratio per the spec.

    // Spine: y 32 → 26.
    bones.push(Bone::child("spine", pelvis, Vec2::ZERO, 0.0, 6.0, 4.0));
    let spine = BoneId(1);

    // Neck: y 26 → 23.
    bones.push(Bone::child("neck", spine, Vec2::ZERO, 0.0, 3.0, 3.0));
    let neck = BoneId(2);

    // Head: y 23 → 10. 13px tall × 11px wide visual radius. z=+1 so the
    // face renders on top of the body when bones overlap in z-order.
    bones.push(Bone::child("head", neck, Vec2::ZERO, 0.0, 13.0, 11.0).with_z(1));
    let head = BoneId(3);

    // ============================================================
    // EARS — koala-tuft size, attached near top of head, tilted out.
    // ============================================================
    // Local (1, ±4) in head frame → world (∓4, -1) from head tip (32, 10),
    // so ear bases land at (28, 9) and (36, 9). Ear angle ±π/5 tilts the
    // bone outward. Soft so the tip drifts when the soft body wiggles.
    bones.push(
        Bone::child("ear_l", head, Vec2::new(1.0, -4.0), -PI / 5.0, 5.0, 4.0)
            .with_z(2)
            .with_stiffness(Stiffness::Soft),
    );
    bones.push(
        Bone::child("ear_r", head, Vec2::new(1.0, 4.0), PI / 5.0, 5.0, 4.0)
            .with_z(2)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // FACE FEATURES — tiny pivot bones for KawaiiEye / snout brush.
    // ============================================================
    // These bones don't paint along their length (length 1 = stub). The
    // brushes that paint eyes/snout read each bone's world *base* and
    // splatter pixels there. Local (-3, ±3) in head frame → eyes 3px back
    // from head tip (= lower on the face) and 3px to either side, matching
    // the wombat-juvenile "eyes low on face" kindchenschema.
    bones.push(Bone::child("eye_l", head, Vec2::new(-3.0, -3.0), 0.0, 1.0, 1.0).with_z(2));
    bones.push(Bone::child("eye_r", head, Vec2::new(-3.0, 3.0), 0.0, 1.0, 1.0).with_z(2));
    // Snout sits 7px back along head (+x is up the face here, so -7 = down
    // the face = below the eyes), centred on the head's midline.
    bones.push(Bone::child("snout", head, Vec2::new(-7.0, 0.0), 0.0, 2.0, 2.0).with_z(2));

    // ============================================================
    // ARMS — short stocky cub arms. Left arm renders behind body, right
    // in front (z=±1). Soft at paw + arm so they sway with motion.
    // ============================================================
    // Shoulders are 0-length pivots offset from the spine. Spine's local
    // (-1, ±4) → world offset (∓4, +1) from spine tip (32, 26) ⇒ shoulders
    // at world (28, 27) and (36, 27).
    bones.push(Bone::child("shoulder_l", spine, Vec2::new(-1.0, -4.0), 0.0, 0.0, 1.0).with_z(-1));
    let shoulder_l = BoneId(9);
    // Left arm should sweep down-out to the LEFT in world. Spine's world
    // angle is -π/2, so local angle 7π/6 → world -π/2 + 7π/6 = 2π/3 ≈ 120°
    // (down + left). Right arm mirrors at local 5π/6 → world π/3 (down +
    // right).
    bones.push(
        Bone::child("arm_l", shoulder_l, Vec2::ZERO, 7.0 * PI / 6.0, 5.0, 2.5)
            .with_z(-1)
            .with_stiffness(Stiffness::Soft),
    );
    let arm_l = BoneId(10);
    bones.push(
        Bone::child("paw_l", arm_l, Vec2::ZERO, PI / 6.0, 2.0, 2.0)
            .with_z(-1)
            .with_stiffness(Stiffness::Soft),
    );

    bones.push(Bone::child("shoulder_r", spine, Vec2::new(-1.0, 4.0), 0.0, 0.0, 1.0).with_z(1));
    let shoulder_r = BoneId(12);
    bones.push(
        Bone::child("arm_r", shoulder_r, Vec2::ZERO, 5.0 * PI / 6.0, 5.0, 2.5)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );
    let arm_r = BoneId(13);
    bones.push(
        Bone::child("paw_r", arm_r, Vec2::ZERO, -PI / 6.0, 2.0, 2.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // LEGS — short stocky cub legs (red panda + wombat juvenile blend).
    // ============================================================
    // Hips offset from pelvis: pelvis-local (-2, ±3) → world (∓3, +2)
    // from pelvis (32, 32) ⇒ hips at world (29, 34) and (35, 34).
    bones.push(Bone::child("hip_l", pelvis, Vec2::new(-2.0, -3.0), 0.0, 0.0, 1.0).with_z(-1));
    let hip_l = BoneId(15);
    // Thigh local angle π → world angle -π/2 + π = π/2 (straight down).
    bones.push(Bone::child("thigh_l", hip_l, Vec2::ZERO, PI, 6.0, 2.5).with_z(-1));
    let thigh_l = BoneId(16);
    // Foot local angle 0 → continues thigh direction (straight down).
    // Soft so the foot can drift onto the "ground" during simulation.
    bones.push(
        Bone::child("foot_l", thigh_l, Vec2::ZERO, 0.0, 3.0, 2.0)
            .with_z(-1)
            .with_stiffness(Stiffness::Soft),
    );

    bones.push(Bone::child("hip_r", pelvis, Vec2::new(-2.0, 3.0), 0.0, 0.0, 1.0).with_z(1));
    let hip_r = BoneId(18);
    bones.push(Bone::child("thigh_r", hip_r, Vec2::ZERO, PI, 6.0, 2.5).with_z(1));
    let thigh_r = BoneId(19);
    bones.push(
        Bone::child("foot_r", thigh_r, Vec2::ZERO, 0.0, 3.0, 2.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // TAIL — RED PANDA SIGNATURE. Long, ringed, peeking out behind-right.
    // ============================================================
    // Five tapered segments, total length 4+4+4+3+2 = 17 px (vs body ~6).
    // tail_1 attaches behind+below pelvis: pelvis-local (-3, 5) → world
    // (5, +3) from pelvis ⇒ tail base at (37, 35). Initial angle 2π/3
    // against pelvis → world angle -π/2 + 2π/3 = π/6 (down-right). Each
    // subsequent segment bends slightly back up (-π/12 local) so the tail
    // forms a gentle S-curve to the right of the cub.
    // Most of the tail is Soft so simulation can sway it independently.
    bones.push(
        Bone::child("tail_1", pelvis, Vec2::new(-3.0, 5.0), 2.0 * PI / 3.0, 4.0, 3.0).with_z(1),
    );
    let tail_1 = BoneId(21);
    bones.push(
        Bone::child("tail_2", tail_1, Vec2::ZERO, -PI / 12.0, 4.0, 2.5)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_2 = BoneId(22);
    bones.push(
        Bone::child("tail_3", tail_2, Vec2::ZERO, -PI / 12.0, 4.0, 2.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_3 = BoneId(23);
    bones.push(
        Bone::child("tail_4", tail_3, Vec2::ZERO, -PI / 12.0, 3.0, 1.5)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_4 = BoneId(24);
    bones.push(
        Bone::child("tail_5", tail_4, Vec2::ZERO, -PI / 12.0, 2.0, 1.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );

    let mut sk = Skeleton::new(bones);
    sk.set_root(Vec2::new(32.0, 32.0));
    sk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn approx_vec(a: Vec2, b: Vec2, eps: f32) -> bool {
        approx(a.x, b.x, eps) && approx(a.y, b.y, eps)
    }

    #[test]
    fn skeleton_has_expected_bone_count() {
        // 1 pelvis + 3 spine/neck/head + 2 ears + 3 face + 6 arm bones
        // + 6 leg bones + 5 tail = 26.
        let sk = cub_skeleton();
        assert_eq!(sk.len(), 26);
    }

    #[test]
    fn key_bones_resolvable_by_name() {
        let sk = cub_skeleton();
        for name in [
            "pelvis", "spine", "neck", "head", "ear_l", "ear_r", "eye_l",
            "eye_r", "snout", "shoulder_l", "arm_l", "paw_l", "shoulder_r",
            "arm_r", "paw_r", "hip_l", "thigh_l", "foot_l", "hip_r",
            "thigh_r", "foot_r", "tail_1", "tail_2", "tail_3", "tail_4",
            "tail_5",
        ] {
            assert!(sk.id_of(name).is_some(), "missing bone '{name}'");
        }
    }

    #[test]
    fn head_tip_lands_above_pelvis_at_expected_height() {
        // Head tip should sit at (32, 10): pelvis (32,32) - spine 6 -
        // neck 3 - head 13 = y 10.
        let mut sk = cub_skeleton();
        sk.forward();
        let head = sk.id_of("head").unwrap();
        let tip = sk.world_tip(head);
        assert!(
            approx_vec(tip, Vec2::new(32.0, 10.0), 1e-3),
            "head tip {tip:?} should be at (32, 10)"
        );
    }

    #[test]
    fn ear_bases_flank_top_of_head() {
        let mut sk = cub_skeleton();
        sk.forward();
        let el = sk.world_base(sk.id_of("ear_l").unwrap());
        let er = sk.world_base(sk.id_of("ear_r").unwrap());
        // Symmetric across x=32; both slightly above head tip (y=10).
        assert!(approx(el.x + er.x, 64.0, 1e-3), "ears should be x-symmetric");
        assert!(el.y < 10.5 && er.y < 10.5, "ears should sit at/above head tip y");
        // Left ear actually on the left.
        assert!(el.x < 32.0 && er.x > 32.0);
    }

    #[test]
    fn eyes_sit_low_on_face_per_kindchenschema() {
        // Eyes attach with head-local (-3, ±3) → world (∓3, +3) from head
        // tip (32, 10) ⇒ (29, 13) and (35, 13). Low on face = closer to
        // the snout = below the head's vertical midpoint (which is y=16.5).
        let mut sk = cub_skeleton();
        sk.forward();
        let eye_l = sk.world_base(sk.id_of("eye_l").unwrap());
        let eye_r = sk.world_base(sk.id_of("eye_r").unwrap());
        assert!(approx_vec(eye_l, Vec2::new(29.0, 13.0), 1e-3));
        assert!(approx_vec(eye_r, Vec2::new(35.0, 13.0), 1e-3));
    }

    #[test]
    fn feet_sit_below_pelvis_near_canvas_floor() {
        let mut sk = cub_skeleton();
        sk.forward();
        let foot_l = sk.world_base(sk.id_of("foot_l").unwrap());
        let foot_r = sk.world_base(sk.id_of("foot_r").unwrap());
        // Hips at y=34, thigh 6px down → y=40 → foot base at y=40.
        assert!(
            approx(foot_l.y, 40.0, 0.5) && approx(foot_r.y, 40.0, 0.5),
            "feet bases should sit ~y=40, got l={:?} r={:?}",
            foot_l, foot_r
        );
        // Foot tips reach further down — should be near the canvas floor.
        let foot_l_tip = sk.world_tip(sk.id_of("foot_l").unwrap());
        assert!(foot_l_tip.y >= 42.0, "foot_l tip should reach the ground");
    }

    #[test]
    fn arms_sweep_outward_to_their_own_sides() {
        // Regression guard for the local-angle inversion bug: left arm
        // should reach into x < 32 (left of body), right arm into x > 32.
        let mut sk = cub_skeleton();
        sk.forward();
        let paw_l = sk.world_tip(sk.id_of("paw_l").unwrap());
        let paw_r = sk.world_tip(sk.id_of("paw_r").unwrap());
        assert!(paw_l.x < 30.0, "left paw should be left of centre, got x={}", paw_l.x);
        assert!(paw_r.x > 34.0, "right paw should be right of centre, got x={}", paw_r.x);
        // And both should hang below the shoulders.
        assert!(paw_l.y > 28.0 && paw_r.y > 28.0);
    }

    #[test]
    fn tail_extends_to_the_right_of_the_cub() {
        // Tail should peek out behind/right (red panda silhouette) — every
        // segment after tail_1 should be at x > 32 and y > 32.
        let mut sk = cub_skeleton();
        sk.forward();
        for name in ["tail_1", "tail_2", "tail_3", "tail_4", "tail_5"] {
            let base = sk.world_base(sk.id_of(name).unwrap());
            assert!(base.x > 32.0, "{name} should be right of centre, got x={}", base.x);
        }
        // Tail tip is the bone *_5*'s world tip — should be the furthest
        // point from the body.
        let tip = sk.world_tip(sk.id_of("tail_5").unwrap());
        assert!(tip.x > 38.0, "tail tip {tip:?} should reach x > 38");
    }

    #[test]
    fn rigid_vs_soft_distribution_matches_design() {
        // Spot-check stiffness: skull/torso = Rigid, ear/paw/foot/tail-late = Soft.
        let sk = cub_skeleton();
        let rigid = ["pelvis", "spine", "neck", "head", "shoulder_l", "hip_r"];
        let soft  = ["ear_l", "ear_r", "paw_l", "foot_r", "tail_3", "tail_5"];
        for name in rigid {
            let s = sk.bone(sk.id_of(name).unwrap()).stiffness;
            assert_eq!(s, Stiffness::Rigid, "{name} should be Rigid");
        }
        for name in soft {
            let s = sk.bone(sk.id_of(name).unwrap()).stiffness;
            assert_eq!(s, Stiffness::Soft, "{name} should be Soft");
        }
    }

    #[test]
    fn z_layering_puts_face_above_body() {
        let sk = cub_skeleton();
        let head_z = sk.bone(sk.id_of("head").unwrap()).z_layer;
        let ear_z  = sk.bone(sk.id_of("ear_l").unwrap()).z_layer;
        let snout_z = sk.bone(sk.id_of("snout").unwrap()).z_layer;
        let body_z = sk.bone(sk.id_of("spine").unwrap()).z_layer;
        assert!(head_z > body_z);
        assert!(ear_z > head_z); // ears render on top of the head
        assert!(snout_z >= head_z);
    }

    #[test]
    fn root_translation_moves_whole_skeleton() {
        // Authored at (32, 32); moving the root should shift every bone
        // identically.
        let mut sk_a = cub_skeleton();
        sk_a.forward();
        let head_a = sk_a.world_tip(sk_a.id_of("head").unwrap());

        let mut sk_b = cub_skeleton();
        sk_b.set_root(Vec2::new(40.0, 40.0));
        sk_b.forward();
        let head_b = sk_b.world_tip(sk_b.id_of("head").unwrap());

        assert!(approx(head_b.x - head_a.x, 8.0, 1e-3));
        assert!(approx(head_b.y - head_a.y, 8.0, 1e-3));
    }

    /// Saves a PNG of the cub skeleton to target/sprite-snapshots/ for
    /// visual review. Ignored by default — run with:
    ///
    /// ```bash
    /// cargo test moluun_cub_skeleton -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_skeleton() {
        use image::{Rgba, RgbaImage};
        use std::path::PathBuf;

        let mut sk = cub_skeleton();
        sk.forward();

        let mut img = RgbaImage::new(64, 64);
        // Cream background — picks the master palette's neutral so the
        // dark bone strokes pop.
        for px in img.pixels_mut() {
            *px = Rgba([217, 199, 174, 255]);
        }

        kokoro_rig::debug::render_skeleton_overlay(
            &mut img,
            &sk,
            Rgba([59, 36, 24, 255]),    // DeepBrown line stroke
            Rgba([217, 13, 67, 255]),   // Red joint dots (pop against cream)
            true,                        // tint by z so layering reads at a glance
        );

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        img.save(dir.join("moluun_cub_skeleton.png")).unwrap();

        // 4× upscale alongside, same trick the snapshot_sprites test uses.
        let mut up = RgbaImage::new(256, 256);
        for y in 0..64u32 {
            for x in 0..64u32 {
                let p = *img.get_pixel(x, y);
                for dy in 0..4u32 {
                    for dx in 0..4u32 {
                        up.put_pixel(x * 4 + dx, y * 4 + dy, p);
                    }
                }
            }
        }
        up.save(dir.join("moluun_cub_skeleton@4x.png")).unwrap();

        eprintln!(
            "moluun_cub_skeleton: wrote PNGs to {:?}",
            dir.canonicalize().unwrap_or(dir)
        );
    }
}
