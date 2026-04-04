//! Kokoro — Nyxal (Abyssal Squid) Sprite Generator
//!
//! Generates modular pixel art sprites for the Nyxal species:
//! bulbous mantle, large bioluminescent eyes, and four tentacles.
//! Deep-sea palette with dark purples, bioluminescent accents.
//!
//! Usage: cargo run --bin generate_nyxal_sprites

mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

// --- Color palette — deep sea bioluminescence ---
const BODY: Rgba<u8> = Rgba([95, 60, 130, 255]);
const BODY_HI: Rgba<u8> = Rgba([120, 80, 155, 255]);
const BODY_SH: Rgba<u8> = Rgba([70, 42, 105, 255]);
const BODY_SH2: Rgba<u8> = Rgba([55, 32, 85, 255]);
const MANTLE: Rgba<u8> = Rgba([85, 50, 120, 255]);
const MANTLE_HI: Rgba<u8> = Rgba([110, 70, 145, 255]);
const OUTLINE: Rgba<u8> = Rgba([25, 18, 40, 255]);
const PUPIL: Rgba<u8> = Rgba([15, 15, 30, 255]);
const EYE_GLOW: Rgba<u8> = Rgba([40, 180, 200, 255]);
const EYE_GLOW_HI: Rgba<u8> = Rgba([80, 220, 240, 255]);
const TENTACLE: Rgba<u8> = Rgba([100, 65, 140, 255]);
const TENTACLE_TIP: Rgba<u8> = Rgba([50, 170, 180, 255]);
const PINK: Rgba<u8> = Rgba([200, 120, 180, 255]);
const TEAR: Rgba<u8> = Rgba([100, 160, 220, 255]);
const GREEN_SICK: Rgba<u8> = Rgba([120, 180, 110, 255]);

fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("nyxal")
}

// ---------------------------------------------------------------------------
// Body — bulbous, soft squid body
// ---------------------------------------------------------------------------

fn gen_body(dir: &Path) {
    let (w, h) = (50, 50);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (25.0_f32, 24.0_f32);

    // Main body — slightly wider than tall, squid-like
    let body = ellipse_pixels(cx, cy, 17.0, 15.0);

    let top = cy - 15.0;
    let bot = cy + 15.0;
    let span = bot - top;

    for &(x, y) in &body {
        let t = (y as f32 - top) / span;
        let dx = ((x as f32 - cx) / 17.0).abs().min(1.0);

        let mut c = if t < 0.2 {
            lerp(BODY_HI, BODY, t / 0.2)
        } else if t < 0.5 {
            BODY
        } else if t < 0.75 {
            lerp(BODY, BODY_SH, (t - 0.5) / 0.25)
        } else {
            lerp(BODY_SH, BODY_SH2, (t - 0.75) / 0.25)
        };

        if dx > 0.6 {
            let edge_t = (dx - 0.6) / 0.4;
            c = lerp(c, BODY_SH, edge_t * 0.5);
        }

        put(&mut img, x, y, c);
    }

    // Specular highlight
    let hi_pts = ellipse_pixels(cx - 4.0, cy - 7.0, 4.0, 2.5);
    for &(x, y) in &hi_pts {
        if body.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.5));
        }
    }

    flood_outline(&mut img, &body, OUTLINE);
    save(&img, "body_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Mantle — dome-shaped top piece
// ---------------------------------------------------------------------------

fn gen_mantle(dir: &Path) {
    let (w, h) = (30, 22);
    let mut img = new_canvas(w, h);

    let dome = ellipse_pixels(15.0, 12.0, 12.0, 9.0);
    for &(x, y) in &dome {
        let t = (y as f32 - 3.0) / 18.0;
        let c = if t < 0.3 {
            lerp(MANTLE_HI, MANTLE, t / 0.3)
        } else {
            lerp(MANTLE, BODY_SH, (t - 0.3) / 0.7)
        };
        put(&mut img, x, y, c);
    }

    // Small highlight
    let hi = ellipse_pixels(12.0, 8.0, 3.0, 2.0);
    for &(x, y) in &hi {
        if dome.contains(&(x, y)) {
            put(&mut img, x, y, MANTLE_HI);
        }
    }

    flood_outline(&mut img, &dome, OUTLINE);
    save(&img, "mantle_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Eyes — large, bioluminescent
// ---------------------------------------------------------------------------

struct EyeParams {
    lid_rows: i32,
    pupil_size: i32,
    glow: bool,
    tear: bool,
    sparkle: bool,
    squint: bool,
}

impl Default for EyeParams {
    fn default() -> Self {
        EyeParams {
            lid_rows: 0,
            pupil_size: 3,
            glow: true,
            tear: false,
            sparkle: false,
            squint: false,
        }
    }
}

fn gen_eye(p: &EyeParams) -> image::RgbaImage {
    let (w, h) = (14, 12);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (7.0_f32, 6.0_f32);

    // Eye globe — large, round
    let ry = if p.squint { 3.0 } else { 4.5 };
    let globe = ellipse_pixels(cx, cy, 5.0, ry);

    // Fill with glow gradient
    for &(x, y) in &globe {
        let dist = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt()) / 5.0;
        let c = if p.glow {
            lerp(EYE_GLOW_HI, EYE_GLOW, dist.min(1.0))
        } else {
            lerp(Rgba([180, 180, 190, 255]), Rgba([140, 140, 155, 255]), dist.min(1.0))
        };
        put(&mut img, x, y, c);
    }

    // Pupil — vertical slit (squid-like)
    let pcx = cx as i32;
    let pcy = cy as i32;
    let ps = p.pupil_size;
    for dy in -(ps)..=(ps) {
        put(&mut img, pcx, pcy + dy, PUPIL);
    }

    // Upper eyelid
    for row in 0..p.lid_rows {
        for x in (cx as i32 - 5)..=(cx as i32 + 5) {
            if globe.contains(&(x, cy as i32 - 4 + row)) {
                put(&mut img, x, cy as i32 - 4 + row, OUTLINE);
            }
        }
    }

    // Sparkle
    if p.sparkle {
        put(&mut img, pcx - 2, pcy - 2, Rgba([255, 255, 255, 255]));
    }

    // Tear
    if p.tear {
        let tear_pts = ellipse_pixels(cx + 2.0, cy + 5.0, 1.5, 2.0);
        px_set(&mut img, &tear_pts, TEAR);
    }

    flood_outline(&mut img, &globe, OUTLINE);
    img
}

fn gen_eyes(dir: &Path) {
    // Idle — open, glowing
    let idle = gen_eye(&EyeParams { sparkle: true, ..Default::default() });
    save(&idle, "eye_left_idle.png", dir);
    save(&flip_h(&idle), "eye_right_idle.png", dir);

    // Hungry — half-lidded, dimmer
    let hungry = gen_eye(&EyeParams { lid_rows: 2, glow: false, ..Default::default() });
    save(&hungry, "eye_left_hungry.png", dir);
    save(&flip_h(&hungry), "eye_right_hungry.png", dir);

    // Tired — squinting
    let tired = gen_eye(&EyeParams { squint: true, lid_rows: 1, glow: false, ..Default::default() });
    save(&tired, "eye_left_tired.png", dir);
    save(&flip_h(&tired), "eye_right_tired.png", dir);

    // Lonely — teary
    let lonely = gen_eye(&EyeParams { tear: true, ..Default::default() });
    save(&lonely, "eye_left_lonely.png", dir);
    save(&flip_h(&lonely), "eye_right_lonely.png", dir);

    // Playful — wide, bright
    let playful = gen_eye(&EyeParams { sparkle: true, pupil_size: 2, ..Default::default() });
    save(&playful, "eye_left_playful.png", dir);
    save(&flip_h(&playful), "eye_right_playful.png", dir);

    // Sick — dim, no glow
    let sick = gen_eye(&EyeParams { glow: false, pupil_size: 4, ..Default::default() });
    save(&sick, "eye_left_sick.png", dir);
    save(&flip_h(&sick), "eye_right_sick.png", dir);

    // Sleeping — closed
    let (w, h) = (14, 12);
    let mut closed = new_canvas(w, h);
    let cy = 6;
    for x in 3..=11 {
        put(&mut closed, x, cy, OUTLINE);
        put(&mut closed, x, cy + 1, OUTLINE);
    }
    save(&closed, "eye_left_sleeping.png", dir);
    save(&flip_h(&closed), "eye_right_sleeping.png", dir);
}

// ---------------------------------------------------------------------------
// Tentacles — tapered with bioluminescent tips
// ---------------------------------------------------------------------------

fn gen_tentacles(dir: &Path) {
    let (w, h) = (12, 32);

    // Front tentacle — shorter, thicker
    let mut front = new_canvas(w, h);
    let mut front_pts = Shape::new();
    for y in 0..28 {
        let t = y as f32 / 28.0;
        let half_w = (4.0 * (1.0 - t * 0.5)) as i32; // tapers
        let cx = 6;
        for x in (cx - half_w)..=(cx + half_w) {
            let c = if t > 0.75 {
                lerp(TENTACLE, TENTACLE_TIP, (t - 0.75) / 0.25)
            } else {
                lerp(TENTACLE, BODY_SH, t * 0.3)
            };
            put(&mut front, x, y, c);
            front_pts.insert((x, y));
        }
    }
    flood_outline(&mut front, &front_pts, OUTLINE);
    save(&front, "tentacle_front_left_idle.png", dir);
    save(&flip_h(&front), "tentacle_front_right_idle.png", dir);

    // Back tentacle — longer, thinner
    let mut back = new_canvas(w, h);
    let mut back_pts = Shape::new();
    for y in 0..30 {
        let t = y as f32 / 30.0;
        let half_w = (3.0 * (1.0 - t * 0.6)) as i32;
        let cx = 6;
        for x in (cx - half_w)..=(cx + half_w) {
            let c = if t > 0.8 {
                lerp(TENTACLE, TENTACLE_TIP, (t - 0.8) / 0.2)
            } else {
                lerp(TENTACLE, BODY_SH, t * 0.4)
            };
            put(&mut back, x, y, c);
            back_pts.insert((x, y));
        }
    }
    flood_outline(&mut back, &back_pts, OUTLINE);
    save(&back, "tentacle_back_left_idle.png", dir);
    save(&flip_h(&back), "tentacle_back_right_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create nyxal sprite directory");
    println!("Generating Nyxal sprites -> {}\n", dir.display());

    println!("[Body]");
    gen_body(&dir);

    println!("\n[Mantle]");
    gen_mantle(&dir);

    println!("\n[Eyes — all moods]");
    gen_eyes(&dir);

    println!("\n[Tentacles]");
    gen_tentacles(&dir);

    println!("\nDone! All sprites saved.");
}
