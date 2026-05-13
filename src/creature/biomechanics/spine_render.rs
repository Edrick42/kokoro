//! Anatomical torso renderer — sitting-cub pear silhouette.
//!
//! Real animal torsos don't look like a stack of cylindrical vertebrae
//! — skin and fur produce **one continuous shape** that wraps over the
//! underlying muscle/fat/bones. For a sitting cub that shape is a
//! **pear**: a small chest oval at the top, a larger softer belly oval
//! at the bottom, joined to a narrow neck capsule going up.
//!
//! This renderer paints the union of those three regions as one
//! contiguous silhouette. Each layer (fur → skin → fat) is the union
//! of the regions expanded by that layer's padding, so:
//!
//! - the outermost halo is fur on every region, blended at the joints
//!   where chest meets belly meets neck
//! - skin sits just inside fur
//! - fat sits inside skin and only shows when skin is stripped in dev
//! - muscle / bone are interior and only visible via the debug overlay
//!
//! The chest and belly ovals' **positions** are derived from the
//! spine bones' world tips, so when the spine eventually starts
//! moving (breathing, mood-driven arching) the silhouette tracks. The
//! ovals' **sizes** plus the belly's forward offset are anatomy
//! presets that live in `cub_torso_shape`.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::BoneId;

use super::moluun_runtime::MoluunCubSpine;
use super::tail_render::LayerVisibility;

const FUR_BODY:  Rgba<u8> = Rgba(Palette::Orange.rgba(255));
const FUR_EDGE:  Rgba<u8> = Rgba(Palette::OrangeDark.rgba(255));
const SKIN_BODY: Rgba<u8> = Rgba(Palette::Tan.rgba(255));
const SKIN_EDGE: Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
const FAT_BODY:  Rgba<u8> = Rgba(Palette::Gold.rgba(255));
const FAT_EDGE:  Rgba<u8> = Rgba(Palette::GoldDark.rgba(255));

/// Per-layer perpendicular padding, in canvas pixels. Each layer's
/// "envelope" is the underlying shape (skin/fat/muscle) expanded by
/// these distances. fur > skin > fat is the real anatomical stack.
const FUR_PAD:  f32 = 1.8;
const SKIN_PAD: f32 = 0.5;
const FAT_PAD:  f32 = 0.5;

/// Geometric primitive that contributes to the torso silhouette.
#[derive(Copy, Clone, Debug)]
enum Region {
    /// Axis-aligned filled ellipse.
    Ellipse { cx: f32, cy: f32, rx: f32, ry: f32 },
    /// Filled capsule (line segment with rounded width).
    Capsule { ax: f32, ay: f32, bx: f32, by: f32, r: f32 },
}

impl Region {
    fn contains(&self, x: f32, y: f32) -> bool {
        match *self {
            Region::Ellipse { cx, cy, rx, ry } => {
                if rx <= 0.0 || ry <= 0.0 { return false; }
                let nx = (x - cx) / rx;
                let ny = (y - cy) / ry;
                nx * nx + ny * ny <= 1.0
            }
            Region::Capsule { ax, ay, bx, by, r } => {
                let dx = bx - ax;
                let dy = by - ay;
                let len_sq = (dx * dx + dy * dy).max(1e-6);
                let t = (((x - ax) * dx + (y - ay) * dy) / len_sq).clamp(0.0, 1.0);
                let px = ax + dx * t;
                let py = ay + dy * t;
                let ddx = x - px;
                let ddy = y - py;
                ddx * ddx + ddy * ddy <= r * r
            }
        }
    }

    /// Same shape, expanded perpendicular by `pad`.
    fn expanded(&self, pad: f32) -> Region {
        match *self {
            Region::Ellipse { cx, cy, rx, ry } =>
                Region::Ellipse { cx, cy, rx: rx + pad, ry: ry + pad },
            Region::Capsule { ax, ay, bx, by, r } =>
                Region::Capsule { ax, ay, bx, by, r: r + pad },
        }
    }
}

/// Anatomical shape presets for the cub's torso. Sizes are species
/// constants here; positions get filled in per-frame from the spine
/// bones. Centres are offset *forward* (toward `+ x_forward`) because
/// a sitting cub bulges its chest and belly outward in front of the
/// vertebral column.
struct TorsoShape {
    /// Half-width of the neck capsule (radius around the cervical bone).
    neck_radius:    f32,
    /// Chest oval half-axes (rx = belly-direction, ry = along spine).
    chest_rx:       f32,
    chest_ry:       f32,
    /// Belly oval half-axes.
    belly_rx:       f32,
    belly_ry:       f32,
    /// How far chest centre offsets forward (toward belly side) from
    /// the spine, in canvas pixels.
    chest_offset:   f32,
    /// How far belly centre offsets forward. Always greater than
    /// chest_offset so the cub's profile has a forward-bulged belly.
    belly_offset:   f32,
}

fn cub_torso_shape() -> TorsoShape {
    TorsoShape {
        neck_radius:    1.4,
        chest_rx:       3.6,
        chest_ry:       4.5,
        belly_rx:       5.4,
        belly_ry:       5.5,
        chest_offset:   1.2,
        belly_offset:   2.4,
    }
}

/// Build the 3 anatomical regions for the cub torso from live spine
/// bone positions. Returns `(neck, chest, belly)` so the caller can
/// expand and union them per layer.
///
/// Bone mapping (1-based, matching `BoneId`):
///   1 = cervical, 2 = thoracic_a, 3 = thoracic_b,
///   4 = lumbar,   5 = sacral
fn cub_torso_regions(spine: &MoluunCubSpine) -> (Region, Region, Region) {
    let sk = &spine.body.skeleton;
    let shape = cub_torso_shape();

    // Forward-bulge direction: perpendicular to the cervical bone,
    // rotated 90° counter-clockwise (so a spine going DOWN gives a
    // forward = +x direction for a cub facing right). Computed from
    // the cervical bone so the forward direction tracks any spine
    // rotation in the future.
    let cervical_base = sk.world_base(BoneId(1));
    let cervical_tip  = sk.world_tip(BoneId(1));
    let bone_dx = cervical_tip.x - cervical_base.x;
    let bone_dy = cervical_tip.y - cervical_base.y;
    let len = (bone_dx * bone_dx + bone_dy * bone_dy).sqrt().max(1e-4);
    let forward_x =  bone_dy / len; // rotate (dx,dy) by -90°: (dy, -dx)
    let forward_y = -bone_dx / len;

    // Neck capsule: along the cervical bone.
    let neck = Region::Capsule {
        ax: cervical_base.x, ay: cervical_base.y,
        bx: cervical_tip.x,  by: cervical_tip.y,
        r:  shape.neck_radius,
    };

    // Chest oval: spans thoracic_a base → thoracic_b tip.
    let chest_top = sk.world_base(BoneId(2));
    let chest_bot = sk.world_tip(BoneId(3));
    let chest_mid_x = (chest_top.x + chest_bot.x) * 0.5 + forward_x * shape.chest_offset;
    let chest_mid_y = (chest_top.y + chest_bot.y) * 0.5 + forward_y * shape.chest_offset;
    let chest = Region::Ellipse {
        cx: chest_mid_x, cy: chest_mid_y,
        rx: shape.chest_rx, ry: shape.chest_ry,
    };

    // Belly oval: spans lumbar base → sacral tip, bulged forward.
    let belly_top = sk.world_base(BoneId(4));
    let belly_bot = sk.world_tip(BoneId(5));
    let belly_mid_x = (belly_top.x + belly_bot.x) * 0.5 + forward_x * shape.belly_offset;
    let belly_mid_y = (belly_top.y + belly_bot.y) * 0.5 + forward_y * shape.belly_offset;
    let belly = Region::Ellipse {
        cx: belly_mid_x, cy: belly_mid_y,
        rx: shape.belly_rx, ry: shape.belly_ry,
    };

    (neck, chest, belly)
}

/// Paint the cub's torso as a continuous pear silhouette. The visible
/// shape is the **outermost layer present**: a furred cub reads as
/// solid orange everywhere (skin and fat are hidden under fur, exactly
/// as in a real animal). Stripping fur in dev reveals the skin shape;
/// stripping skin + fur reveals fat; stripping all three reveals
/// nothing (the dev viewer is left with the bone/muscle debug
/// overlays).
pub fn paint_anatomical_spine(
    img: &mut RgbaImage,
    spine: &MoluunCubSpine,
    vis: LayerVisibility,
) {
    let sk = &spine.body.skeleton;
    if sk.dirty() {
        return;
    }
    let (neck, chest, belly) = cub_torso_regions(spine);

    // The pad of the outermost present layer = how far the silhouette
    // extends past the bare muscle envelope. Layers stack inside-out
    // as muscle → fat → skin → fur, so when fur is on the full pad is
    // the sum of every layer's contribution.
    let (pad, body, edge) = if vis.fur {
        (FAT_PAD + SKIN_PAD + FUR_PAD, FUR_BODY, FUR_EDGE)
    } else if vis.skin {
        (FAT_PAD + SKIN_PAD, SKIN_BODY, SKIN_EDGE)
    } else if vis.fat {
        (FAT_PAD, FAT_BODY, FAT_EDGE)
    } else {
        return; // body has been peeled to bone + muscle, both debug-overlay only
    };

    let regions = [
        neck .expanded(pad),
        chest.expanded(pad),
        belly.expanded(pad),
    ];
    paint_union(img, &regions, body, edge);
}

/// Paint the union of `regions` as a continuous filled shape with a
/// 1-pixel outline. Two passes: first build the inside-mask, then
/// scan it for boundary pixels (any inside pixel that has at least
/// one outside-neighbour).
fn paint_union(img: &mut RgbaImage, regions: &[Region], body: Rgba<u8>, edge: Rgba<u8>) {
    let w = img.width()  as i32;
    let h = img.height() as i32;
    let mut mask = vec![false; (w * h) as usize];

    // Compute bounding box of all regions so we don't scan the whole
    // canvas — typical torso covers maybe 1/8 of the 64×64 image.
    let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
    let (mut max_x, mut max_y) = (i32::MIN, i32::MIN);
    for r in regions {
        let (rx0, ry0, rx1, ry1) = bbox(r);
        min_x = min_x.min(rx0); min_y = min_y.min(ry0);
        max_x = max_x.max(rx1); max_y = max_y.max(ry1);
    }
    let min_x = min_x.max(0);
    let min_y = min_y.max(0);
    let max_x = max_x.min(w - 1);
    let max_y = max_y.min(h - 1);
    if min_x > max_x || min_y > max_y {
        return;
    }

    // Pass 1: fill mask + paint body color.
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let inside = regions.iter().any(|r| r.contains(fx, fy));
            if inside {
                mask[(y * w + x) as usize] = true;
                put(img, x, y, body);
            }
        }
    }

    // Pass 2: overwrite boundary pixels with edge color.
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if !mask[(y * w + x) as usize] { continue; }
            let n_out = ((x > 0    && !mask[(y * w + (x - 1)) as usize])
                     ||  (x + 1 < w && !mask[(y * w + (x + 1)) as usize])
                     ||  (y > 0    && !mask[((y - 1) * w + x) as usize])
                     ||  (y + 1 < h && !mask[((y + 1) * w + x) as usize]))
                     || (x == 0 || x == w - 1 || y == 0 || y == h - 1);
            if n_out {
                put(img, x, y, edge);
            }
        }
    }
}

/// Integer bounding box (inclusive on all sides) of a region.
fn bbox(r: &Region) -> (i32, i32, i32, i32) {
    match *r {
        Region::Ellipse { cx, cy, rx, ry } => {
            ((cx - rx).floor() as i32, (cy - ry).floor() as i32,
             (cx + rx).ceil()  as i32, (cy + ry).ceil()  as i32)
        }
        Region::Capsule { ax, ay, bx, by, r } => {
            let x0 = ax.min(bx) - r;
            let x1 = ax.max(bx) + r;
            let y0 = ay.min(by) - r;
            let y1 = ay.max(by) + r;
            (x0.floor() as i32, y0.floor() as i32, x1.ceil() as i32, y1.ceil() as i32)
        }
    }
}

fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    img.put_pixel(x as u32, y as u32, color);
}

#[cfg(test)]
mod snapshot {
    use super::*;
    use kokoro_rig::{BoneId as BId, Vec2};
    use std::path::PathBuf;

    fn out_dir() -> PathBuf {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target").join("sprite-snapshots");
        let _ = std::fs::create_dir_all(&p);
        p
    }

    fn upscale_save(img: &RgbaImage, name: &str) {
        let dir = out_dir();
        let _ = img.save(dir.join(format!("{name}.png")));
        let (w, h) = (img.width(), img.height());
        let mut up = RgbaImage::new(w * 4, h * 4);
        for y in 0..h { for x in 0..w {
            let p = *img.get_pixel(x, y);
            for dy in 0..4 { for dx in 0..4 { up.put_pixel(x * 4 + dx, y * 4 + dy, p); }}
        }}
        let _ = up.save(dir.join(format!("{name}@4x.png")));
    }

    #[test]
    #[ignore]
    fn snapshot_sitting_cub_torso() {
        use super::super::moluun::{
            cub_spine_body_for_creature, cub_tail_body_for_creature,
            STANDALONE_SPINE_SEGMENTS,
        };
        use super::super::moluun_runtime::{MoluunCubSpine, MoluunCubTail};
        let genes = crate::genome::TailGenes::default();
        let mut spine_body = cub_spine_body_for_creature(
            0.5, 0.5, 0.5,
            22.0,
            Vec2::new(35.0, 24.0),
            std::f32::consts::FRAC_PI_2,
        );
        spine_body.skeleton.forward();
        let sacral_tip = spine_body.skeleton.world_tip(BId(STANDALONE_SPINE_SEGMENTS as u16));
        let mut tail_body = cub_tail_body_for_creature(
            &genes, 0.5, 0.5,
            32.0, sacral_tip, std::f32::consts::PI,
        );
        tail_body.skeleton.forward();

        let spine = MoluunCubSpine { body: spine_body, sim_time: 0.0 };
        let tail = MoluunCubTail {
            body: tail_body,
            sim_time: 0.0,
            last_intent: vec![kokoro_body::actuation::PairIntent::rest(); 16],
        };

        let mut img = RgbaImage::new(64, 64);
        for px in img.pixels_mut() { *px = Rgba([0, 0, 0, 0]); }
        paint_anatomical_spine(&mut img, &spine, LayerVisibility::ALL);
        super::super::tail_render::paint_anatomical_tail(&mut img, &tail, LayerVisibility::ALL);
        upscale_save(&img, "sitting_cub_torso");
    }
}
