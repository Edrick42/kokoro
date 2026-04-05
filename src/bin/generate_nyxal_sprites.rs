//! Kokoro — Nyxal (Abyssal Squid) Sprite Generator
//!
//! Stages: egg → cub → young → adult → elder
//! Usage: cargo run --bin generate_nyxal_sprites

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
col!(BODY,          95,  60, 130);
col!(BODY_HI,     120,  80, 155);
col!(BODY_SH,      70,  42, 105);
col!(MANTLE,        85,  50, 120);
col!(OUTLINE,       25,  18,  40);
col!(PUPIL,         15,  15,  30);
col!(EYE_GLOW,      40, 180, 200);
col!(TENTACLE,     100,  65, 140);
col!(TENTACLE_TIP,  50, 170, 180);
col!(TEAR,         100, 160, 220);
col!(EGG_BASE,     130, 100, 160);
col!(EGG_SPOT,     100,  80, 140);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("nyxal")
}

fn gen_egg(dir: &Path) {
    let (w, h) = (200, 200);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (100.0, 100.0);
    // Roe cluster — multiple small spheres
    for &(sx, sy, r) in &[
        (cx, cy, 35.0), (cx - 25.0, cy - 20.0, 22.0), (cx + 25.0, cy - 15.0, 20.0),
        (cx - 15.0, cy + 25.0, 18.0), (cx + 20.0, cy + 22.0, 16.0),
    ] {
        let verts: Vec<(f32, f32)> = (0..8).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            (sx + a.cos() * r, sy + a.sin() * r)
        }).collect();
        smooth_shade(&mut img, &verts, (sx, sy), EGG_BASE, (-0.3, -1.0));
        flood_outline(&mut img, &polygon_pixels(&verts), OUTLINE);
    }
    save_raw(&img, "body_idle.png", dir);
}

fn gen_cub_body(dir: &Path) {
    let (w, h) = (240, 260);
    let mut img = new_canvas(w as u32, h as u32);
    let (cx, cy) = (120.0, 105.0);
    // Soft round body
    let verts = vec![
        (cx, cy - 70.0), (cx + 45.0, cy - 60.0), (cx + 70.0, cy - 10.0),
        (cx + 65.0, cy + 35.0), (cx + 35.0, cy + 60.0), (cx, cy + 65.0),
        (cx - 35.0, cy + 60.0), (cx - 65.0, cy + 35.0), (cx - 70.0, cy - 10.0),
        (cx - 45.0, cy - 60.0),
    ];
    smooth_shade(&mut img, &verts, (cx, cy), BODY, (-0.4, -0.9));
    // 2 stub tentacles
    let tl = [(cx - 20.0, cy + 63.0), (cx - 8.0, cy + 63.0), (cx - 12.0, cy + 100.0), (cx - 18.0, cy + 100.0)];
    let tr = [(cx + 8.0, cy + 63.0), (cx + 20.0, cy + 63.0), (cx + 18.0, cy + 100.0), (cx + 12.0, cy + 100.0)];
    smooth_shade(&mut img, &tl, (cx - 14.0, cy + 80.0), TENTACLE, (0.0, -1.0));
    smooth_shade(&mut img, &tr, (cx + 14.0, cy + 80.0), TENTACLE, (0.0, -1.0));
    // Glow tips
    let tip_l = diamond_pixels(cx - 15.0, cy + 98.0, 4.0, 3.0);
    let tip_r = diamond_pixels(cx + 15.0, cy + 98.0, 4.0, 3.0);
    px_set(&mut img, &tip_l, TENTACLE_TIP); px_set(&mut img, &tip_r, TENTACLE_TIP);

    let body = polygon_pixels(&verts);
    let all: Shape = body.iter().chain(&polygon_pixels(&tl)).chain(&polygon_pixels(&tr)).copied().collect();
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_adult_body(dir: &Path) {
    let (w, h) = (320, 380);
    let mut img = new_canvas(w as u32, h as u32);
    let cx = 160.0;
    // Mantle dome
    let mantle = vec![
        (cx, 10.0), (cx + 40.0, 15.0), (cx + 55.0, 40.0),
        (cx + 50.0, 65.0), (cx, 75.0),
        (cx - 50.0, 65.0), (cx - 55.0, 40.0), (cx - 40.0, 15.0),
    ];
    smooth_shade(&mut img, &mantle, (cx, 42.0), MANTLE, (-0.3, -1.0));
    // Body
    let body = vec![
        (cx + 50.0, 70.0), (cx + 65.0, 100.0), (cx + 60.0, 140.0),
        (cx + 40.0, 165.0), (cx, 175.0),
        (cx - 40.0, 165.0), (cx - 60.0, 140.0), (cx - 65.0, 100.0),
        (cx - 50.0, 70.0),
    ];
    smooth_shade(&mut img, &body, (cx, 120.0), BODY, (-0.4, -0.9));
    // 4 tentacles
    let tentacles: Vec<([(f32, f32); 4], f32)> = vec![
        ([(cx - 30.0, 170.0), (cx - 18.0, 170.0), (cx - 20.0, 280.0), (cx - 28.0, 280.0)], cx - 24.0),
        ([(cx + 18.0, 170.0), (cx + 30.0, 170.0), (cx + 28.0, 280.0), (cx + 20.0, 280.0)], cx + 24.0),
        ([(cx - 50.0, 160.0), (cx - 38.0, 160.0), (cx - 42.0, 260.0), (cx - 48.0, 260.0)], cx - 45.0),
        ([(cx + 38.0, 160.0), (cx + 50.0, 160.0), (cx + 48.0, 260.0), (cx + 42.0, 260.0)], cx + 45.0),
    ];
    for (tv, tcx) in &tentacles {
        smooth_shade(&mut img, tv, (*tcx, 220.0), TENTACLE, (0.0, -1.0));
        // Glow tip
        let tip_y = tv[2].1 - 3.0;
        let tip = diamond_pixels(*tcx, tip_y, 4.0, 3.0);
        px_set(&mut img, &tip, TENTACLE_TIP);
    }

    let mantle_s = polygon_pixels(&mantle);
    let body_s = polygon_pixels(&body);
    let mut all: Shape = mantle_s.iter().chain(&body_s).copied().collect();
    for (tv, _) in &tentacles { all.extend(&polygon_pixels(tv)); }
    flood_outline(&mut img, &all, OUTLINE);
    save_raw(&img, "body_idle.png", dir);
}

fn gen_eyes(dir: &Path, size: u32, hw: f32, hh: f32) {
    for mood in ["idle", "happy", "hungry", "tired", "lonely", "playful", "sick"] {
        let squint = mood == "tired" || mood == "sick";
        let glow = mood == "idle" || mood == "playful" || mood == "happy";
        let tear = mood == "lonely";
        let h = if squint { hh * 0.5 } else { hh };
        let mut img = new_canvas(size, size);
        let c = size as f32 / 2.0;
        let eye = diamond_pixels(c, c, hw, h);
        let color = if glow { EYE_GLOW } else { PUPIL };
        px_set(&mut img, &eye, color);
        // Thick slit pupil when glowing (2px wide)
        if glow {
            for dy in -(h as i32)..=(h as i32) {
                for dx in 0..=1 {
                    let pos = (c as i32 + dx, c as i32 + dy);
                    let neg = (c as i32 - dx, c as i32 + dy);
                    if eye.contains(&pos) { put(&mut img, pos.0, pos.1, PUPIL); }
                    if eye.contains(&neg) { put(&mut img, neg.0, neg.1, PUPIL); }
                }
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

fn gen_mantle(dir: &Path, size: u32) {
    let s = size as f32;
    let mut img = new_canvas(size, (s * 0.75) as u32);
    let verts = vec![
        (s * 0.5, s * 0.05), (s * 0.8, s * 0.15), (s * 0.9, s * 0.4),
        (s * 0.85, s * 0.65), (s * 0.15, s * 0.65), (s * 0.1, s * 0.4),
        (s * 0.2, s * 0.15),
    ];
    smooth_shade(&mut img, &verts, (s * 0.5, s * 0.4), MANTLE, (-0.3, -1.0));
    flood_outline(&mut img, &polygon_pixels(&verts), OUTLINE);
    save_raw(&img, "mantle_idle.png", dir);
}

fn gen_tentacles(dir: &Path, size: u32) {
    let s = size as f32;
    // Front tentacle
    let mut img = new_canvas(size, (s * 2.5) as u32);
    let verts = [(s * 0.3, 2.0), (s * 0.7, 2.0), (s * 0.6, s * 1.8), (s * 0.5, s * 2.2), (s * 0.4, s * 1.8)];
    smooth_shade(&mut img, &verts, (s * 0.5, s * 1.1), TENTACLE, (0.0, -1.0));
    let tip = diamond_pixels(s * 0.5, s * 2.1, s * 0.12, s * 0.1);
    px_set(&mut img, &tip, TENTACLE_TIP);
    flood_outline(&mut img, &polygon_pixels(&verts), OUTLINE);
    save_raw(&img, "tentacle_front_left_idle.png", dir);
    save_raw(&flip_h(&img), "tentacle_front_right_idle.png", dir);

    // Back tentacle — longer
    let mut img2 = new_canvas(size, (s * 2.8) as u32);
    let verts2 = [(s * 0.35, 2.0), (s * 0.65, 2.0), (s * 0.58, s * 2.0), (s * 0.5, s * 2.5), (s * 0.42, s * 2.0)];
    smooth_shade(&mut img2, &verts2, (s * 0.5, s * 1.3), TENTACLE, (0.0, -1.0));
    let tip2 = diamond_pixels(s * 0.5, s * 2.4, s * 0.1, s * 0.08);
    px_set(&mut img2, &tip2, TENTACLE_TIP);
    flood_outline(&mut img2, &polygon_pixels(&verts2), OUTLINE);
    save_raw(&img2, "tentacle_back_left_idle.png", dir);
    save_raw(&flip_h(&img2), "tentacle_back_right_idle.png", dir);
}

fn main() {
    let base = base_dir();
    let egg = base.join("egg"); let cub = base.join("cub"); let young = base.join("young");
    let adult = base.join("adult"); let elder = base.join("elder");

    println!("Generating Nyxal sprites (all stages)\n");

    println!("=== Egg ==="); gen_egg(&egg);
    println!("\n=== Cub ==="); gen_cub_body(&cub);
    gen_eyes(&cub, 38, 14.0, 14.0); gen_mantle(&cub, 40); gen_tentacles(&cub, 16);
    println!("\n=== Young ==="); gen_cub_body(&young);
    gen_eyes(&young, 34, 12.0, 13.0); gen_mantle(&young, 44); gen_tentacles(&young, 18);
    println!("\n=== Adult ==="); gen_adult_body(&adult);
    gen_eyes(&adult, 30, 10.0, 11.0); gen_mantle(&adult, 48); gen_tentacles(&adult, 20);
    println!("\n=== Elder ==="); gen_adult_body(&elder);
    gen_eyes(&elder, 30, 10.0, 11.0); gen_mantle(&elder, 48); gen_tentacles(&elder, 20);

    // Clean old root sprites
    for entry in std::fs::read_dir(&base).unwrap().flatten() {
        if entry.path().extension().map(|e| e == "png").unwrap_or(false) { std::fs::remove_file(entry.path()).ok(); }
    }
    println!("\nDone!");
}
