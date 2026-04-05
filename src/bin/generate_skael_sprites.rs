//! Kokoro — Skael (Cave Reptile) Sprite Generator
//!
//! Stages: egg → cub → young → adult → elder
//! Usage: cargo run --bin generate_skael_sprites

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
col!(BODY,      120, 180, 130);
col!(BODY_HI,   150, 210, 155);
col!(BODY_SH,    90, 150, 105);
col!(BODY_SH2,   70, 125,  85);
col!(SCALE_COL, 100, 165, 115);
col!(CREST_COL, 160, 100,  80);
col!(CREST_TIP, 190, 120,  90);
col!(OUTLINE,    35,  50,  40);
col!(PUPIL,      30,  30,  30);
col!(IRIS,      200, 160,  50);
col!(SNOUT_COL,  80, 100,  80);
col!(TEAR,      140, 195, 250);
col!(EGG_BASE,  160, 190, 150);
col!(EGG_SPOT,  130, 170, 130);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("skael")
}

fn gen_egg(dir: &Path) {
    let (w, h) = (180, 230);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (90.0, 125.0);
    // Crystalline egg — more angular
    let verts = vec![
        (cx, cy - 90.0), (cx + 35.0, cy - 70.0), (cx + 55.0, cy - 30.0),
        (cx + 50.0, cy + 20.0), (cx + 35.0, cy + 60.0), (cx, cy + 80.0),
        (cx - 35.0, cy + 60.0), (cx - 50.0, cy + 20.0), (cx - 55.0, cy - 30.0),
        (cx - 35.0, cy - 70.0),
    ];
    smooth_shade(&mut img, &verts, (cx, cy), EGG_BASE, (-0.3, -1.0));
    let shape = polygon_pixels(&verts);
    // Crystal facet lines
    for &(sx, sy, r) in &[(cx + 10.0, cy - 20.0, 10.0), (cx - 15.0, cy + 20.0, 8.0)] {
        let spot = diamond_pixels(sx, sy, r, r * 0.7);
        for &(x, y) in &spot { if shape.contains(&(x, y)) { put(&mut img, x, y, EGG_SPOT); } }
    }
    flood_outline(&mut img, &shape, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_cub_body(dir: &Path) {
    let (w, h) = (240, 260);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (120.0, 115.0);
    let verts = vec![
        (cx, cy - 75.0), (cx + 40.0, cy - 65.0), (cx + 65.0, cy - 20.0),
        (cx + 60.0, cy + 30.0), (cx + 35.0, cy + 70.0), (cx, cy + 75.0),
        (cx - 35.0, cy + 70.0), (cx - 60.0, cy + 30.0), (cx - 65.0, cy - 20.0),
        (cx - 40.0, cy - 65.0),
    ];
    smooth_shade(&mut img, &verts, (cx, cy), BODY, (-0.2, -1.0));
    // Stub tail
    let tail = [(cx, cy + 73.0), (cx + 8.0, cy + 70.0), (cx + 5.0, cy + 95.0), (cx - 5.0, cy + 95.0), (cx - 8.0, cy + 70.0)];
    smooth_shade(&mut img, &tail, (cx, cy + 82.0), BODY_SH, (0.0, -1.0));
    // Stub legs
    let fl = [(cx - 30.0, cy + 68.0), (cx - 12.0, cy + 68.0), (cx - 14.0, cy + 88.0), (cx - 32.0, cy + 88.0)];
    let fr = [(cx + 12.0, cy + 68.0), (cx + 30.0, cy + 68.0), (cx + 32.0, cy + 88.0), (cx + 14.0, cy + 88.0)];
    smooth_shade(&mut img, &fl, (cx - 22.0, cy + 78.0), BODY_SH, (-0.3, -1.0));
    smooth_shade(&mut img, &fr, (cx + 22.0, cy + 78.0), BODY_SH, (0.3, -1.0));

    let body = polygon_pixels(&verts);
    let all: Shape = body.iter().chain(&polygon_pixels(&tail)).chain(&polygon_pixels(&fl)).chain(&polygon_pixels(&fr)).copied().collect();
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_adult_body(dir: &Path) {
    let (w, h) = (300, 380);
    let mut img = new_canvas(w as u32, h as u32);
    let cx = 150.0;
    // Head — angular, reptilian
    let head = vec![
        (cx, 15.0), (cx + 45.0, 22.0), (cx + 60.0, 55.0),
        (cx + 50.0, 90.0), (cx, 100.0),
        (cx - 50.0, 90.0), (cx - 60.0, 55.0), (cx - 45.0, 22.0),
    ];
    smooth_shade(&mut img, &head, (cx, 55.0), BODY, (-0.2, -1.0));
    // Crests
    let cl = [(cx - 40.0, 22.0), (cx - 30.0, 5.0), (cx - 20.0, 18.0)];
    let cr = [(cx + 20.0, 18.0), (cx + 30.0, 5.0), (cx + 40.0, 22.0)];
    smooth_shade(&mut img, &cl, (cx - 30.0, 15.0), CREST_COL, (0.0, -1.0));
    smooth_shade(&mut img, &cr, (cx + 30.0, 15.0), CREST_COL, (0.0, -1.0));
    // Body — elongated, armored
    let torso = vec![
        (cx + 50.0, 95.0), (cx + 70.0, 130.0), (cx + 75.0, 180.0),
        (cx + 65.0, 230.0), (cx + 45.0, 260.0),
        (cx - 45.0, 260.0), (cx - 65.0, 230.0), (cx - 75.0, 180.0),
        (cx - 70.0, 130.0), (cx - 50.0, 95.0),
    ];
    smooth_shade(&mut img, &torso, (cx, 175.0), BODY_SH, (-0.2, -1.0));
    // Scale bands
    let torso_s = polygon_pixels(&torso);
    for y_off in (-50..=50).step_by(12) {
        let row = 175 + y_off;
        for &(x, y) in &torso_s { if y == row { put(&mut img, x, y, SCALE_COL); } }
    }
    // Legs
    let ll = [(cx - 45.0, 255.0), (cx - 20.0, 255.0), (cx - 18.0, 320.0), (cx - 25.0, 340.0), (cx - 50.0, 340.0), (cx - 48.0, 320.0)];
    let lr = [(cx + 20.0, 255.0), (cx + 45.0, 255.0), (cx + 48.0, 320.0), (cx + 50.0, 340.0), (cx + 25.0, 340.0), (cx + 18.0, 320.0)];
    smooth_shade(&mut img, &ll, (cx - 33.0, 295.0), BODY_SH2, (-0.3, -1.0));
    smooth_shade(&mut img, &lr, (cx + 33.0, 295.0), BODY_SH2, (0.3, -1.0));
    // Tail — thick
    let tail = [(cx - 15.0, 255.0), (cx + 15.0, 255.0), (cx + 10.0, 330.0), (cx + 3.0, 370.0), (cx - 3.0, 370.0), (cx - 10.0, 330.0)];
    smooth_shade(&mut img, &tail, (cx, 310.0), BODY_SH, (0.0, -1.0));

    let head_s = polygon_pixels(&head);
    let all: Shape = head_s.iter().chain(&torso_s).chain(&polygon_pixels(&cl)).chain(&polygon_pixels(&cr))
        .chain(&polygon_pixels(&ll)).chain(&polygon_pixels(&lr)).chain(&polygon_pixels(&tail))
        .copied().collect();
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_eyes(dir: &Path, size: u32, hw: f32, hh: f32) {
    for mood in ["idle", "happy", "hungry", "tired", "lonely", "playful", "sick"] {
        let squint = mood == "tired" || mood == "sick";
        let tear = mood == "lonely";
        let h = if squint { hh * 0.5 } else { hh };
        let mut img = new_canvas(size, size);
        let c = size as f32 / 2.0;
        // Golden iris diamond
        let iris = diamond_pixels(c, c, hw, h);
        px_set(&mut img, &iris, IRIS);
        // Thick vertical slit pupil (2px wide)
        for dy in -(h as i32)..=(h as i32) {
            for dx in 0..=1 {
                let pos = (c as i32 + dx, c as i32 + dy);
                let neg = (c as i32 - dx, c as i32 + dy);
                if iris.contains(&pos) { put(&mut img, pos.0, pos.1, PUPIL); }
                if iris.contains(&neg) { put(&mut img, neg.0, neg.1, PUPIL); }
            }
        }
        if tear { let t = ellipse_pixels(c + hw + 2.0, c + hh * 0.3, 2.0, 3.0); px_set(&mut img, &t, TEAR); }
        save_raw(&img, &format!("eye_left_{mood}.png"), dir);
        save_raw(&flip_h(&img), &format!("eye_right_{mood}.png"), dir);
    }
    let mut closed = new_canvas(size, size / 2);
    let c2 = size as f32 / 2.0;
    px_set(&mut closed, &diamond_pixels(c2, size as f32 / 4.0, hw, 2.0), OUTLINE);
    save_raw(&closed, "eye_left_sleeping.png", dir);
    save_raw(&flip_h(&closed), "eye_right_sleeping.png", dir);
}

fn gen_snout(dir: &Path, size: u32) {
    let s = size as f32;
    for mood in ["idle", "hungry", "tired", "lonely", "playful", "sick", "sleeping"] {
        let mut img = new_canvas(size, size);
        let (cx, cy) = (s / 2.0, s / 2.0);
        let open = mood == "hungry" || mood == "playful";
        if open {
            let upper = polygon_pixels(&[(cx - 5.0, cy - 3.0), (cx + 5.0, cy - 3.0), (cx + 4.0, cy), (cx - 4.0, cy)]);
            let lower = polygon_pixels(&[(cx - 4.0, cy + 2.0), (cx + 4.0, cy + 2.0), (cx + 3.0, cy + 5.0), (cx - 3.0, cy + 5.0)]);
            px_set(&mut img, &upper, SNOUT_COL); px_set(&mut img, &lower, SNOUT_COL);
            let all: Shape = upper.union(&lower).copied().collect();
            flood_outline(&mut img, &all, OUTLINE);
        } else {
            let snout = polygon_pixels(&[(cx - 5.0, cy - 2.0), (cx + 5.0, cy - 2.0), (cx + 4.0, cy + 3.0), (cx - 4.0, cy + 3.0)]);
            px_set(&mut img, &snout, SNOUT_COL);
            flood_outline(&mut img, &snout, OUTLINE);
        }
        save_raw(&img, &format!("snout_{mood}.png"), dir);
    }
}

fn gen_crests(dir: &Path, size: u32) {
    let s = size as f32;
    let mut img = new_canvas(size, (s * 1.5) as u32);
    let verts = [(s * 0.5, s * 0.1), (s * 0.8, s * 0.5), (s * 0.7, s * 1.3), (s * 0.3, s * 1.3), (s * 0.2, s * 0.5)];
    smooth_shade(&mut img, &verts, (s * 0.5, s * 0.7), CREST_COL, (0.0, -1.0));
    flood_outline(&mut img, &polygon_pixels(&verts), OUTLINE);
    save_raw(&img, "crest_left_idle.png", dir);
    save_raw(&flip_h(&img), "crest_right_idle.png", dir);
}

fn gen_tail(dir: &Path, size: u32) {
    let s = size as f32;
    let mut img = new_canvas(size, (s * 2.0) as u32);
    let verts = [(s * 0.3, 2.0), (s * 0.7, 2.0), (s * 0.6, s * 1.2), (s * 0.55, s * 1.8), (s * 0.45, s * 1.8), (s * 0.4, s * 1.2)];
    smooth_shade(&mut img, &verts, (s * 0.5, s), BODY_SH, (0.0, -1.0));
    flood_outline(&mut img, &polygon_pixels(&verts), OUTLINE);
    save_raw(&img, "tail_idle.png", dir);
}

fn main() {
    let base = base_dir();
    let egg = base.join("egg"); let cub = base.join("cub"); let young = base.join("young");
    let adult = base.join("adult"); let elder = base.join("elder");

    println!("Generating Skael sprites (all stages)\n");

    println!("=== Egg ==="); gen_egg(&egg);
    println!("\n=== Cub ==="); gen_cub_body(&cub);
    gen_eyes(&cub, 36, 13.0, 13.0); gen_snout(&cub, 24); gen_crests(&cub, 20); gen_tail(&cub, 18);
    println!("\n=== Young ==="); gen_cub_body(&young);
    gen_eyes(&young, 32, 11.0, 12.0); gen_snout(&young, 22); gen_crests(&young, 24); gen_tail(&young, 20);
    println!("\n=== Adult ==="); gen_adult_body(&adult);
    gen_eyes(&adult, 28, 10.0, 11.0); gen_snout(&adult, 24); gen_crests(&adult, 26); gen_tail(&adult, 22);
    println!("\n=== Elder ==="); gen_adult_body(&elder);
    gen_eyes(&elder, 28, 10.0, 11.0); gen_snout(&elder, 24); gen_crests(&elder, 26); gen_tail(&elder, 22);

    // Clean old root sprites
    for entry in std::fs::read_dir(&base).unwrap().flatten() {
        if entry.path().extension().map(|e| e == "png").unwrap_or(false) { std::fs::remove_file(entry.path()).ok(); }
    }
    println!("\nDone!");
}
