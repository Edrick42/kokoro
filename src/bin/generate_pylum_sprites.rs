//! Kokoro — Pylum (Highland Bird) Sprite Generator
//!
//! Stages: egg → cub → young → adult → elder
//! No pixelization — full resolution with collage shading.
//!
//! Usage: cargo run --bin generate_pylum_sprites

#[allow(dead_code)]
mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

macro_rules! col {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        #[allow(dead_code)] const $name: Rgba<u8> = Rgba([$r, $g, $b, 255]);
    };
}
col!(BODY,      255, 220, 140);
col!(BODY_HI,   255, 235, 170);
col!(BODY_SH,   230, 190, 110);
col!(WING_BASE, 240, 200, 120);
col!(WING_TIP,  200, 160,  90);
col!(OUTLINE,    60,  45,  30);
col!(PUPIL,      30,  30,  45);
col!(BEAK_COL,  255, 160,  60);
col!(BEAK_DARK, 220, 130,  40);
col!(PINK,      240, 160, 165);
col!(TEAR,      140, 195, 250);
col!(EGG_BASE,  240, 225, 190);
col!(EGG_SPOT,  220, 200, 160);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("pylum")
}
fn effects_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("shared").join("effects")
}

// === EGG ===
fn gen_egg(dir: &Path) {
    let (w, h) = (180, 220);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (90.0, 120.0);
    let verts = vec![
        (cx, cy - 85.0), (cx + 25.0, cy - 80.0), (cx + 50.0, cy - 55.0),
        (cx + 55.0, cy - 15.0), (cx + 50.0, cy + 25.0), (cx + 35.0, cy + 60.0),
        (cx + 15.0, cy + 75.0), (cx, cy + 80.0), (cx - 15.0, cy + 75.0),
        (cx - 35.0, cy + 60.0), (cx - 50.0, cy + 25.0), (cx - 55.0, cy - 15.0),
        (cx - 50.0, cy - 55.0), (cx - 25.0, cy - 80.0),
    ];
    smooth_shade(&mut img, &verts, (cx, cy), EGG_BASE, (-0.3, -1.0));
    let shape = polygon_pixels(&verts);
    // Speckles
    for &(sx, sy, r) in &[(cx - 15.0, cy - 20.0, 8.0), (cx + 20.0, cy + 15.0, 6.0), (cx - 5.0, cy + 35.0, 5.0)] {
        let spot = ellipse_pixels(sx, sy, r, r * 0.7);
        for &(x, y) in &spot { if shape.contains(&(x, y)) { put(&mut img, x, y, EGG_SPOT); } }
    }
    flood_outline(&mut img, &shape, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// === BODY GENERATORS ===
fn gen_cub_body(dir: &Path) {
    let (w, h) = (240, 240);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (120.0, 110.0);
    let verts = vec![
        (cx, cy - 75.0), (cx + 45.0, cy - 65.0), (cx + 75.0, cy - 15.0),
        (cx + 70.0, cy + 35.0), (cx + 40.0, cy + 65.0), (cx, cy + 70.0),
        (cx - 40.0, cy + 65.0), (cx - 70.0, cy + 35.0), (cx - 75.0, cy - 15.0),
        (cx - 45.0, cy - 65.0),
    ];
    smooth_shade(&mut img, &verts, (cx, cy), BODY, (-0.3, -1.0));
    // Wing stubs
    let wl = [(cx - 70.0, cy - 5.0), (cx - 85.0, cy + 10.0), (cx - 68.0, cy + 25.0)];
    let wr = [(cx + 70.0, cy - 5.0), (cx + 85.0, cy + 10.0), (cx + 68.0, cy + 25.0)];
    smooth_shade(&mut img, &wl, (cx - 75.0, cy + 10.0), WING_BASE, (-0.5, -1.0));
    smooth_shade(&mut img, &wr, (cx + 75.0, cy + 10.0), WING_BASE, (0.5, -1.0));
    // Feet
    let fl = [(cx - 25.0, cy + 68.0), (cx - 8.0, cy + 68.0), (cx - 16.0, cy + 85.0)];
    let fr = [(cx + 8.0, cy + 68.0), (cx + 25.0, cy + 68.0), (cx + 16.0, cy + 85.0)];
    smooth_shade(&mut img, &fl, (cx - 16.0, cy + 76.0), BEAK_COL, (0.0, -1.0));
    smooth_shade(&mut img, &fr, (cx + 16.0, cy + 76.0), BEAK_COL, (0.0, -1.0));

    let body = polygon_pixels(&verts);
    let all: Shape = body.iter().chain(&polygon_pixels(&wl)).chain(&polygon_pixels(&wr))
        .chain(&polygon_pixels(&fl)).chain(&polygon_pixels(&fr)).copied().collect();
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_adult_body(dir: &Path) {
    let (w, h) = (320, 340);
    let mut img = new_canvas(w as u32, h as u32);
    let cx = 160.0;
    // Head
    let head = vec![
        (cx, 15.0), (cx + 40.0, 20.0), (cx + 55.0, 50.0),
        (cx + 45.0, 80.0), (cx, 90.0),
        (cx - 45.0, 80.0), (cx - 55.0, 50.0), (cx - 40.0, 20.0),
    ];
    smooth_shade(&mut img, &head, (cx, 50.0), BODY, (-0.3, -1.0));
    // Crest tuft on top
    let tuft = [(cx - 5.0, 18.0), (cx + 5.0, 18.0), (cx + 3.0, 2.0), (cx - 3.0, 2.0)];
    smooth_shade(&mut img, &tuft, (cx, 10.0), BODY_HI, (0.0, -1.0));
    // Body — egg-shaped
    let torso = vec![
        (cx + 45.0, 85.0), (cx + 65.0, 120.0), (cx + 70.0, 170.0),
        (cx + 55.0, 220.0), (cx + 25.0, 240.0),
        (cx - 25.0, 240.0), (cx - 55.0, 220.0), (cx - 70.0, 170.0),
        (cx - 65.0, 120.0), (cx - 45.0, 85.0),
    ];
    smooth_shade(&mut img, &torso, (cx, 160.0), BODY_SH, (-0.3, -1.0));
    // Belly
    let torso_shape = polygon_pixels(&torso);
    let belly = ellipse_pixels(cx, 170.0, 40.0, 35.0);
    for &(x, y) in &belly {
        if torso_shape.contains(&(x, y)) {
            let e = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(e, BODY_HI, 0.3));
        }
    }
    // Wings — large, angular
    let wl = [(cx - 65.0, 100.0), (cx - 55.0, 90.0), (cx - 90.0, 140.0), (cx - 120.0, 160.0), (cx - 110.0, 130.0)];
    let wr = [(cx + 55.0, 90.0), (cx + 65.0, 100.0), (cx + 110.0, 130.0), (cx + 120.0, 160.0), (cx + 90.0, 140.0)];
    smooth_shade(&mut img, &wl, (cx - 85.0, 125.0), WING_BASE, (-0.5, -1.0));
    smooth_shade(&mut img, &wr, (cx + 85.0, 125.0), WING_BASE, (0.5, -1.0));
    // Legs — raptor
    let ll = [(cx - 25.0, 235.0), (cx - 10.0, 235.0), (cx - 8.0, 290.0), (cx - 15.0, 310.0), (cx - 30.0, 310.0), (cx - 28.0, 290.0)];
    let lr = [(cx + 10.0, 235.0), (cx + 25.0, 235.0), (cx + 28.0, 290.0), (cx + 30.0, 310.0), (cx + 15.0, 310.0), (cx + 8.0, 290.0)];
    smooth_shade(&mut img, &ll, (cx - 18.0, 270.0), BEAK_COL, (-0.3, -1.0));
    smooth_shade(&mut img, &lr, (cx + 18.0, 270.0), BEAK_COL, (0.3, -1.0));
    // Tail
    let tail = [(cx - 12.0, 235.0), (cx + 12.0, 235.0), (cx + 8.0, 330.0), (cx, 338.0), (cx - 8.0, 330.0)];
    smooth_shade(&mut img, &tail, (cx, 285.0), WING_TIP, (0.0, -1.0));

    let head_s = polygon_pixels(&head);
    let all: Shape = head_s.iter().chain(&torso_shape).chain(&polygon_pixels(&tuft))
        .chain(&polygon_pixels(&wl)).chain(&polygon_pixels(&wr))
        .chain(&polygon_pixels(&ll)).chain(&polygon_pixels(&lr))
        .chain(&polygon_pixels(&tail))
        .copied().collect();
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

// === EYES (black diamonds) ===
fn gen_eyes(dir: &Path, size: u32, hw: f32, hh: f32) {
    for mood in ["idle", "happy", "hungry", "tired", "lonely", "playful", "sick"] {
        let squint = mood == "tired" || mood == "sick";
        let tear = mood == "lonely";
        let h = if squint { hh * 0.5 } else { hh };
        let mut img = new_canvas(size, size);
        let c = size as f32 / 2.0;
        px_set(&mut img, &diamond_pixels(c, c, hw, h), PUPIL);
        if tear { let t = ellipse_pixels(c + hw + 2.0, c + hh * 0.3, 2.0, 3.0); px_set(&mut img, &t, TEAR); }
        save_raw(&img, &format!("eye_left_{mood}.png"), dir);
        save_raw(&flip_h(&img), &format!("eye_right_{mood}.png"), dir);
    }
    let mut closed = new_canvas(size, size / 2);
    let c = size as f32 / 2.0;
    px_set(&mut closed, &diamond_pixels(c, size as f32 / 4.0, hw, 2.0), OUTLINE);
    save_raw(&closed, "eye_left_sleeping.png", dir);
    save_raw(&flip_h(&closed), "eye_right_sleeping.png", dir);
}

// === BEAK ===
fn gen_beak(dir: &Path, size: u32) {
    let s = size as f32;
    for mood in ["idle", "hungry", "tired", "lonely", "playful", "sick", "sleeping"] {
        let mut img = new_canvas(size, size);
        let cx = s / 2.0;
        let cy = s / 2.0;
        let open = mood == "hungry" || mood == "playful";
        if open {
            let upper = polygon_pixels(&[(cx - 4.0, cy - 4.0), (cx + 4.0, cy - 4.0), (cx, cy)]);
            let lower = polygon_pixels(&[(cx - 3.0, cy + 2.0), (cx + 3.0, cy + 2.0), (cx, cy + 6.0)]);
            px_set(&mut img, &upper, BEAK_COL);
            px_set(&mut img, &lower, BEAK_DARK);
            let all: Shape = upper.union(&lower).copied().collect();
            flood_outline(&mut img, &all, OUTLINE);
        } else {
            let beak = polygon_pixels(&[(cx - 4.0, cy - 3.0), (cx + 4.0, cy - 3.0), (cx, cy + 5.0)]);
            px_set(&mut img, &beak, BEAK_COL);
            flood_outline(&mut img, &beak, OUTLINE);
        }
        save_raw(&img, &format!("beak_{mood}.png"), dir);
    }
}

// === WINGS (standalone for rig) ===
fn gen_wings(dir: &Path, size: u32) {
    let s = size as f32;
    let verts_l = vec![(s * 0.8, s * 0.2), (s * 0.9, s * 0.5), (s * 0.6, s * 0.8), (s * 0.1, s * 0.7), (s * 0.05, s * 0.4), (s * 0.4, s * 0.15)];
    let mut img = new_canvas(size, size);
    smooth_shade(&mut img, &verts_l, (s * 0.5, s * 0.5), WING_BASE, (-0.5, -0.8));
    flood_outline(&mut img, &polygon_pixels(&verts_l), OUTLINE);
    save_raw(&img, "wing_left_idle.png", dir);
    save_raw(&flip_h(&img), "wing_right_idle.png", dir);
}

// === TAIL ===
fn gen_tail(dir: &Path, size: u32) {
    let s = size as f32;
    let mut img = new_canvas(size, (s * 1.5) as u32);
    for (off, t) in [(-3.0_f32, 0.5), (0.0, 0.3), (3.0, 0.5)] {
        let x = s / 2.0 + off;
        let v = [(x - 2.0, 3.0), (x + 2.0, 3.0), (x + 1.0, s * 1.3), (x - 1.0, s * 1.3)];
        let pts = polygon_pixels(&v);
        px_set(&mut img, &pts, lerp(WING_BASE, WING_TIP, t));
        flood_outline(&mut img, &pts, OUTLINE);
    }
    save_raw(&img, "tail_idle.png", dir);
}

fn main() {
    let base = base_dir();
    let egg = base.join("egg");
    let cub = base.join("cub");
    let young = base.join("young");
    let adult = base.join("adult");
    let elder = base.join("elder");

    println!("Generating Pylum sprites (all stages)\n");

    println!("=== Egg ==="); gen_egg(&egg);

    println!("\n=== Cub ==="); gen_cub_body(&cub);
    gen_eyes(&cub, 38, 14.0, 14.0); gen_beak(&cub, 24); gen_wings(&cub, 40); gen_tail(&cub, 18);

    println!("\n=== Young ==="); gen_cub_body(&young);
    gen_eyes(&young, 34, 12.0, 13.0); gen_beak(&young, 22); gen_wings(&young, 48); gen_tail(&young, 20);

    println!("\n=== Adult ==="); gen_adult_body(&adult);
    gen_eyes(&adult, 30, 10.0, 11.0); gen_beak(&adult, 24); gen_wings(&adult, 56); gen_tail(&adult, 22);

    println!("\n=== Elder ==="); gen_adult_body(&elder);
    gen_eyes(&elder, 30, 10.0, 11.0); gen_beak(&elder, 24); gen_wings(&elder, 56); gen_tail(&elder, 22);

    // Delete old root sprites
    for entry in std::fs::read_dir(&base).unwrap().flatten() {
        if entry.path().extension().map(|e| e == "png").unwrap_or(false) {
            std::fs::remove_file(entry.path()).ok();
        }
    }

    println!("\nDone!");
}
