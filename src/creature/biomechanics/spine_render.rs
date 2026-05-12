//! Anatomical spine renderer — five-vertebra trunk painted from live
//! `Tissue` thicknesses on each bone. Same disc-tube approach as
//! `tail_render::paint_anatomical_tail`, but with two differences that
//! match real spine anatomy:
//!
//! 1. **Uniform fur** — no ring banding. Body fur on a Moluun is a
//!    continuous coat; the banded look is a tail-only trait.
//! 2. **Uniform fat / skin** — taper is set in the helpers in
//!    `moluun.rs` and read off `bone.tissue` here, but the values
//!    chosen there are constant per segment.
//!
//! The renderer takes the same `LayerVisibility` flags as the tail so
//! the dev panel toggles apply consistently across body parts.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::BoneId;

use super::moluun::STANDALONE_SPINE_SEGMENTS;
use super::moluun_runtime::MoluunCubSpine;
use super::tail_render::LayerVisibility;

const FUR_BODY:  Rgba<u8> = Rgba(Palette::Orange.rgba(255));
const FUR_EDGE:  Rgba<u8> = Rgba(Palette::OrangeDark.rgba(255));
const SKIN_BODY: Rgba<u8> = Rgba(Palette::Tan.rgba(255));
const SKIN_EDGE: Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
const FAT_BODY:  Rgba<u8> = Rgba(Palette::Gold.rgba(255));
const FAT_EDGE:  Rgba<u8> = Rgba(Palette::GoldDark.rgba(255));
const OUTLINE_PAD: f32 = 0.5;

/// Paint the cub's spine anatomically. Same outside-in disc-stamping
/// pattern the tail renderer uses; see `tail_render` for the why.
pub fn paint_anatomical_spine(
    img: &mut RgbaImage,
    spine: &MoluunCubSpine,
    vis: LayerVisibility,
) {
    let skeleton = &spine.body.skeleton;
    if skeleton.dirty() {
        return;
    }
    for seg in 0..STANDALONE_SPINE_SEGMENTS {
        let bone_id = BoneId((seg + 1) as u16);
        let bone = skeleton.bone(bone_id);
        let base = skeleton.world_base(bone_id);
        let tip  = skeleton.world_tip(bone_id);
        let (flex_t, ext_t) = muscle_thickness_for(spine, bone_id);

        let fat_th  = if vis.fat  { bone.tissue.fat_thickness  } else { 0.0 };
        let skin_th = if vis.skin { bone.tissue.skin_thickness } else { 0.0 };
        let fur_th  = if vis.fur  { bone.tissue.fur_length     } else { 0.0 };

        if fur_th > 0.0 {
            stamp_tube(img, base, tip,
                       flex_t + fat_th + skin_th + fur_th,
                       ext_t + fat_th + skin_th + fur_th,
                       FUR_BODY, FUR_EDGE);
        }
        if skin_th > 0.0 {
            stamp_tube(img, base, tip,
                       flex_t + fat_th + skin_th,
                       ext_t + fat_th + skin_th,
                       SKIN_BODY, SKIN_EDGE);
        } else if fat_th > 0.0 {
            stamp_tube(img, base, tip,
                       flex_t + fat_th, ext_t + fat_th,
                       FAT_BODY, FAT_EDGE);
        }
    }
}

fn muscle_thickness_for(spine: &MoluunCubSpine, bone: BoneId) -> (f32, f32) {
    if let Some(a) = spine.body.actuators.iter().find(|a| a.bone == bone) {
        (a.muscles.flexor.current_thickness(), a.muscles.extensor.current_thickness())
    } else {
        (1.0, 1.0)
    }
}

fn stamp_tube(
    img: &mut RgbaImage,
    base: kokoro_rig::Vec2,
    tip: kokoro_rig::Vec2,
    flex_t: f32,
    ext_t: f32,
    body_color: Rgba<u8>,
    edge_color: Rgba<u8>,
) {
    let dx = tip.x - base.x;
    let dy = tip.y - base.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let nx = -dy / len;
    let ny = dx / len;
    let perp_offset = (flex_t - ext_t) * 0.5;
    let radius = (flex_t + ext_t) * 0.5;
    if radius < 0.3 {
        return;
    }
    let samples = (len.ceil() as i32).max(2);
    for k in 0..=samples {
        let s = k as f32 / samples as f32;
        let cx = base.x + dx * s + nx * perp_offset;
        let cy = base.y + dy * s + ny * perp_offset;
        fill_disc(img, cx, cy, radius + OUTLINE_PAD, edge_color);
    }
    for k in 0..=samples {
        let s = k as f32 / samples as f32;
        let cx = base.x + dx * s + nx * perp_offset;
        let cy = base.y + dy * s + ny * perp_offset;
        fill_disc(img, cx, cy, radius, body_color);
    }
}

fn fill_disc(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    if radius <= 0.0 {
        return;
    }
    let r_sq = radius * radius;
    let x0 = (cx - radius - 0.5).floor() as i32;
    let x1 = (cx + radius + 0.5).ceil() as i32;
    let y0 = (cy - radius - 0.5).floor() as i32;
    let y1 = (cy + radius + 0.5).ceil() as i32;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r_sq {
                put(img, x, y, color);
            }
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
            .join("target")
            .join("sprite-snapshots");
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
            for dy in 0..4 { for dx in 0..4 {
                up.put_pixel(x * 4 + dx, y * 4 + dy, p);
            }}
        }}
        let _ = up.save(dir.join(format!("{name}@4x.png")));
    }

    /// Snapshot the spine + tail composed. Spine builds from median
    /// genes, tail anchors at the spine's sacral tip via a manual
    /// `set_root` call (the Bevy bridge isn't running in tests).
    #[test]
    #[ignore]
    fn snapshot_spine_with_tail() {
        use super::super::moluun::{cub_spine_body_for_creature, cub_tail_body_for_creature, STANDALONE_SPINE_SEGMENTS};
        use super::super::moluun_runtime::{MoluunCubSpine, MoluunCubTail};
        let genes = crate::genome::TailGenes::default();
        let mut spine_body = cub_spine_body_for_creature(
            0.5, 0.5, 0.5,
            20.0, Vec2::new(48.0, 32.0), std::f32::consts::PI,
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
        upscale_save(&img, "spine_with_tail");
    }
}
