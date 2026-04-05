//! Kokoro — Moluun (Forest Mammal) Sprite Generator
//!
//! Generates smooth low-poly sprites for all growth stages:
//!   egg → cub → young → adult → elder
//!
//! No pixelization — renders at full resolution with smooth volumetric shading.
//!
//! Usage: cargo run --bin generate_moluun_sprites

#[allow(dead_code)]
mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

// --- Color palette (not all colors used in every stage) ---
macro_rules! col {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        #[allow(dead_code)]
        const $name: Rgba<u8> = Rgba([$r, $g, $b, 255]);
    };
}
col!(BODY,        156, 232, 252);
col!(BODY_HI,     180, 240, 255);
col!(BODY_SH,     120, 195, 225);
col!(BODY_SH2,    100, 170, 205);
col!(EAR_INNER,   130, 200, 230);
col!(OUTLINE,      40,  45,  55);
col!(PUPIL,        30,  30,  40);
col!(PINK,        240, 160, 165);
col!(TEAR,        140, 195, 250);
col!(DORMANT_DOT,  60,  55,  75);
col!(LATERAL_EYE, 150, 100, 180);
col!(HORN_BASE,   130, 110,  90);
col!(HORN_MOSS,    80, 140,  70);
col!(FACE_MARK,   100, 180, 210);
col!(EGG_BASE,    200, 220, 230);
col!(EGG_SPOT,    160, 200, 220);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("moluun")
}

fn effects_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("shared").join("effects")
}

// ===================================================================
// EGG STAGE
// ===================================================================

fn gen_egg(dir: &Path) {
    let (w, h) = (200, 240);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (100.0_f32, 130.0_f32);

    // Egg shape — tall oval, narrower at top (real egg proportions)
    let egg_verts: Vec<(f32, f32)> = vec![
        (cx,        cy - 90.0),
        (cx + 30.0, cy - 85.0),
        (cx + 55.0, cy - 60.0),
        (cx + 65.0, cy - 20.0),
        (cx + 60.0, cy + 30.0),
        (cx + 45.0, cy + 65.0),
        (cx + 20.0, cy + 80.0),
        (cx,        cy + 85.0),
        (cx - 20.0, cy + 80.0),
        (cx - 45.0, cy + 65.0),
        (cx - 60.0, cy + 30.0),
        (cx - 65.0, cy - 20.0),
        (cx - 55.0, cy - 60.0),
        (cx - 30.0, cy - 85.0),
    ];

    smooth_shade(&mut img, &egg_verts, (cx, cy), EGG_BASE, (-0.3, -1.0));

    // Spots — species hint (lighter patches)
    let egg_shape = polygon_pixels(&egg_verts);
    for &(sx, sy, r) in &[(cx - 20.0, cy - 30.0, 12.0), (cx + 15.0, cy + 10.0, 10.0), (cx - 5.0, cy + 40.0, 8.0)] {
        let spot = ellipse_pixels(sx, sy, r, r * 0.8);
        for &(x, y) in &spot {
            if egg_shape.contains(&(x, y)) {
                let existing = *img.get_pixel(x as u32, y as u32);
                put(&mut img, x, y, lerp(existing, EGG_SPOT, 0.4));
            }
        }
    }

    flood_outline(&mut img, &egg_shape, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// ===================================================================
// CUB STAGE — round ball, huge eyes, stub limbs
// ===================================================================

fn gen_cub_body(dir: &Path) {
    let (w, h) = (256, 256);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (128.0_f32, 115.0_f32);

    // Body — one big chunky ball
    let body_verts: Vec<(f32, f32)> = vec![
        (cx,        cy - 80.0),
        (cx + 50.0, cy - 70.0),
        (cx + 80.0, cy - 20.0),
        (cx + 75.0, cy + 40.0),
        (cx + 45.0, cy + 75.0),
        (cx,        cy + 80.0),
        (cx - 45.0, cy + 75.0),
        (cx - 75.0, cy + 40.0),
        (cx - 80.0, cy - 20.0),
        (cx - 50.0, cy - 70.0),
    ];

    smooth_shade(&mut img, &body_verts, (cx, cy), BODY, (-0.3, -1.0));

    let body_shape = polygon_pixels(&body_verts);

    // Lighter belly
    let belly = ellipse_pixels(cx, cy + 20.0, 45.0, 35.0);
    for &(x, y) in &belly {
        if body_shape.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.3));
        }
    }

    // Rosy cheeks
    for &(chx, chy) in &[(cx - 45.0, cy + 10.0), (cx + 45.0, cy + 10.0)] {
        let cheek = ellipse_pixels(chx, chy, 15.0, 10.0);
        for &(x, y) in &cheek {
            if body_shape.contains(&(x, y)) {
                let existing = *img.get_pixel(x as u32, y as u32);
                put(&mut img, x, y, lerp(existing, PINK, 0.35));
            }
        }
    }

    // Dormant lateral eye dots
    put(&mut img, (cx - 68.0) as i32, (cy - 5.0) as i32, DORMANT_DOT);
    put(&mut img, (cx - 67.0) as i32, (cy - 5.0) as i32, DORMANT_DOT);
    put(&mut img, (cx + 67.0) as i32, (cy - 5.0) as i32, DORMANT_DOT);
    put(&mut img, (cx + 68.0) as i32, (cy - 5.0) as i32, DORMANT_DOT);

    // Stub feet
    let foot_l_v = [(cx - 35.0, cy + 73.0), (cx - 12.0, cy + 73.0), (cx - 24.0, cy + 95.0)];
    let foot_r_v = [(cx + 12.0, cy + 73.0), (cx + 35.0, cy + 73.0), (cx + 24.0, cy + 95.0)];
    smooth_shade(&mut img, &foot_l_v, (cx - 24.0, cy + 82.0), BODY_SH, (-0.3, -1.0));
    smooth_shade(&mut img, &foot_r_v, (cx + 24.0, cy + 82.0), BODY_SH, (0.3, -1.0));

    // Stub arms
    let arm_l_v = [(cx - 75.0, cy - 5.0), (cx - 90.0, cy + 15.0), (cx - 75.0, cy + 35.0)];
    let arm_r_v = [(cx + 75.0, cy - 5.0), (cx + 90.0, cy + 15.0), (cx + 75.0, cy + 35.0)];
    smooth_shade(&mut img, &arm_l_v, (cx - 82.0, cy + 15.0), BODY_SH, (-0.5, -1.0));
    smooth_shade(&mut img, &arm_r_v, (cx + 82.0, cy + 15.0), BODY_SH, (0.5, -1.0));

    let all: Shape = body_shape.iter()
        .chain(&polygon_pixels(&foot_l_v)).chain(&polygon_pixels(&foot_r_v))
        .chain(&polygon_pixels(&arm_l_v)).chain(&polygon_pixels(&arm_r_v))
        .copied().collect();

    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// ===================================================================
// YOUNG STAGE — proportions evening out, ears appearing
// ===================================================================

fn gen_young_body(dir: &Path) {
    let (w, h) = (280, 300);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (140.0_f32, 130.0_f32);

    // Body — more defined, less ball-like
    let body_verts: Vec<(f32, f32)> = vec![
        (cx,        cy - 85.0),
        (cx + 45.0, cy - 75.0),
        (cx + 75.0, cy - 25.0),
        (cx + 80.0, cy + 20.0),
        (cx + 65.0, cy + 65.0),
        (cx + 30.0, cy + 80.0),
        (cx - 30.0, cy + 80.0),
        (cx - 65.0, cy + 65.0),
        (cx - 80.0, cy + 20.0),
        (cx - 75.0, cy - 25.0),
        (cx - 45.0, cy - 75.0),
    ];

    smooth_shade(&mut img, &body_verts, (cx, cy), BODY, (-0.3, -1.0));
    let body_shape = polygon_pixels(&body_verts);

    // Belly
    let belly = ellipse_pixels(cx, cy + 20.0, 40.0, 30.0);
    for &(x, y) in &belly {
        if body_shape.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.25));
        }
    }

    // Feet — larger, more defined
    let foot_l_v = [(cx - 40.0, cy + 78.0), (cx - 12.0, cy + 78.0), (cx - 12.0, cy + 100.0), (cx - 45.0, cy + 100.0)];
    let foot_r_v = [(cx + 12.0, cy + 78.0), (cx + 40.0, cy + 78.0), (cx + 45.0, cy + 100.0), (cx + 12.0, cy + 100.0)];
    smooth_shade(&mut img, &foot_l_v, (cx - 26.0, cy + 88.0), BODY_SH, (-0.3, -1.0));
    smooth_shade(&mut img, &foot_r_v, (cx + 26.0, cy + 88.0), BODY_SH, (0.3, -1.0));

    // Arms — growing
    let arm_l_v = [(cx - 75.0, cy - 10.0), (cx - 60.0, cy - 15.0), (cx - 65.0, cy + 40.0), (cx - 85.0, cy + 35.0)];
    let arm_r_v = [(cx + 60.0, cy - 15.0), (cx + 75.0, cy - 10.0), (cx + 85.0, cy + 35.0), (cx + 65.0, cy + 40.0)];
    smooth_shade(&mut img, &arm_l_v, (cx - 72.0, cy + 12.0), BODY_SH, (-0.5, -1.0));
    smooth_shade(&mut img, &arm_r_v, (cx + 72.0, cy + 12.0), BODY_SH, (0.5, -1.0));

    let all: Shape = body_shape.iter()
        .chain(&polygon_pixels(&foot_l_v)).chain(&polygon_pixels(&foot_r_v))
        .chain(&polygon_pixels(&arm_l_v)).chain(&polygon_pixels(&arm_r_v))
        .copied().collect();

    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// ===================================================================
// ADULT STAGE — bear-like, semi-erect, imposing
// ===================================================================

fn gen_adult_body(dir: &Path) {
    let (w, h) = (320, 360);
    let mut img = new_canvas(w as u32, h as u32);
    let cx = 160.0_f32;

    // Head — broad bear head
    let head_verts: Vec<(f32, f32)> = vec![
        (cx,        16.0),
        (cx + 55.0, 22.0),
        (cx + 70.0, 60.0),
        (cx + 60.0, 95.0),
        (cx,        105.0),
        (cx - 60.0, 95.0),
        (cx - 70.0, 60.0),
        (cx - 55.0, 22.0),
    ];
    smooth_shade(&mut img, &head_verts, (cx, 60.0), BODY, (-0.3, -1.0));
    let head_shape = polygon_pixels(&head_verts);

    // Bear ears — small rounded
    let ear_l_v = [(cx - 55.0, 24.0), (cx - 40.0, 4.0), (cx - 25.0, 18.0)];
    let ear_r_v = [(cx + 25.0, 18.0), (cx + 40.0, 4.0), (cx + 55.0, 24.0)];
    smooth_shade(&mut img, &ear_l_v, (cx - 40.0, 16.0), BODY_SH, (-0.3, -1.0));
    smooth_shade(&mut img, &ear_r_v, (cx + 40.0, 16.0), BODY_SH, (0.3, -1.0));

    // Muzzle — lighter area
    let muzzle = ellipse_pixels(cx, 80.0, 25.0, 18.0);
    for &(x, y) in &muzzle {
        if head_shape.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.4));
        }
    }

    // Body — barrel torso (bear build)
    let torso_verts: Vec<(f32, f32)> = vec![
        (cx + 60.0, 100.0),
        (cx + 85.0, 140.0),
        (cx + 95.0, 190.0),
        (cx + 80.0, 240.0),
        (cx + 55.0, 260.0),
        (cx - 55.0, 260.0),
        (cx - 80.0, 240.0),
        (cx - 95.0, 190.0),
        (cx - 85.0, 140.0),
        (cx - 60.0, 100.0),
    ];
    smooth_shade(&mut img, &torso_verts, (cx, 180.0), BODY_SH, (-0.3, -1.0));
    let torso_shape = polygon_pixels(&torso_verts);

    // Belly highlight
    let belly = ellipse_pixels(cx, 190.0, 50.0, 40.0);
    for &(x, y) in &belly {
        if torso_shape.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.2));
        }
    }

    // Legs — thick bear paws
    let leg_l_v = [
        (cx - 55.0, 255.0), (cx - 22.0, 255.0),
        (cx - 18.0, 320.0), (cx - 30.0, 345.0), (cx - 60.0, 345.0), (cx - 60.0, 320.0),
    ];
    let leg_r_v = [
        (cx + 22.0, 255.0), (cx + 55.0, 255.0),
        (cx + 60.0, 320.0), (cx + 60.0, 345.0), (cx + 30.0, 345.0), (cx + 18.0, 320.0),
    ];
    smooth_shade(&mut img, &leg_l_v, (cx - 40.0, 300.0), BODY_SH2, (-0.3, -1.0));
    smooth_shade(&mut img, &leg_r_v, (cx + 40.0, 300.0), BODY_SH2, (0.3, -1.0));

    // Arms — powerful bear forearms
    let arm_l_v = [
        (cx - 85.0, 120.0), (cx - 60.0, 105.0),
        (cx - 70.0, 195.0), (cx - 85.0, 210.0), (cx - 110.0, 200.0), (cx - 110.0, 160.0),
    ];
    let arm_r_v = [
        (cx + 60.0, 105.0), (cx + 85.0, 120.0),
        (cx + 110.0, 160.0), (cx + 110.0, 200.0), (cx + 85.0, 210.0), (cx + 70.0, 195.0),
    ];
    smooth_shade(&mut img, &arm_l_v, (cx - 88.0, 160.0), BODY_SH, (-0.5, -1.0));
    smooth_shade(&mut img, &arm_r_v, (cx + 88.0, 160.0), BODY_SH, (0.5, -1.0));

    // Horns — bone with moss
    let horn_l_v = [(cx - 45.0, 18.0), (cx - 38.0, 8.0), (cx - 70.0, 0.0), (cx - 78.0, 12.0)];
    let horn_r_v = [(cx + 38.0, 8.0), (cx + 45.0, 18.0), (cx + 78.0, 12.0), (cx + 70.0, 0.0)];
    smooth_shade(&mut img, &horn_l_v, (cx - 58.0, 10.0), HORN_BASE, (0.0, -1.0));
    smooth_shade(&mut img, &horn_r_v, (cx + 58.0, 10.0), HORN_BASE, (0.0, -1.0));
    // Moss tips
    let moss_l = ellipse_pixels(cx - 74.0, 6.0, 4.0, 3.0);
    let moss_r = ellipse_pixels(cx + 74.0, 6.0, 4.0, 3.0);
    px_set(&mut img, &moss_l, HORN_MOSS);
    px_set(&mut img, &moss_r, HORN_MOSS);

    // Face markings
    let mark_l = polygon_pixels(&[(cx - 38.0, 45.0), (cx - 20.0, 35.0), (cx - 28.0, 60.0)]);
    let mark_r = polygon_pixels(&[(cx + 20.0, 35.0), (cx + 38.0, 45.0), (cx + 28.0, 60.0)]);
    for &(x, y) in mark_l.iter().chain(&mark_r) {
        if head_shape.contains(&(x, y)) {
            put(&mut img, x, y, FACE_MARK);
        }
    }

    // Lateral eyes (4 eyes — violet)
    for &(lx, ly) in &[(cx - 62.0, 55.0), (cx + 62.0, 55.0)] {
        let dot = ellipse_pixels(lx, ly, 3.0, 3.0);
        px_set(&mut img, &dot, LATERAL_EYE);
    }

    // Stub tail
    let tail_v = [(cx + 55.0, 240.0), (cx + 60.0, 230.0), (cx + 80.0, 245.0), (cx + 72.0, 260.0)];
    smooth_shade(&mut img, &tail_v, (cx + 68.0, 245.0), BODY_SH, (0.5, -1.0));

    // Outline everything
    let ear_l = polygon_pixels(&ear_l_v);
    let ear_r = polygon_pixels(&ear_r_v);
    let leg_l = polygon_pixels(&leg_l_v);
    let leg_r = polygon_pixels(&leg_r_v);
    let arm_l = polygon_pixels(&arm_l_v);
    let arm_r = polygon_pixels(&arm_r_v);
    let horn_l = polygon_pixels(&horn_l_v);
    let horn_r = polygon_pixels(&horn_r_v);
    let tail = polygon_pixels(&tail_v);
    let all: Shape = head_shape.iter()
        .chain(&torso_shape).chain(&ear_l).chain(&ear_r)
        .chain(&leg_l).chain(&leg_r).chain(&arm_l).chain(&arm_r)
        .chain(&horn_l).chain(&horn_r).chain(&tail)
        .copied().collect();

    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// ===================================================================
// EYES — solid black diamonds, all stages
// ===================================================================

fn gen_eyes(dir: &Path, size: u32, half_w: f32, half_h: f32) {
    let moods = ["idle", "happy", "hungry", "tired", "lonely", "playful", "sick"];

    for mood in &moods {
        let squint = *mood == "tired" || *mood == "sick";
        let tear = *mood == "lonely";
        let h = if squint { half_h * 0.5 } else { half_h };

        let mut img = new_canvas(size, size);
        let cx = size as f32 / 2.0;
        let cy = size as f32 / 2.0;
        let eye = diamond_pixels(cx, cy, half_w, h);
        px_set(&mut img, &eye, PUPIL);
        if tear {
            // Thick tear drop
            let tear_drop = ellipse_pixels(cx + half_w + 2.0, cy + half_h * 0.3, 2.0, 3.0);
            px_set(&mut img, &tear_drop, TEAR);
        }

        save_raw(&img, &format!("eye_left_{mood}.png"), dir);
        save_raw(&flip_h(&img), &format!("eye_right_{mood}.png"), dir);
    }

    // Sleeping — thick closed shape (not a thin line)
    let mut closed = new_canvas(size, size / 2);
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 4.0;
    let closed_shape = diamond_pixels(cx, cy, half_w, 2.0);
    px_set(&mut closed, &closed_shape, OUTLINE);
    save_raw(&closed, "eye_left_sleeping.png", dir);
    save_raw(&flip_h(&closed), "eye_right_sleeping.png", dir);
}

// ===================================================================
// EARS — angular, all stages
// ===================================================================

fn gen_ears(dir: &Path, size: u32) {
    let s = size as f32;
    let mut img = new_canvas(size, size);

    let ear_verts = vec![
        (s * 0.5, s * 0.08),  // tip
        (s * 0.85, s * 0.55),
        (s * 0.85, s * 0.92),
        (s * 0.15, s * 0.92),
        (s * 0.15, s * 0.55),
    ];
    smooth_shade(&mut img, &ear_verts, (s * 0.5, s * 0.6), BODY, (-0.3, -1.0));

    // Inner ear
    let inner = diamond_pixels(s * 0.5, s * 0.65, s * 0.2, s * 0.2);
    let ear_shape = polygon_pixels(&ear_verts);
    for &(x, y) in &inner {
        if ear_shape.contains(&(x, y)) {
            put(&mut img, x, y, EAR_INNER);
        }
    }

    flood_outline(&mut img, &ear_shape, OUTLINE);
    save_raw(&img, "ear_left_idle.png", dir);
    save_raw(&flip_h(&img), "ear_right_idle.png", dir);
}

// ===================================================================
// MOUTH — simple line art (reused across stages)
// ===================================================================

fn gen_mouths(dir: &Path, size: u32) {
    let s = size as f32;
    let cx = s / 2.0;
    let cy = s / 4.0;
    let w = s * 0.3;

    let moods = [
        ("idle", false, false),
        ("happy", true, false),
        ("hungry", false, true),
        ("tired", false, false),
        ("sleeping", false, false),
        ("lonely", false, false),
        ("playful", true, false),
        ("sick", false, false),
    ];

    for (mood, smile, open) in &moods {
        let mut img = new_canvas(size, size / 2);

        if *smile {
            // Smile — thick curved shape (filled arc, not a line)
            let smile_shape = polygon_pixels(&[
                (cx - w, cy), (cx - w * 0.5, cy + 1.0),
                (cx, cy + 3.0),
                (cx + w * 0.5, cy + 1.0), (cx + w, cy),
                (cx + w * 0.5, cy - 1.0), (cx, cy),
                (cx - w * 0.5, cy - 1.0),
            ]);
            px_set(&mut img, &smile_shape, OUTLINE);
        } else if *open {
            // Open mouth — filled oval
            let mouth = ellipse_pixels(cx, cy, w * 0.6, s * 0.15);
            px_set(&mut img, &mouth, PINK);
            flood_outline(&mut img, &mouth, OUTLINE);
        } else {
            // Neutral — small filled diamond (not a thin line)
            let neutral = diamond_pixels(cx, cy, w * 0.5, 2.0);
            px_set(&mut img, &neutral, OUTLINE);
        }

        save_raw(&img, &format!("mouth_{mood}.png"), dir);
    }
}

// ===================================================================
// SHARED EFFECTS
// ===================================================================

fn gen_effects(dir: &Path) {
    let sz = 120_u32;

    // ZZZ
    let mut img = new_canvas(sz, sz);
    let white = Rgba([255, 255, 255, 255]);
    for &(ox, oy, scale) in &[(15, 70, 1.0_f32), (40, 40, 1.5), (70, 10, 2.0)] {
        let s = (8.0 * scale) as i32;
        for x in 0..s { put(&mut img, ox + x, oy, white); }
        for i in 0..s { put(&mut img, ox + s - 1 - i, oy + i, white); }
        for x in 0..s { put(&mut img, ox + x, oy + s - 1, white); }
    }
    save_raw(&img, "zzz.png", dir);

    // Hearts
    let mut img = new_canvas(sz, sz);
    let heart_col = Rgba([255, 80, 100, 255]);
    for &(hx, hy, r) in &[(35.0, 75.0, 8.0), (75.0, 25.0, 14.0)] {
        let left = ellipse_pixels(hx - r * 0.5, hy - r * 0.3, r * 0.5, r * 0.45);
        let right = ellipse_pixels(hx + r * 0.5, hy - r * 0.3, r * 0.5, r * 0.45);
        let bottom = triangle_pixels((hx - r, hy), (hx + r, hy), (hx, hy + r));
        let heart: Shape = left.iter().chain(&right).chain(&bottom).copied().collect();
        px_set(&mut img, &heart, heart_col);
    }
    save_raw(&img, "hearts.png", dir);

    // Rain cloud
    let mut img = new_canvas(sz + 20, sz);
    let gray = Rgba([185, 185, 195, 255]);
    let tear = Rgba([140, 195, 250, 255]);
    for &(ccx, ccy, r) in &[(60.0, 25.0, 25.0), (40.0, 30.0, 18.0), (80.0, 30.0, 18.0)] {
        let cloud = ellipse_pixels(ccx, ccy, r, r * 0.6);
        px_set(&mut img, &cloud, gray);
    }
    for &(rx, ry) in &[(45, 50), (60, 55), (75, 50)] {
        for dy in 0..15 {
            put(&mut img, rx, ry + dy, tear);
        }
    }
    save_raw(&img, "rain_cloud.png", dir);

    // Stars
    let mut img = new_canvas(sz, sz);
    let yellow = Rgba([255, 255, 110, 255]);
    for &(sx, sy) in &[(30, 25), (85, 35), (25, 80), (90, 85)] {
        for d in 0..8 {
            put(&mut img, sx, sy - d, yellow);
            put(&mut img, sx, sy + d, yellow);
            put(&mut img, sx - d, sy, yellow);
            put(&mut img, sx + d, sy, yellow);
        }
        put(&mut img, sx, sy, white);
    }
    save_raw(&img, "stars_dizzy.png", dir);

    // Sparkle
    let mut img = new_canvas(sz, sz);
    let sparkle = Rgba([255, 255, 200, 255]);
    for &(sx, sy) in &[(25, 20), (90, 30), (20, 90), (95, 85), (55, 10), (55, 105)] {
        for d in 0..6 {
            put(&mut img, sx, sy - d, sparkle);
            put(&mut img, sx, sy + d, sparkle);
            put(&mut img, sx - d, sy, sparkle);
            put(&mut img, sx + d, sy, sparkle);
        }
        put(&mut img, sx, sy, white);
    }
    save_raw(&img, "sparkle.png", dir);
}

// ===================================================================
// MAIN
// ===================================================================

fn main() {
    let base = base_dir();
    let egg_dir = base.join("egg");
    let cub_dir = base.join("cub");
    let young_dir = base.join("young");
    let adult_dir = base.join("adult");
    let elder_dir = base.join("elder");
    let edir = effects_dir();

    for d in [&egg_dir, &cub_dir, &young_dir, &adult_dir, &elder_dir, &edir] {
        std::fs::create_dir_all(d).ok();
    }

    println!("Generating Moluun sprites (all stages)\n");

    // --- Egg ---
    println!("=== Egg ===");
    gen_egg(&egg_dir);

    // --- Cub (big cute eyes, round everything) ---
    println!("\n=== Cub ===");
    gen_cub_body(&cub_dir);
    gen_eyes(&cub_dir, 40, 15.0, 15.0);  // huge eyes for max cuteness
    gen_ears(&cub_dir, 32);
    gen_mouths(&cub_dir, 30);

    // --- Young ---
    println!("\n=== Young ===");
    gen_young_body(&young_dir);
    gen_eyes(&young_dir, 36, 13.0, 14.0);
    gen_ears(&young_dir, 36);
    gen_mouths(&young_dir, 30);

    // --- Adult ---
    println!("\n=== Adult ===");
    gen_adult_body(&adult_dir);
    gen_eyes(&adult_dir, 32, 11.0, 12.0);
    gen_ears(&adult_dir, 34);
    gen_mouths(&adult_dir, 28);

    // --- Elder ---
    println!("\n=== Elder ===");
    gen_adult_body(&elder_dir);
    gen_eyes(&elder_dir, 32, 11.0, 12.0);
    gen_ears(&elder_dir, 34);
    gen_mouths(&elder_dir, 28);

    // --- Shared effects ---
    println!("\n=== Shared Effects ===");
    gen_effects(&edir);

    println!("\nDone! All stages saved.");
}
