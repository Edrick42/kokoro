//! Moluun skeletons.
//!
//! Anatomy lean per `feedback_moluun_red_panda_lean.md`: red panda primary,
//! wombat juvenile body mass secondary, koala ear shape tertiary.
//!
//! ## Pose convention
//!
//! Cub is rendered in **quadrupedal side view**, head facing right, tail
//! curling left — matching the pixel-art ref the user picked. This breaks
//! with the bipedal-frontal pose used by the other species' draws today;
//! they'll either follow suit or stay frontal as their own design choices.
//!
//! All canvas math assumes the standard 64×64 sprite buffer. Pelvis sits
//! at canvas (22, 36) by default — the cub occupies roughly x=8..58 and
//! y=22..50, leaving room above for ear tips and below for ground shadow.
//!
//! ## Convention notes
//!
//! - **Pelvis is the root**, length 0, angle 0 — children with rest_angle
//!   0 continue along +x (which in this side-view rig is "forward",
//!   toward the head on the right).
//! - **Z layering** controls front-vs-back leg rendering and the back ear
//!   peeking around the head: front leg + front ear z=+1; back leg + back
//!   ear z=-1.
//! - **Stiffness**: skull / spine / hips = Rigid (skeleton wins).
//!   Tail beyond tail_2 + paws + ear tips = Soft (soft body wins).

use kokoro_rig::{Bone, BoneId, Skeleton, Stiffness, Vec2};
use std::f32::consts::PI;

/// Build the Moluun cub skeleton in quadrupedal side-view pose.
///
/// 19 bones total. Default rest pose centred so cub silhouette covers
/// roughly x=8..58 and y=22..50 of the 64×64 canvas. Genome scale-factors
/// all default to 1.0.
//
// `dead_code` allow: cub_skeleton is the consumer-facing API but the
// consumer (moluun::draw_cub) lives in a sibling crate file. The unit
// tests here are the only direct callers in this module's view.
#[allow(dead_code)]
pub fn cub_skeleton() -> Skeleton {
    let mut bones = Vec::with_capacity(19);

    // ============================================================
    // ROOT — pelvis at the BACK of the body, pointing forward (+x).
    // ============================================================
    bones.push(Bone::root("pelvis", 0.0, 1.0));
    let pelvis = BoneId(0);

    // ============================================================
    // SPINE → NECK → HEAD — horizontal column running forward.
    // ============================================================
    // Spine length 12 → spine tip at (root + 12, root) = the shoulder
    // area. Wide width 6 = body's vertical thickness.
    bones.push(Bone::child("spine", pelvis, Vec2::ZERO, 0.0, 12.0, 6.0));
    let spine = BoneId(1);

    // Neck angles slightly upward (-π/8 from spine) so the head sits a
    // bit above the spine line — proper red-panda "alert" silhouette.
    bones.push(Bone::child("neck", spine, Vec2::ZERO, -PI / 8.0, 3.0, 3.0));
    let neck = BoneId(2);

    // Head — large for cub, length defines vertical extent of the cranium.
    // z=+1 so face renders over the body when overlapping.
    bones.push(Bone::child("head", neck, Vec2::ZERO, -PI / 12.0, 9.0, 8.0).with_z(1));
    let head = BoneId(3);

    // ============================================================
    // EARS — both above head; back ear partially hidden behind front.
    // ============================================================
    // Local +x in head frame ≈ "up the head" given head's tilted-up angle.
    // Local (4, -2) places ear bases above and slightly to the front of
    // the head's tip; angle differences spread the two ears apart.
    bones.push(
        Bone::child("ear_front", head, Vec2::new(4.0, -1.0), -PI / 4.0, 4.0, 3.0)
            .with_z(2)
            .with_stiffness(Stiffness::Soft),
    );
    bones.push(
        Bone::child("ear_back", head, Vec2::new(2.0, 1.0), -PI / 6.0, 4.0, 3.0)
            .with_z(0) // behind the head dome — only the tip pokes through
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // FACE FEATURES — single visible eye + snout (side view).
    // ============================================================
    // Eye sits on the side of the head (toward viewer). Local (3, 1)
    // places it lower on the face per kindchenschema.
    bones.push(Bone::child("eye", head, Vec2::new(3.0, 1.0), 0.0, 1.0, 1.0).with_z(2));
    // Snout at the front-bottom of the head — caller paints it as a tiny
    // black wedge.
    bones.push(Bone::child("snout", head, Vec2::new(7.0, 1.0), 0.0, 2.0, 2.0).with_z(2));

    // ============================================================
    // FRONT LEG — at forward end of spine, dangling down.
    // ============================================================
    // Local (10, 2) places shoulder at world (~32, ~38) — under-front of
    // the body. Angle π/2 against pelvis (which is angle 0) → world π/2
    // = straight DOWN. Front leg renders in front (z=+1).
    bones.push(
        Bone::child("shoulder_front", spine, Vec2::new(10.0, 2.0), PI / 2.0, 0.0, 1.0).with_z(1),
    );
    let shoulder_front = BoneId(8);
    bones.push(
        Bone::child("leg_front", shoulder_front, Vec2::ZERO, 0.0, 6.0, 3.0).with_z(1),
    );
    let leg_front = BoneId(9);
    bones.push(
        Bone::child("paw_front", leg_front, Vec2::ZERO, 0.0, 2.0, 3.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // BACK LEG — at pelvis end, dangling down.
    // ============================================================
    // Local (0, 2) places hip at world (~22, ~38) — under-back of body.
    // Same downward angle. Back leg renders BEHIND the body (z=-1).
    bones.push(
        Bone::child("hip_back", pelvis, Vec2::new(0.0, 2.0), PI / 2.0, 0.0, 1.0).with_z(-1),
    );
    let hip_back = BoneId(11);
    bones.push(
        Bone::child("leg_back", hip_back, Vec2::ZERO, 0.0, 6.0, 3.0).with_z(-1),
    );
    let leg_back = BoneId(12);
    bones.push(
        Bone::child("paw_back", leg_back, Vec2::ZERO, 0.0, 2.0, 3.0)
            .with_z(-1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // TAIL — RED PANDA SIGNATURE. Extends BACKWARD (-x = leftward in
    // canvas) from pelvis with a gentle downward arc.
    // ============================================================
    // Local angle PI against pelvis (which is angle 0) → world angle
    // π = pointing in -x direction (LEFT). Each subsequent segment bends
    // π/10 downward so the tail forms a gentle arc going down-left.
    // Total tail length 4+4+4+3+2 = 17 px.
    bones.push(
        Bone::child("tail_1", pelvis, Vec2::new(-2.0, 0.0), PI, 4.0, 4.0).with_z(0),
    );
    let tail_1 = BoneId(14);
    bones.push(
        Bone::child("tail_2", tail_1, Vec2::ZERO, PI / 10.0, 4.0, 3.5)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_2 = BoneId(15);
    bones.push(
        Bone::child("tail_3", tail_2, Vec2::ZERO, PI / 10.0, 4.0, 3.0)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_3 = BoneId(16);
    bones.push(
        Bone::child("tail_4", tail_3, Vec2::ZERO, PI / 10.0, 3.0, 2.5)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_4 = BoneId(17);
    bones.push(
        Bone::child("tail_5", tail_4, Vec2::ZERO, PI / 10.0, 2.0, 2.0)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );

    let mut sk = Skeleton::new(bones);
    sk.set_root(Vec2::new(22.0, 36.0));
    sk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn skeleton_has_19_bones() {
        // 1 pelvis + 3 spine/neck/head + 2 ears + 2 face + 3 front leg
        // + 3 back leg + 5 tail = 19.
        assert_eq!(cub_skeleton().len(), 19);
    }

    #[test]
    fn key_bones_resolvable_by_name() {
        let sk = cub_skeleton();
        for name in [
            "pelvis", "spine", "neck", "head", "ear_front", "ear_back",
            "eye", "snout",
            "shoulder_front", "leg_front", "paw_front",
            "hip_back", "leg_back", "paw_back",
            "tail_1", "tail_2", "tail_3", "tail_4", "tail_5",
        ] {
            assert!(sk.id_of(name).is_some(), "missing bone '{name}'");
        }
    }

    #[test]
    fn head_sits_to_the_right_of_pelvis() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis = sk.world_base(sk.id_of("pelvis").unwrap());
        let head_tip = sk.world_tip(sk.id_of("head").unwrap());
        assert!(
            head_tip.x > pelvis.x + 15.0,
            "head tip {head_tip:?} should be well to the right of pelvis {pelvis:?}"
        );
    }

    #[test]
    fn tail_extends_to_the_left() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis = sk.world_base(sk.id_of("pelvis").unwrap());
        let tail_tip = sk.world_tip(sk.id_of("tail_5").unwrap());
        assert!(
            tail_tip.x < pelvis.x - 10.0,
            "tail tip {tail_tip:?} should be well to the left of pelvis {pelvis:?}"
        );
    }

    #[test]
    fn legs_hang_below_body() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis_y = sk.world_base(sk.id_of("pelvis").unwrap()).y;
        for leg in ["paw_front", "paw_back"] {
            let paw_tip = sk.world_tip(sk.id_of(leg).unwrap());
            assert!(paw_tip.y > pelvis_y + 6.0, "{leg} tip should sit below pelvis");
        }
    }

    #[test]
    fn front_leg_sits_forward_of_back_leg() {
        let mut sk = cub_skeleton();
        sk.forward();
        let front = sk.world_base(sk.id_of("shoulder_front").unwrap());
        let back = sk.world_base(sk.id_of("hip_back").unwrap());
        assert!(
            front.x > back.x + 8.0,
            "front leg ({front:?}) should be forward of back leg ({back:?})"
        );
    }

    #[test]
    fn z_layering_separates_front_back() {
        let sk = cub_skeleton();
        let z = |name: &str| sk.bone(sk.id_of(name).unwrap()).z_layer;
        assert!(z("leg_front") > z("leg_back"));
        assert!(z("ear_front") > z("ear_back"));
        assert!(z("head") > z("spine"));
    }

    #[test]
    fn rigid_vs_soft_distribution_matches_design() {
        let sk = cub_skeleton();
        let stiff = |name: &str| sk.bone(sk.id_of(name).unwrap()).stiffness;
        // Skeleton-driven (Rigid).
        for name in ["pelvis", "spine", "head", "shoulder_front", "hip_back", "tail_1"] {
            assert_eq!(stiff(name), Stiffness::Rigid, "{name} should be Rigid");
        }
        // Soft-body-driven (Soft).
        for name in ["ear_front", "ear_back", "paw_front", "paw_back", "tail_3", "tail_5"] {
            assert_eq!(stiff(name), Stiffness::Soft, "{name} should be Soft");
        }
    }

    #[test]
    fn root_translation_moves_whole_skeleton() {
        let mut a = cub_skeleton();
        a.forward();
        let head_a = a.world_tip(a.id_of("head").unwrap());

        let mut b = cub_skeleton();
        b.set_root(Vec2::new(30.0, 40.0));
        b.forward();
        let head_b = b.world_tip(b.id_of("head").unwrap());

        assert!(approx(head_b.x - head_a.x, 8.0, 1e-3));
        assert!(approx(head_b.y - head_a.y, 4.0, 1e-3));
    }

    /// Visualise the skeleton — see `cargo test snapshot_moluun_cub_skeleton
    /// -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_skeleton() {
        use image::{Rgba, RgbaImage};
        use kokoro_rig::Bone;
        use std::path::PathBuf;

        let mut sk = cub_skeleton();
        sk.forward();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();

        // Sticks
        let mut sticks = RgbaImage::new(64, 64);
        for px in sticks.pixels_mut() {
            *px = Rgba([217, 199, 174, 255]);
        }
        kokoro_rig::debug::render_skeleton_overlay(
            &mut sticks,
            &sk,
            Rgba([59, 36, 24, 255]),
            Rgba([217, 13, 67, 255]),
            true,
        );
        save_with_4x(&sticks, &dir, "moluun_cub_skeleton");

        // Tinted silhouette
        let mut silh = RgbaImage::new(64, 64);
        for px in silh.pixels_mut() {
            *px = Rgba([217, 199, 174, 255]);
        }
        let color_for = |bone: &Bone| -> Rgba<u8> {
            match bone.name {
                "head" | "neck" | "spine" => Rgba([160, 76, 3, 255]), // OrangeDark
                "ear_front" | "ear_back" => Rgba([240, 136, 40, 255]),
                "eye" => Rgba([27, 19, 13, 255]),
                "snout" => Rgba([27, 19, 13, 255]),
                "tail_1" | "tail_2" | "tail_3" | "tail_4" | "tail_5" => {
                    Rgba([217, 103, 4, 255]) // Orange
                }
                "shoulder_front" | "leg_front" | "paw_front"
                | "hip_back" | "leg_back" | "paw_back" => Rgba([90, 54, 34, 255]), // BrownDark
                _ => Rgba([0, 0, 0, 0]),
            }
        };
        kokoro_rig::debug::render_skeleton_silhouette(&mut silh, &sk, color_for);
        save_with_4x(&silh, &dir, "moluun_cub_silhouette");

        eprintln!("moluun_cub: wrote skeleton + silhouette PNGs");
    }

    fn save_with_4x(img: &image::RgbaImage, dir: &std::path::Path, name: &str) {
        img.save(dir.join(format!("{name}.png"))).unwrap();
        let (w, h) = (img.width(), img.height());
        let mut up = image::RgbaImage::new(w * 4, h * 4);
        for y in 0..h {
            for x in 0..w {
                let p = *img.get_pixel(x, y);
                for dy in 0..4 {
                    for dx in 0..4 {
                        up.put_pixel(x * 4 + dx, y * 4 + dy, p);
                    }
                }
            }
        }
        up.save(dir.join(format!("{name}@4x.png"))).unwrap();
    }
}
