//! Anatomical tail renderer — the *visible* shape of the tail is derived
//! from the live muscle state, not a fixed brush.
//!
//! Each bone segment is rendered as an asymmetric tube of overlapping
//! filled discs. Two muscle thicknesses determine the tube at each
//! point:
//!
//! - The **flexor** (firing side `+perp`) bulges by
//!   `rest_thickness × (1 + 0.4 × activation)` — same volume-conservation
//!   approximation that [`kokoro_body::muscle::MUSCLE_BULGE_FACTOR`]
//!   codifies.
//! - The **extensor** does the same on `−perp`.
//!
//! A disc per stamp gives crisp pixel-art silhouettes at any scale and
//! lets adjacent segments naturally blend together without scanline
//! gaps at fractional thicknesses. The disc is *shifted* off the bone
//! axis toward whichever side is firing harder, so the tail's centre
//! line drifts (skin pushed by the contracting muscle) and its envelope
//! grows on that side — the same way a real wagging tail looks.
//!
//! Render order per segment:
//! 1. Edge disc in `Palette::OrangeDark` (slight outline halo)
//! 2. Body disc in `Palette::Orange` (or `Palette::OffWhite` if the
//!    segment is on the ring stride for that signature banded look)
//!
//! The biomechanics debug overlay (bones, joints, muscles) is layered
//! *on top* of this by `skin::mod.rs`; this renderer paints only the
//! "skin" the player sees in the released game.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::BoneId;

use super::moluun::STANDALONE_TAIL_SEGMENTS;
use super::moluun_runtime::MoluunCubTail;

const BODY: Rgba<u8> = Rgba(Palette::Orange.rgba(255));
const EDGE: Rgba<u8> = Rgba(Palette::OrangeDark.rgba(255));
const RING: Rgba<u8> = Rgba(Palette::OffWhite.rgba(255));
/// Every Nth segment stamps a cream disc instead of orange — the cub's
/// banded-tail identity. Stride 4 over 16 segments gives 4 rings.
const RING_STRIDE: usize = 4;
/// Outline halo thickness (pixels) added around every body disc.
const OUTLINE_PAD: f32 = 0.5;

/// Bit flags for which anatomical layers contribute to the rendered
/// silhouette. The five-layer stack from inside to outside is bone,
/// muscle, fat, skin, fur — the muscle layer is always on (it's the
/// envelope's core), but fat / skin / fur are independently
/// toggleable so the dev panel can visualise what each contributes.
#[derive(Copy, Clone, Debug)]
pub struct LayerVisibility {
    pub fat:  bool,
    pub skin: bool,
    pub fur:  bool,
}

impl LayerVisibility {
    pub const ALL: Self = Self { fat: true, skin: true, fur: true };
}

/// Paint the cub's tail anatomically. The visible silhouette stacks
/// `muscle + fat + skin + fur` perpendicular to the bone; each non-muscle
/// layer can be stripped by clearing its flag in `vis` so the dev
/// inspector can see what each tissue contributes.
pub fn paint_anatomical_tail(
    img: &mut RgbaImage,
    tail: &MoluunCubTail,
    vis: LayerVisibility,
) {
    let skeleton = &tail.body.skeleton;
    if skeleton.dirty() {
        return;
    }
    for seg in 0..STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId((seg + 1) as u16);
        let base = skeleton.world_base(bone_id);
        let tip  = skeleton.world_tip(bone_id);
        let (flex_t, ext_t) = thicknesses_for(tail, bone_id);
        // Halo layers are symmetric (same on both perp sides). Muscle
        // already carries the asymmetric flex/ext difference. Toggling
        // a layer to off sets its contribution to zero — i.e. the dev
        // viewer literally sees the cub minus that tissue.
        let fat  = if vis.fat  { tail.fat_thicknesses.get(seg).copied().unwrap_or(0.0)  } else { 0.0 };
        let skin = if vis.skin { tail.skin_thicknesses.get(seg).copied().unwrap_or(0.0) } else { 0.0 };
        let fur  = if vis.fur  { tail.fur_lengths.get(seg).copied().unwrap_or(0.0)      } else { 0.0 };
        let halo = fat + skin + fur;
        let is_ring = seg > 0 && seg % RING_STRIDE == 0;
        let body_color = if is_ring { RING } else { BODY };
        stamp_tube(img, base, tip, flex_t + halo, ext_t + halo, body_color);
    }
}

/// Stamp an asymmetric disc-tube along the segment from `base` to `tip`.
/// `flex_t` is the muscle thickness on `+perp`; `ext_t` on `−perp`.
/// Two passes: outline halo first, body fill on top.
fn stamp_tube(
    img: &mut RgbaImage,
    base: kokoro_rig::Vec2,
    tip: kokoro_rig::Vec2,
    flex_t: f32,
    ext_t: f32,
    body_color: Rgba<u8>,
) {
    let dx = tip.x - base.x;
    let dy = tip.y - base.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return; // degenerate
    }
    let nx = -dy / len;
    let ny = dx / len;

    // The disc that approximates this segment's cross-section is
    // centred on the bone axis, offset perpendicular by (flex - ext)/2
    // so the firing side gets more of the bulge. Its radius is the
    // average of the two muscle thicknesses — the *radial extent* the
    // tube takes up around its (shifted) centre line.
    let perp_offset = (flex_t - ext_t) * 0.5;
    let radius = (flex_t + ext_t) * 0.5;
    if radius < 0.3 {
        return; // sub-pixel — nothing to draw
    }

    // Roughly one stamp per pixel of bone length so adjacent stamps
    // overlap and the tube reads as continuous instead of beaded.
    let samples = (len.ceil() as i32).max(2);
    for k in 0..=samples {
        let s = k as f32 / samples as f32;
        let cx = base.x + dx * s + nx * perp_offset;
        let cy = base.y + dy * s + ny * perp_offset;
        fill_disc(img, cx, cy, radius + OUTLINE_PAD, EDGE);
    }
    for k in 0..=samples {
        let s = k as f32 / samples as f32;
        let cx = base.x + dx * s + nx * perp_offset;
        let cy = base.y + dy * s + ny * perp_offset;
        fill_disc(img, cx, cy, radius, body_color);
    }
}

/// Lookup the current_thickness for the muscle pair attached to `bone`.
/// Returns rest thickness (or 1.0 fallback) if the actuator isn't found.
fn thicknesses_for(tail: &MoluunCubTail, bone: BoneId) -> (f32, f32) {
    if let Some(a) = tail.body.actuators.iter().find(|a| a.bone == bone) {
        (a.muscles.flexor.current_thickness(), a.muscles.extensor.current_thickness())
    } else {
        (1.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Pixel primitives
// ---------------------------------------------------------------------------

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
    use crate::genome::TailGenes;
    use kokoro_rig::Vec2;
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

    /// Tail at rest — both muscles slack. Visualises the natural taper
    /// from base to tip without any wave deformation.
    #[test]
    #[ignore]
    fn snapshot_anatomical_tail_rest() {
        let genes = TailGenes::default();
        let body = super::super::moluun::cub_tail_body_for_creature(
            &genes,
            32.0,
            Vec2::new(48.0, 32.0),
            std::f32::consts::PI,
        );
        let fur_lengths = (0..STANDALONE_TAIL_SEGMENTS).map(|i| {
            let t = (i as f32) / ((STANDALONE_TAIL_SEGMENTS - 1) as f32);
            2.5 * (std::f32::consts::PI * t).sin()
        }).collect();
        let fat_thicknesses = (0..STANDALONE_TAIL_SEGMENTS).map(|i| {
            let t = (i as f32) / ((STANDALONE_TAIL_SEGMENTS - 1) as f32);
            0.6 * (1.0 - t) + 0.2 * t
        }).collect();
        let skin_thicknesses = vec![0.3_f32; STANDALONE_TAIL_SEGMENTS];
        let mut tail = super::super::moluun_runtime::MoluunCubTail {
            body,
            sim_time: 0.0,
            last_intent: vec![kokoro_body::actuation::PairIntent::rest(); 16],
            fat_thicknesses,
            skin_thicknesses,
            fur_lengths,
        };
        let mut img = RgbaImage::new(64, 64);
        for px in img.pixels_mut() { *px = Rgba([0, 0, 0, 0]); }
        tail.body.skeleton.forward();
        paint_anatomical_tail(&mut img, &tail, LayerVisibility::ALL);
        upscale_save(&img, "anatomical_tail_rest");
    }

    /// Tail with every flexor fired at 1.0 — every disc shifts toward
    /// +perp and bulges to its 40%-bigger contracted thickness. Looks
    /// like a tail held curled rigidly to one side; useful as a sanity
    /// check that the asymmetry plumbs through.
    #[test]
    #[ignore]
    fn snapshot_anatomical_tail_flexed() {
        let genes = TailGenes::default();
        let body = super::super::moluun::cub_tail_body_for_creature(
            &genes,
            32.0,
            Vec2::new(48.0, 32.0),
            std::f32::consts::PI,
        );
        let fur_lengths = (0..STANDALONE_TAIL_SEGMENTS).map(|i| {
            let t = (i as f32) / ((STANDALONE_TAIL_SEGMENTS - 1) as f32);
            2.5 * (std::f32::consts::PI * t).sin()
        }).collect();
        let fat_thicknesses = (0..STANDALONE_TAIL_SEGMENTS).map(|i| {
            let t = (i as f32) / ((STANDALONE_TAIL_SEGMENTS - 1) as f32);
            0.6 * (1.0 - t) + 0.2 * t
        }).collect();
        let skin_thicknesses = vec![0.3_f32; STANDALONE_TAIL_SEGMENTS];
        let mut tail = super::super::moluun_runtime::MoluunCubTail {
            body,
            sim_time: 0.0,
            last_intent: vec![kokoro_body::actuation::PairIntent::rest(); 16],
            fat_thicknesses,
            skin_thicknesses,
            fur_lengths,
        };
        tail.body.skeleton.forward();
        for a in tail.body.actuators.iter_mut() {
            a.muscles.flexor.activation = 1.0;
            a.muscles.extensor.activation = 0.0;
        }
        let mut img = RgbaImage::new(64, 64);
        for px in img.pixels_mut() { *px = Rgba([0, 0, 0, 0]); }
        paint_anatomical_tail(&mut img, &tail, LayerVisibility::ALL);
        upscale_save(&img, "anatomical_tail_flexed");
    }
}
