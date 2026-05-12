//! Spine renderer — wireframe phase.
//!
//! The cub is being rebuilt body-part by body-part. Step 1 is "get the
//! whole skeleton sitting correctly"; soft tissues (fat, skin, fur,
//! muscle bellies) come back once the bone layout reads right at every
//! body part. The tail is already in its finished anatomical render
//! (see `tail_render`); everything else paints as a stick figure for
//! now.
//!
//! This function is intentionally cheap and uniform: a thin NearBlack
//! line along every bone in the spine, with CyanBright joint dots at
//! the endpoints. Same primitives the debug overlay uses, but called
//! unconditionally from `skin::mod.rs` so the skeleton is visible in
//! production builds while we keep working on layout.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::BoneId;

use super::moluun::STANDALONE_SPINE_SEGMENTS;
use super::moluun_runtime::MoluunCubSpine;
use super::tail_render::LayerVisibility;

const BONE_COLOR:  Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const JOINT_COLOR: Rgba<u8> = Rgba(Palette::CyanBright.rgba(220));

/// Paint the spine as a wireframe: bone lines + joint dots, no soft
/// tissue. `vis` is accepted for signature symmetry with the tail
/// renderer but ignored — there's no tissue to strip yet.
pub fn paint_anatomical_spine(
    img: &mut RgbaImage,
    spine: &MoluunCubSpine,
    _vis: LayerVisibility,
) {
    let skeleton = &spine.body.skeleton;
    if skeleton.dirty() {
        return;
    }
    for seg in 0..STANDALONE_SPINE_SEGMENTS {
        let bone_id = BoneId((seg + 1) as u16);
        let base = skeleton.world_base(bone_id);
        let tip  = skeleton.world_tip(bone_id);
        let (x0, y0) = (base.x.round() as i32, base.y.round() as i32);
        let (x1, y1) = (tip.x.round()  as i32, tip.y.round()  as i32);
        draw_line(img, x0, y0, x1, y1, BONE_COLOR);
        draw_joint_dot(img, x0, y0, JOINT_COLOR);
        draw_joint_dot(img, x1, y1, JOINT_COLOR);
    }
}

fn draw_line(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let (mut x, mut y) = (x0, y0);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        put(img, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_joint_dot(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    put(img, cx,     cy,     color);
    put(img, cx - 1, cy,     color);
    put(img, cx + 1, cy,     color);
    put(img, cx,     cy - 1, color);
    put(img, cx,     cy + 1, color);
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

    /// Sitting-cub composition: spine wireframe goes top→bottom from
    /// (35, 24) to (35, 46); the tail's anatomical render then trails
    /// left from the sacral tip.
    #[test]
    #[ignore]
    fn snapshot_sitting_cub_skeleton() {
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
        upscale_save(&img, "sitting_cub_skeleton");
    }
}
