//! Kokoro — Pylum (Highland Bird) Species Sprite Generator
//!
//! Rounder egg-shaped body, wings, pointed beak, tail feathers.
//! Warm golden palette.
//!
//! Usage: cargo run --bin generate_pylum_sprites

mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

// --- Color palette — warm bird tones ---
const BODY: Rgba<u8> = Rgba([255, 220, 140, 255]);
const BODY_HI: Rgba<u8> = Rgba([255, 235, 170, 255]);
const BODY_SH: Rgba<u8> = Rgba([230, 190, 110, 255]);
const WING_BASE: Rgba<u8> = Rgba([240, 200, 120, 255]);
const WING_TIP: Rgba<u8> = Rgba([200, 160, 90, 255]);
const OUTLINE: Rgba<u8> = Rgba([60, 45, 30, 255]);
const PUPIL: Rgba<u8> = Rgba([30, 30, 45, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const EYE_WHITE: Rgba<u8> = Rgba([250, 250, 255, 255]);
const BEAK_COL: Rgba<u8> = Rgba([255, 160, 60, 255]);
const BEAK_DARK: Rgba<u8> = Rgba([220, 130, 40, 255]);
const PINK: Rgba<u8> = Rgba([240, 160, 165, 255]);
const TEAR: Rgba<u8> = Rgba([140, 195, 250, 255]);

fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("pylum")
}

// ---------------------------------------------------------------------------
// Body — egg-shaped
// ---------------------------------------------------------------------------

fn gen_body(dir: &Path) {
    let (w, h) = (48u32, 44u32);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0 + 2.0);

    let body = ellipse_pixels(cx, cy, 18.0, 16.0);
    for &(x, y) in &body {
        let t = (y as f32 - (cy - 16.0)) / 32.0;
        let c = if t < 0.3 {
            lerp(BODY_HI, BODY, t / 0.3)
        } else if t < 0.7 {
            BODY
        } else {
            lerp(BODY, BODY_SH, (t - 0.7) / 0.3)
        };
        put(&mut img, x, y, c);
    }

    flood_outline(&mut img, &body, OUTLINE);
    save(&img, "body_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Wings
// ---------------------------------------------------------------------------

fn gen_wing(dir: &Path, side: &str) {
    let (w, h) = (22u32, 18u32);
    let mut img = new_canvas(w, h);

    let cx = if side == "left" {
        w as f32 - 5.0
    } else {
        5.0
    };
    let pts = ellipse_pixels(cx, h as f32 / 2.0, 12.0, 7.0);

    for &(x, y) in &pts {
        let t = if side == "left" {
            ((cx - x as f32) / 12.0).clamp(0.0, 1.0)
        } else {
            ((x as f32 - cx) / 12.0).clamp(0.0, 1.0)
        };
        let c = lerp(WING_BASE, WING_TIP, t);
        put(&mut img, x, y, c);
    }

    flood_outline(&mut img, &pts, OUTLINE);
    save(&img, &format!("wing_{side}_idle.png"), dir);
}

// ---------------------------------------------------------------------------
// Eyes — round, expressive
// ---------------------------------------------------------------------------

struct PylumEye {
    pupil_dx: i32,
    lid_rows: i32,
    sparkle: bool,
    tear: bool,
}

impl Default for PylumEye {
    fn default() -> Self {
        PylumEye {
            pupil_dx: 0,
            lid_rows: 0,
            sparkle: true,
            tear: false,
        }
    }
}

fn pylum_eye(p: &PylumEye) -> image::RgbaImage {
    let (w, h) = (12u32, 12u32);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (w as i32 / 2, h as i32 / 2);

    let eye_pts = ellipse_pixels(cx as f32, cy as f32, 4.0, 4.0);
    px_set(&mut img, &eye_pts, EYE_WHITE);
    flood_outline(&mut img, &eye_pts, OUTLINE);

    let pupil_pts = ellipse_pixels((cx + p.pupil_dx) as f32, cy as f32, 2.0, 2.0);
    px_set(&mut img, &pupil_pts, PUPIL);

    if p.sparkle {
        px(&mut img, &[(cx - 1, cy - 2)], WHITE);
    }
    if p.tear {
        px(&mut img, &[(cx, cy + 4), (cx, cy + 5)], TEAR);
    }

    for r in 0..p.lid_rows {
        for x in (cx - 4)..=(cx + 4) {
            if eye_pts.contains(&(x, cy - 4 + r)) {
                put(&mut img, x, cy - 4 + r, OUTLINE);
            }
        }
    }

    img
}

fn gen_eyes(dir: &Path) {
    let moods: Vec<(&str, PylumEye)> = vec![
        ("idle", PylumEye { sparkle: true, ..Default::default() }),
        ("hungry", PylumEye { lid_rows: 1, ..Default::default() }),
        ("tired", PylumEye { lid_rows: 3, sparkle: false, ..Default::default() }),
        ("lonely", PylumEye { tear: true, ..Default::default() }),
        ("playful", PylumEye { sparkle: true, ..Default::default() }),
        ("sick", PylumEye { lid_rows: 2, sparkle: false, ..Default::default() }),
        ("sleeping", PylumEye { lid_rows: 5, sparkle: false, ..Default::default() }),
    ];

    for (mood, params) in &moods {
        let left = pylum_eye(params);
        save(&left, &format!("eye_left_{mood}.png"), dir);
        let right = flip_h(&left);
        save(&right, &format!("eye_right_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Beak (replaces mouth)
// ---------------------------------------------------------------------------

fn gen_beak(dir: &Path) {
    let styles: Vec<(&str, &str)> = vec![
        ("idle", "closed"),
        ("hungry", "open"),
        ("tired", "closed"),
        ("lonely", "closed"),
        ("playful", "open_wide"),
        ("sick", "closed"),
        ("sleeping", "closed"),
    ];

    for (mood, style) in &styles {
        let (w, h) = (12u32, 10u32);
        let mut img = new_canvas(w, h);
        let cx = w as i32 / 2;

        match *style {
            "closed" => {
                let mut pts = Shape::new();
                for y in 3..7 {
                    let span = (4 - (y - 3)).max(1);
                    for x in (cx - span)..=(cx + span) {
                        pts.insert((x, y));
                    }
                }
                px_set(&mut img, &pts, BEAK_COL);
                flood_outline(&mut img, &pts, OUTLINE);
            }
            "open" => {
                let mut upper = Shape::new();
                for y in 2..5 {
                    let span = (4 - (y - 2)).max(1);
                    for x in (cx - span)..=(cx + span) {
                        upper.insert((x, y));
                    }
                }
                px_set(&mut img, &upper, BEAK_COL);

                let mut lower = Shape::new();
                for y in 6..8 {
                    let span = (3 - (y - 6)).max(1);
                    for x in (cx - span)..=(cx + span) {
                        lower.insert((x, y));
                    }
                }
                px_set(&mut img, &lower, BEAK_DARK);

                let all: Shape = upper.union(&lower).copied().collect();
                flood_outline(&mut img, &all, OUTLINE);
            }
            "open_wide" => {
                let mut upper = Shape::new();
                for y in 1..4 {
                    let span = (5 - (y - 1)).max(1);
                    for x in (cx - span)..=(cx + span) {
                        upper.insert((x, y));
                    }
                }
                px_set(&mut img, &upper, BEAK_COL);

                let mut lower = Shape::new();
                for y in 5..9 {
                    let span = (4 - (y - 5)).max(1);
                    for x in (cx - span)..=(cx + span) {
                        lower.insert((x, y));
                    }
                }
                px_set(&mut img, &lower, BEAK_DARK);

                // Pink mouth interior
                let mut interior = Shape::new();
                for x in (cx - 2)..=(cx + 2) {
                    interior.insert((x, 4));
                }
                px_set(&mut img, &interior, PINK);

                let all: Shape = upper.union(&lower).chain(&interior).copied().collect();
                flood_outline(&mut img, &all, OUTLINE);
            }
            _ => {}
        }

        save(&img, &format!("beak_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Tail — three feathers
// ---------------------------------------------------------------------------

fn gen_tail(dir: &Path) {
    let (w, h) = (16u32, 18u32);
    let mut img = new_canvas(w, h);
    let cx = w as i32 / 2;

    for offset in [-3, 0, 3] {
        let mut feather = Shape::new();
        for y in 4..(h as i32 - 1) {
            let span = (3 - (y - 10).abs() / 3).max(1);
            for x in (cx + offset - span)..=(cx + offset + span) {
                feather.insert((x, y));
            }
        }
        let t_base = if offset == 0 { 0.3 } else { 0.5 };
        let c = lerp(WING_BASE, WING_TIP, t_base);
        px_set(&mut img, &feather, c);
        flood_outline(&mut img, &feather, OUTLINE);
    }

    save(&img, "tail_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let dir = out_dir();
    println!("Generating Pylum sprites → {}", dir.display());

    gen_body(&dir);
    gen_wing(&dir, "left");
    gen_wing(&dir, "right");
    gen_eyes(&dir);
    gen_beak(&dir);
    gen_tail(&dir);

    println!("\nDone!");
}
