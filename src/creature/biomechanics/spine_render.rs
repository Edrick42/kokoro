//! Anatomical torso renderer — **front-facing** sitting-cub silhouette.
//!
//! The cub faces the camera (Pokémon-sprite style), so what we paint
//! isn't a side profile of the spine — it's the **frontal projection
//! of the trunk**. The spine bones still live inside the body and
//! drive future breathing/arching, but they're hidden behind the
//! visible silhouette.
//!
//! Front-view torso shape decomposes into **two stacked ovals**:
//!
//! - **Shoulder oval** — top of the trunk, narrower (chest width).
//! - **Hip oval** — bottom of the trunk, wider (sitting cub spreads
//!   at the hips).
//!
//! Their union forms one continuous chubby oblong, smoothly tapering
//! from narrow top to wide bottom. Both centred on the canvas vertical
//! axis since the cub is symmetric in front view.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::BoneId;

use super::moluun_runtime::MoluunCubSpine;
use super::tail_render::LayerVisibility;

const FUR_BODY:  Rgba<u8> = Rgba(Palette::Orange.rgba(255));
const FUR_EDGE:  Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const SKIN_BODY: Rgba<u8> = Rgba(Palette::Tan.rgba(255));
const SKIN_EDGE: Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
const FAT_BODY:  Rgba<u8> = Rgba(Palette::Gold.rgba(255));
const FAT_EDGE:  Rgba<u8> = Rgba(Palette::GoldDark.rgba(255));

/// Per-layer perpendicular padding, in canvas pixels.
const FUR_PAD:  f32 = 1.8;
const SKIN_PAD: f32 = 0.5;
const FAT_PAD:  f32 = 0.5;

#[derive(Copy, Clone, Debug)]
enum Region {
    Ellipse { cx: f32, cy: f32, rx: f32, ry: f32 },
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
        }
    }
    fn expanded(&self, pad: f32) -> Region {
        match *self {
            Region::Ellipse { cx, cy, rx, ry } =>
                Region::Ellipse { cx, cy, rx: rx + pad, ry: ry + pad },
        }
    }
}

/// Anatomical shape presets for the **front-facing** cub torso. Both
/// ovals are centred horizontally on the canvas; vertical placement
/// is derived from the spine bones in [`cub_torso_regions`] so the
/// silhouette tracks if the spine ever shifts position.
struct TorsoShape {
    shoulder_rx: f32,
    shoulder_ry: f32,
    hip_rx:      f32,
    hip_ry:      f32,
}

fn cub_torso_shape() -> TorsoShape {
    // Chubby-cub proportions: hips noticeably wider than shoulders.
    // The ry values are sized so each oval REACHES into the other
    // (their bounding boxes overlap by ~6 px vertically); without
    // that overlap the union shows a snowman waist in the middle.
    TorsoShape {
        shoulder_rx: 9.0,
        shoulder_ry: 7.0,
        hip_rx:      11.5,
        hip_ry:      8.0,
    }
}

/// Build the two anatomical regions for the cub torso in **front view**.
/// Returns `(shoulder, hip)`.
///
/// Vertical positions come from the spine bones so the silhouette
/// anchors correctly along the cub's height — the shoulder oval sits
/// over the upper spine, the hip oval over the lower spine. Both are
/// centred horizontally on the canvas (the cub is symmetric in front
/// view).
fn cub_torso_regions(spine: &MoluunCubSpine) -> (Region, Region) {
    let sk = &spine.body.skeleton;
    let shape = cub_torso_shape();

    let cervical_top = sk.world_base(BoneId(1));
    let sacral_bot   = sk.world_tip(BoneId(5));
    let canvas_center_x = 32.0;

    // Place the two oval centres CLOSE TOGETHER along the spine so
    // their bounding boxes overlap heavily (≈ 6 px). The lower the
    // ratios below, the more the shoulder and hip blend into one
    // continuous silhouette; without enough overlap the union shows a
    // visible "snowman waist".
    let shoulder_cy = cervical_top.y + shape.shoulder_ry;     // centre at top + full radius down
    let hip_cy      = sacral_bot.y   - shape.hip_ry;          // centre at bottom + full radius up

    let shoulder = Region::Ellipse {
        cx: canvas_center_x, cy: shoulder_cy,
        rx: shape.shoulder_rx, ry: shape.shoulder_ry,
    };
    let hip = Region::Ellipse {
        cx: canvas_center_x, cy: hip_cy,
        rx: shape.hip_rx, ry: shape.hip_ry,
    };

    (shoulder, hip)
}

/// Paint the cub's torso as a continuous chubby silhouette. The
/// visible shape is the outermost present layer's colour — a furred
/// cub reads as solid orange everywhere, exactly like the real
/// animal.
pub fn paint_anatomical_spine(
    img: &mut RgbaImage,
    spine: &MoluunCubSpine,
    vis: LayerVisibility,
) {
    let sk = &spine.body.skeleton;
    if sk.dirty() {
        return;
    }
    let (shoulder, hip) = cub_torso_regions(spine);

    let (pad, body, edge) = if vis.fur {
        (FAT_PAD + SKIN_PAD + FUR_PAD, FUR_BODY, FUR_EDGE)
    } else if vis.skin {
        (FAT_PAD + SKIN_PAD, SKIN_BODY, SKIN_EDGE)
    } else if vis.fat {
        (FAT_PAD, FAT_BODY, FAT_EDGE)
    } else {
        return;
    };

    let regions = [shoulder.expanded(pad), hip.expanded(pad)];
    paint_union(img, &regions, body, edge);
}

/// Paint the union of `regions` as a continuous filled shape with a
/// 1-pixel outline.
fn paint_union(img: &mut RgbaImage, regions: &[Region], body: Rgba<u8>, edge: Rgba<u8>) {
    let w = img.width()  as i32;
    let h = img.height() as i32;
    let mut mask = vec![false; (w * h) as usize];

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

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if !mask[(y * w + x) as usize] { continue; }
            let on_edge = (x > 0    && !mask[(y * w + (x - 1)) as usize])
                       || (x + 1 < w && !mask[(y * w + (x + 1)) as usize])
                       || (y > 0    && !mask[((y - 1) * w + x) as usize])
                       || (y + 1 < h && !mask[((y + 1) * w + x) as usize])
                       || (x == 0 || x == w - 1 || y == 0 || y == h - 1);
            if on_edge {
                put(img, x, y, edge);
            }
        }
    }
}

fn bbox(r: &Region) -> (i32, i32, i32, i32) {
    match *r {
        Region::Ellipse { cx, cy, rx, ry } => {
            ((cx - rx).floor() as i32, (cy - ry).floor() as i32,
             (cx + rx).ceil()  as i32, (cy + ry).ceil()  as i32)
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
    use kokoro_rig::Vec2;
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

    /// Front-view torso only — no head, no legs, no tail yet. Pure
    /// chubby silhouette to verify the shape before composing other
    /// body parts in.
    #[test]
    #[ignore]
    fn snapshot_sitting_cub_torso() {
        use super::super::moluun::cub_spine_body_for_creature;
        use super::super::moluun_runtime::MoluunCubSpine;
        let mut spine_body = cub_spine_body_for_creature(
            0.5, 0.5, 0.5,
            22.0,
            Vec2::new(32.0, 24.0),  // centred on canvas mid (front view)
            std::f32::consts::FRAC_PI_2,
        );
        spine_body.skeleton.forward();
        let spine = MoluunCubSpine { body: spine_body, sim_time: 0.0 };

        let mut img = RgbaImage::new(64, 64);
        for px in img.pixels_mut() { *px = Rgba([0, 0, 0, 0]); }
        paint_anatomical_spine(&mut img, &spine, LayerVisibility::ALL);
        upscale_save(&img, "sitting_cub_torso");
    }
}
