//! Kokoro — Skael (Cave Reptile) Species Sprite Generator
//!
//! Elongated angular body, crests/horns, slitted predator eyes,
//! wide snout, thick scaled tail. Cool green palette.
//!
//! Usage: cargo run --bin generate_skael_sprites

mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

// --- Color palette — cool reptile tones ---
const BODY: Rgba<u8> = Rgba([120, 180, 130, 255]);
const BODY_HI: Rgba<u8> = Rgba([150, 210, 155, 255]);
const BODY_SH: Rgba<u8> = Rgba([90, 150, 105, 255]);
const BODY_SH2: Rgba<u8> = Rgba([70, 125, 85, 255]);
const SCALE_COL: Rgba<u8> = Rgba([100, 165, 115, 255]);
const CREST_COL: Rgba<u8> = Rgba([160, 100, 80, 255]);
const CREST_TIP: Rgba<u8> = Rgba([190, 120, 90, 255]);
const OUTLINE: Rgba<u8> = Rgba([35, 50, 40, 255]);
const PUPIL: Rgba<u8> = Rgba([30, 30, 30, 255]);
const IRIS: Rgba<u8> = Rgba([200, 160, 50, 255]);
const EYE_WHITE: Rgba<u8> = Rgba([230, 240, 200, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const SNOUT_COL: Rgba<u8> = Rgba([80, 100, 80, 255]);
const TEAR: Rgba<u8> = Rgba([140, 195, 250, 255]);

fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("skael")
}

// ---------------------------------------------------------------------------
// Body — elongated with scale texture
// ---------------------------------------------------------------------------

fn gen_body(dir: &Path) {
    let (w, h) = (42u32, 50u32);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);

    let body = ellipse_pixels(cx, cy, 16.0, 20.0);

    for &(x, y) in &body {
        let t = (y as f32 - (cy - 20.0)) / 40.0;
        let c = if t < 0.25 {
            lerp(BODY_HI, BODY, t / 0.25)
        } else if t < 0.6 {
            BODY
        } else {
            lerp(BODY, BODY_SH, (t - 0.6) / 0.4)
        };
        put(&mut img, x, y, c);
    }

    // Scale pattern — horizontal lines for texture
    for y_off in (-15..=15).step_by(4) {
        let row_y = cy as i32 + y_off;
        for &(x, y) in &body {
            if y == row_y {
                put(&mut img, x, y, SCALE_COL);
            }
        }
    }

    flood_outline(&mut img, &body, OUTLINE);
    save(&img, "body_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Crests (horns)
// ---------------------------------------------------------------------------

fn gen_crest(dir: &Path, side: &str) {
    let (w, h) = (10u32, 20u32);
    let mut img = new_canvas(w, h);

    let mut pts = Shape::new();
    if side == "left" {
        for y in 2..(h as i32 - 2) {
            let span = ((h as i32 - 2 - y) / 3).max(1);
            let base_x = w as i32 - 4;
            for x in (base_x - span)..=(base_x + 1) {
                pts.insert((x, y));
            }
        }
    } else {
        for y in 2..(h as i32 - 2) {
            let span = ((h as i32 - 2 - y) / 3).max(1);
            let base_x = 4;
            for x in (base_x - 1)..=(base_x + span) {
                pts.insert((x, y));
            }
        }
    }

    for &(x, y) in &pts {
        let t = ((h as f32 - y as f32) / h as f32).min(1.0);
        let c = lerp(CREST_COL, CREST_TIP, t);
        put(&mut img, x, y, c);
    }

    flood_outline(&mut img, &pts, OUTLINE);
    save(&img, &format!("crest_{side}_idle.png"), dir);
}

// ---------------------------------------------------------------------------
// Eyes — slitted reptile pupils
// ---------------------------------------------------------------------------

struct ReptileEye {
    slit_open: i32,
    lid_rows: i32,
    sparkle: bool,
    tear: bool,
}

impl Default for ReptileEye {
    fn default() -> Self {
        ReptileEye {
            slit_open: 2,
            lid_rows: 0,
            sparkle: false,
            tear: false,
        }
    }
}

fn reptile_eye(p: &ReptileEye) -> image::RgbaImage {
    let (w, h) = (12u32, 12u32);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (w as i32 / 2, h as i32 / 2);

    // Eye shape
    let eye_pts = ellipse_pixels(cx as f32, cy as f32, 4.0, 3.0);
    px_set(&mut img, &eye_pts, EYE_WHITE);

    // Golden iris
    let iris_pts = ellipse_pixels(cx as f32, cy as f32, 3.0, 3.0);
    px_set(&mut img, &iris_pts, IRIS);

    // Vertical slit pupil
    for dy in -3..=3 {
        for dx in (-p.slit_open / 2)..=(p.slit_open / 2) {
            let pos = (cx + dx, cy + dy);
            if iris_pts.contains(&pos) {
                put(&mut img, pos.0, pos.1, PUPIL);
            }
        }
    }

    if p.sparkle {
        px(&mut img, &[(cx - 1, cy - 1)], WHITE);
    }
    if p.tear {
        px(&mut img, &[(cx, cy + 3), (cx, cy + 4)], TEAR);
    }

    // Upper lid
    for r in 0..p.lid_rows {
        for x in (cx - 4)..=(cx + 4) {
            if eye_pts.contains(&(x, cy - 3 + r)) {
                put(&mut img, x, cy - 3 + r, OUTLINE);
            }
        }
    }

    flood_outline(&mut img, &eye_pts, OUTLINE);
    img
}

fn gen_eyes(dir: &Path) {
    let moods: Vec<(&str, ReptileEye)> = vec![
        ("idle", ReptileEye { slit_open: 1, sparkle: true, ..Default::default() }),
        ("hungry", ReptileEye { slit_open: 2, ..Default::default() }),
        ("tired", ReptileEye { slit_open: 1, lid_rows: 2, ..Default::default() }),
        ("lonely", ReptileEye { slit_open: 1, tear: true, ..Default::default() }),
        ("playful", ReptileEye { slit_open: 2, sparkle: true, ..Default::default() }),
        ("sick", ReptileEye { slit_open: 1, lid_rows: 1, ..Default::default() }),
        ("sleeping", ReptileEye { slit_open: 0, lid_rows: 4, ..Default::default() }),
    ];

    for (mood, params) in &moods {
        let left = reptile_eye(params);
        save(&left, &format!("eye_left_{mood}.png"), dir);
        let right = flip_h(&left);
        save(&right, &format!("eye_right_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Snout (replaces mouth)
// ---------------------------------------------------------------------------

fn gen_snout(dir: &Path) {
    let styles: Vec<(&str, &str)> = vec![
        ("idle", "closed"),
        ("hungry", "open"),
        ("tired", "closed"),
        ("lonely", "closed"),
        ("playful", "grin"),
        ("sick", "closed"),
        ("sleeping", "closed"),
    ];

    for (mood, style) in &styles {
        let (w, h) = (16u32, 10u32);
        let mut img = new_canvas(w, h);
        let cx = w as i32 / 2;

        match *style {
            "closed" => {
                let mut pts = Shape::new();
                for y in 3i32..7 {
                    let span = 5 - (y - 5).abs();
                    for x in (cx - span)..=(cx + span) {
                        pts.insert((x, y));
                    }
                }
                px_set(&mut img, &pts, SNOUT_COL);
                px(&mut img, &[(cx - 2, 4), (cx + 2, 4)], OUTLINE);
                flood_outline(&mut img, &pts, OUTLINE);
            }
            "open" => {
                let mut upper = Shape::new();
                for y in 2i32..5 {
                    let span = 5 - (y - 3).abs();
                    for x in (cx - span)..=(cx + span) {
                        upper.insert((x, y));
                    }
                }
                px_set(&mut img, &upper, SNOUT_COL);

                let mut lower = Shape::new();
                for y in 6i32..9 {
                    let span = 4 - (y - 7).abs();
                    for x in (cx - span)..=(cx + span) {
                        lower.insert((x, y));
                    }
                }
                px_set(&mut img, &lower, SNOUT_COL);
                px(&mut img, &[(cx - 2, 5), (cx, 5), (cx + 2, 5)], WHITE);

                let all: Shape = upper.union(&lower).copied().collect();
                flood_outline(&mut img, &all, OUTLINE);
            }
            "grin" => {
                let mut pts = Shape::new();
                for y in 3i32..7 {
                    let span = 6 - (y - 5).abs();
                    for x in (cx - span)..=(cx + span) {
                        pts.insert((x, y));
                    }
                }
                px_set(&mut img, &pts, SNOUT_COL);
                // Grin line
                for x in (cx - 4)..=(cx + 4) {
                    px(&mut img, &[(x, 5)], OUTLINE);
                }
                // Fangs
                px(&mut img, &[(cx - 3, 6), (cx + 3, 6)], WHITE);
                flood_outline(&mut img, &pts, OUTLINE);
            }
            _ => {}
        }

        save(&img, &format!("snout_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Tail — thick, scaled
// ---------------------------------------------------------------------------

fn gen_tail(dir: &Path) {
    let (w, h) = (14u32, 24u32);
    let mut img = new_canvas(w, h);
    let cx = w as i32 / 2;

    let mut pts = Shape::new();
    for y in 2..(h as i32 - 1) {
        let width = (5.0 * (1.0 - (y - 2) as f32 / (h as f32 - 3.0) * 0.6)) as i32;
        let width = width.max(1);
        for x in (cx - width)..=(cx + width) {
            pts.insert((x, y));
        }
    }

    for &(x, y) in &pts {
        let t = y as f32 / h as f32;
        let c = lerp(BODY, BODY_SH2, t);
        put(&mut img, x, y, c);
    }

    // Scale bands
    for y_off in (4..(h as i32 - 2)).step_by(3) {
        for &(x, y) in &pts {
            if y == y_off {
                put(&mut img, x, y, SCALE_COL);
            }
        }
    }

    flood_outline(&mut img, &pts, OUTLINE);
    save(&img, "tail_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let dir = out_dir();
    println!("Generating Skael sprites → {}", dir.display());

    gen_body(&dir);
    gen_crest(&dir, "left");
    gen_crest(&dir, "right");
    gen_eyes(&dir);
    gen_snout(&dir);
    gen_tail(&dir);

    println!("\nDone!");
}
