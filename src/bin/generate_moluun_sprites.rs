//! Kokoro — Moluun (Forest Mammal) Sprite Generator
//!
//! Generates modular pixel art sprites matching the idle_00.png reference style:
//! round, chunky character with gradient shading, thick outlines, and personality.
//!
//! Usage: cargo run --bin generate_moluun_sprites

mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

// --- Color palette (sampled from idle_00.png reference) ---
const BODY: Rgba<u8> = Rgba([156, 232, 252, 255]);
const BODY_HI: Rgba<u8> = Rgba([180, 240, 255, 255]);
const BODY_SH: Rgba<u8> = Rgba([120, 195, 225, 255]);
const BODY_SH2: Rgba<u8> = Rgba([100, 170, 205, 255]);
const EAR_INNER: Rgba<u8> = Rgba([130, 200, 230, 255]);
const OUTLINE: Rgba<u8> = Rgba([40, 45, 55, 255]);
const PUPIL: Rgba<u8> = Rgba([50, 50, 65, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const EYE_WHITE: Rgba<u8> = Rgba([245, 248, 255, 255]);
const PINK: Rgba<u8> = Rgba([240, 160, 165, 255]);
const TEAR: Rgba<u8> = Rgba([140, 195, 250, 255]);
const GREEN_SICK: Rgba<u8> = Rgba([165, 215, 145, 255]);
const GRAY_CLOUD: Rgba<u8> = Rgba([185, 185, 195, 255]);
const YELLOW: Rgba<u8> = Rgba([255, 255, 110, 255]);

fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("moluun")
}

fn effects_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("shared")
        .join("effects")
}

// ---------------------------------------------------------------------------
// Body — round, chunky blob with smooth gradient shading
// ---------------------------------------------------------------------------

fn gen_body(dir: &Path) {
    let (w, h) = (52, 52);
    let mut img = new_canvas(w, h);
    let (cx, cy) = (26.0_f32, 23.0_f32);

    // Main body ellipse
    let body = ellipse_pixels(cx, cy, 19.0, 18.0);
    // Feet — overlap body bottom
    let foot_l = ellipse_pixels(18.0, 40.0, 7.0, 5.0);
    let foot_r = ellipse_pixels(34.0, 41.0, 7.0, 5.0);
    // Arms
    let arm_l = ellipse_pixels(8.0, 29.0, 4.0, 5.0);
    let arm_r = ellipse_pixels(44.0, 28.0, 4.0, 5.0);

    let all: Shape = body
        .iter()
        .chain(&foot_l)
        .chain(&foot_r)
        .chain(&arm_l)
        .chain(&arm_r)
        .copied()
        .collect();

    let top = cy - 18.0;
    let bot = all.iter().map(|&(_, y)| y).max().unwrap_or(0) as f32;
    let span = bot - top;

    for &(x, y) in &all {
        let t = (y as f32 - top) / span;
        let dx = ((x as f32 - cx) / 19.0).abs().min(1.0);

        let mut c = if t < 0.18 {
            lerp(BODY_HI, BODY, t / 0.18)
        } else if t < 0.45 {
            BODY
        } else if t < 0.70 {
            lerp(BODY, BODY_SH, (t - 0.45) / 0.25)
        } else {
            lerp(BODY_SH, BODY_SH2, (t - 0.70) / 0.30)
        };

        if dx > 0.60 {
            let edge_t = (dx - 0.60) / 0.40;
            c = lerp(c, BODY_SH, edge_t * 0.6);
        }

        put(&mut img, x, y, c);
    }

    // Specular highlight
    let hi_pts = ellipse_pixels(cx - 3.0, cy - 9.0, 5.0, 3.0);
    for &(x, y) in &hi_pts {
        if body.contains(&(x, y)) {
            let existing = *img.get_pixel(x as u32, y as u32);
            put(&mut img, x, y, lerp(existing, BODY_HI, 0.6));
        }
    }

    // Crotch shadow
    let crotch = ellipse_pixels(26.0, 40.0, 5.0, 2.0);
    for &(x, y) in &crotch {
        if all.contains(&(x, y)) {
            put(&mut img, x, y, BODY_SH2);
        }
    }

    flood_outline(&mut img, &all, OUTLINE);
    save(&img, "body_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Ears
// ---------------------------------------------------------------------------

fn gen_ears(dir: &Path) {
    let (w, h) = (14, 14);
    let mut img = new_canvas(w, h);

    let ear = ellipse_pixels(7.0, 7.0, 5.0, 6.0);
    for &(x, y) in &ear {
        let t = (y as f32 - 1.0) / 12.0;
        let c = if t < 0.3 {
            BODY_HI
        } else if t < 0.6 {
            BODY
        } else {
            BODY_SH
        };
        put(&mut img, x, y, c);
    }

    let inner = ellipse_pixels(7.0, 8.0, 3.0, 3.0);
    px_set(&mut img, &inner, EAR_INNER);
    flood_outline(&mut img, &ear, OUTLINE);
    save(&img, "ear_left_idle.png", dir);

    let right = flip_h(&img);
    save(&right, "ear_right_idle.png", dir);
}

// ---------------------------------------------------------------------------
// Eyes — parameterized builder for all mood variants
// ---------------------------------------------------------------------------

struct EyeParams {
    w: u32,
    h: u32,
    lid_rows: i32,
    pupil_y_off: i32,
    pupil_h: i32,
    sparkle: bool,
    tear: bool,
}

impl Default for EyeParams {
    fn default() -> Self {
        EyeParams {
            w: 13,
            h: 10,
            lid_rows: 1,
            pupil_y_off: 0,
            pupil_h: 4,
            sparkle: false,
            tear: false,
        }
    }
}

fn eye_base(p: &EyeParams) -> image::RgbaImage {
    let mut img = new_canvas(p.w, p.h);
    let (ew_l, ew_t) = (2_i32, 2_i32);
    let (ew_r, ew_b) = (p.w as i32 - 3, p.h as i32 - 3);

    // Eye white area with rounded corners
    let mut eye_pts = Shape::new();
    let corner_r = 2_i32;
    let corners = [
        (ew_l + corner_r, ew_t + corner_r),
        (ew_r - corner_r, ew_t + corner_r),
        (ew_l + corner_r, ew_b - corner_r),
        (ew_r - corner_r, ew_b - corner_r),
    ];

    for y in ew_t..=ew_b {
        for x in ew_l..=ew_r {
            let mut in_corner = false;
            for &(ccx, ccy) in &corners {
                if (x < ew_l + corner_r || x > ew_r - corner_r)
                    && (y < ew_t + corner_r || y > ew_b - corner_r)
                {
                    if (x - ccx) * (x - ccx) + (y - ccy) * (y - ccy) > corner_r * corner_r + 1 {
                        in_corner = true;
                    }
                }
            }
            if !in_corner {
                put(&mut img, x, y, EYE_WHITE);
                eye_pts.insert((x, y));
            }
        }
    }

    // Upper eyelid
    for row in 0..p.lid_rows {
        for x in ew_l..=ew_r {
            if eye_pts.contains(&(x, ew_t + row)) {
                put(&mut img, x, ew_t + row, OUTLINE);
            }
        }
    }

    // Pupil
    let (pw, ph) = (3_i32, p.pupil_h);
    let pcx = (ew_l + ew_r) / 2;
    let pcy = (ew_t + ew_b) / 2 + p.pupil_y_off;
    for py in (pcy - ph / 2)..=(pcy + ph / 2) {
        for ppx in (pcx - pw / 2)..=(pcx + pw / 2) {
            if eye_pts.contains(&(ppx, py)) {
                put(&mut img, ppx, py, PUPIL);
            }
        }
    }

    // Sparkle
    if p.sparkle {
        let (sx, sy) = (pcx - 1, pcy - ph / 2);
        if eye_pts.contains(&(sx, sy)) {
            put(&mut img, sx, sy, WHITE);
        }
    }

    flood_outline(&mut img, &eye_pts, OUTLINE);

    // Tear
    if p.tear {
        let tx = ew_r + 1;
        for ty in (ew_b - 1)..=(ew_b + 3) {
            if tx >= 0 && tx < p.w as i32 && ty >= 0 && ty < p.h as i32 {
                put(&mut img, tx, ty, TEAR);
            }
        }
    }

    img
}

fn eye_sleeping() -> image::RgbaImage {
    let mut img = new_canvas(13, 6);
    px(
        &mut img,
        &[(3, 2), (4, 3), (5, 3), (6, 4), (7, 3), (8, 3), (9, 2)],
        OUTLINE,
    );
    px(&mut img, &[(4, 4), (8, 4)], OUTLINE);
    img
}

fn eye_playful() -> image::RgbaImage {
    let mut img = eye_base(&EyeParams {
        w: 13,
        h: 12,
        lid_rows: 0,
        ..Default::default()
    });
    let (cx, cy) = (6_i32, 6_i32);
    let star = [
        (cx, cy - 2),
        (cx, cy - 1),
        (cx, cy),
        (cx, cy + 1),
        (cx, cy + 2),
        (cx - 2, cy),
        (cx - 1, cy),
        (cx + 1, cy),
        (cx + 2, cy),
        (cx - 1, cy - 1),
        (cx + 1, cy - 1),
        (cx - 1, cy + 1),
        (cx + 1, cy + 1),
    ];
    px(&mut img, &star, PUPIL);
    put(&mut img, cx, cy, WHITE);
    img
}

fn eye_sick() -> image::RgbaImage {
    let mut img = new_canvas(13, 10);
    let mut eye_pts = Shape::new();
    for y in 2..8 {
        for x in 2..11 {
            put(&mut img, x as i32, y as i32, EYE_WHITE);
            eye_pts.insert((x as i32, y as i32));
        }
    }
    flood_outline(&mut img, &eye_pts, OUTLINE);
    let x_pts = [
        (4, 3),
        (5, 4),
        (6, 5),
        (7, 6),
        (8, 3),
        (7, 4),
        (5, 6),
        (4, 7),
        (8, 7),
    ];
    px(&mut img, &x_pts, PUPIL);
    img
}

fn gen_eyes(dir: &Path) {
    let variants: Vec<(&str, image::RgbaImage)> = vec![
        (
            "idle",
            eye_base(&EyeParams {
                lid_rows: 3,
                pupil_y_off: 1,
                pupil_h: 3,
                sparkle: true,
                ..Default::default()
            }),
        ),
        (
            "happy",
            eye_base(&EyeParams {
                w: 13,
                h: 12,
                lid_rows: 1,
                pupil_h: 4,
                sparkle: true,
                ..Default::default()
            }),
        ),
        (
            "hungry",
            eye_base(&EyeParams {
                w: 13,
                h: 12,
                lid_rows: 0,
                pupil_y_off: -1,
                pupil_h: 4,
                sparkle: true,
                tear: true,
                ..Default::default()
            }),
        ),
        (
            "tired",
            eye_base(&EyeParams {
                w: 13,
                h: 8,
                lid_rows: 4,
                pupil_y_off: 1,
                pupil_h: 2,
                ..Default::default()
            }),
        ),
        ("sleeping", eye_sleeping()),
        (
            "lonely",
            eye_base(&EyeParams {
                w: 13,
                h: 12,
                lid_rows: 2,
                pupil_y_off: 2,
                pupil_h: 3,
                tear: true,
                ..Default::default()
            }),
        ),
        ("playful", eye_playful()),
        ("sick", eye_sick()),
    ];

    for (mood, img) in &variants {
        save(img, &format!("eye_left_{mood}.png"), dir);
        let right = flip_h(img);
        save(&right, &format!("eye_right_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Mouths — mood variants
// ---------------------------------------------------------------------------

fn mouth_idle() -> image::RgbaImage {
    let mut img = new_canvas(12, 6);
    px(
        &mut img,
        &[(3, 1), (4, 2), (5, 2), (6, 1), (7, 2), (8, 2), (9, 1)],
        OUTLINE,
    );
    img
}

fn mouth_happy() -> image::RgbaImage {
    let mut img = new_canvas(14, 8);
    px(
        &mut img,
        &[
            (3, 1),
            (4, 1),
            (9, 1),
            (10, 1),
            (2, 2),
            (11, 2),
            (2, 3),
            (11, 3),
            (2, 4),
            (11, 4),
            (3, 5),
            (4, 5),
            (5, 5),
            (6, 5),
            (7, 5),
            (8, 5),
            (9, 5),
            (10, 5),
        ],
        OUTLINE,
    );
    for y in 2..5 {
        for x in 3..11 {
            put(&mut img, x as i32, y as i32, PINK);
        }
    }
    px(
        &mut img,
        &[
            (5, 1),
            (6, 1),
            (7, 1),
            (8, 1),
            (5, 2),
            (6, 2),
            (7, 2),
            (8, 2),
        ],
        WHITE,
    );
    img
}

fn mouth_hungry() -> image::RgbaImage {
    let mut img = new_canvas(10, 10);
    let dark_inside = Rgba([70, 35, 40, 255]);
    let pts = ellipse_pixels(5.0, 5.0, 3.0, 3.0);
    px_set(&mut img, &pts, dark_inside);
    flood_outline(&mut img, &pts, OUTLINE);
    px(&mut img, &[(4, 6), (5, 6), (6, 6)], PINK);
    img
}

fn mouth_tired() -> image::RgbaImage {
    let mut img = new_canvas(12, 10);
    let dark_inside = Rgba([70, 35, 40, 255]);
    let pts = ellipse_pixels(6.0, 5.0, 4.0, 3.0);
    px_set(&mut img, &pts, dark_inside);
    flood_outline(&mut img, &pts, OUTLINE);
    px(&mut img, &[(5, 6), (6, 6), (7, 6), (5, 7), (6, 7)], PINK);
    img
}

fn mouth_sleeping() -> image::RgbaImage {
    let mut img = new_canvas(8, 4);
    px(&mut img, &[(3, 1), (4, 1)], OUTLINE);
    img
}

fn mouth_lonely() -> image::RgbaImage {
    let mut img = new_canvas(12, 6);
    px(
        &mut img,
        &[(3, 4), (4, 3), (5, 2), (6, 2), (7, 2), (8, 3), (9, 4)],
        OUTLINE,
    );
    img
}

fn mouth_playful() -> image::RgbaImage {
    let mut img = new_canvas(14, 10);
    px(
        &mut img,
        &[
            (3, 1),
            (4, 1),
            (9, 1),
            (10, 1),
            (2, 2),
            (11, 2),
            (2, 3),
            (11, 3),
            (2, 4),
            (11, 4),
            (3, 5),
            (4, 5),
            (5, 5),
            (6, 5),
            (7, 5),
            (8, 5),
            (9, 5),
            (10, 5),
        ],
        OUTLINE,
    );
    for y in 2..5 {
        for x in 3..11 {
            put(&mut img, x as i32, y as i32, PINK);
        }
    }
    px(
        &mut img,
        &[
            (5, 1),
            (6, 1),
            (7, 1),
            (8, 1),
            (5, 2),
            (6, 2),
            (7, 2),
            (8, 2),
        ],
        WHITE,
    );
    // Tongue
    let tongue = [(10, 5), (11, 5), (10, 6), (11, 6), (12, 6), (11, 7), (12, 7)];
    px(&mut img, &tongue, PINK);
    let tongue_shape: Shape = tongue.iter().copied().collect();
    flood_outline(&mut img, &tongue_shape, OUTLINE);
    img
}

fn mouth_sick() -> image::RgbaImage {
    let mut img = new_canvas(12, 6);
    px(
        &mut img,
        &[
            (2, 3),
            (3, 2),
            (4, 3),
            (5, 4),
            (6, 3),
            (7, 2),
            (8, 3),
            (9, 4),
        ],
        OUTLINE,
    );
    px(&mut img, &[(3, 4), (7, 4)], GREEN_SICK);
    img
}

fn gen_mouths(dir: &Path) {
    let variants: Vec<(&str, image::RgbaImage)> = vec![
        ("idle", mouth_idle()),
        ("happy", mouth_happy()),
        ("hungry", mouth_hungry()),
        ("tired", mouth_tired()),
        ("sleeping", mouth_sleeping()),
        ("lonely", mouth_lonely()),
        ("playful", mouth_playful()),
        ("sick", mouth_sick()),
    ];
    for (mood, img) in &variants {
        save(img, &format!("mouth_{mood}.png"), dir);
    }
}

// ---------------------------------------------------------------------------
// Shared effects
// ---------------------------------------------------------------------------

fn gen_effects(dir: &Path) {
    // ZZZ
    let mut img = new_canvas(20, 20);
    px(
        &mut img,
        &[
            // Small z
            (3, 14), (4, 14), (5, 14), (5, 15), (4, 16), (3, 16), (3, 17), (4, 17), (5, 17),
            // Medium Z
            (8, 9), (9, 9), (10, 9), (11, 9), (11, 10), (10, 11), (9, 12), (8, 13), (9, 13),
            (10, 13), (11, 13),
            // Large Z
            (13, 2), (14, 2), (15, 2), (16, 2), (17, 2), (16, 3), (15, 4), (14, 5), (13, 6),
            (13, 7), (14, 7), (15, 7), (16, 7), (17, 7),
        ],
        WHITE,
    );
    save(&img, "zzz.png", dir);

    // Hearts
    let mut img = new_canvas(20, 20);
    let draw_heart = |img: &mut image::RgbaImage, ox: i32, oy: i32, small: bool, color: Rgba<u8>| {
        let pts: Vec<(i32, i32)> = if small {
            vec![
                (0, 0), (1, 0), (3, 0), (4, 0),
                (-1, 1), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1),
                (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),
                (1, 3), (2, 3), (3, 3), (2, 4),
            ]
        } else {
            vec![
                (1, 0), (2, 0), (4, 0), (5, 0),
                (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1),
                (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2),
                (1, 3), (2, 3), (3, 3), (4, 3), (5, 3),
                (2, 4), (3, 4), (4, 4), (3, 5),
            ]
        };
        let coords: Vec<(i32, i32)> = pts.iter().map(|&(x, y)| (x + ox, y + oy)).collect();
        px(img, &coords, color);
    };
    draw_heart(&mut img, 3, 12, true, Rgba([255, 100, 120, 255]));
    draw_heart(&mut img, 10, 3, false, Rgba([255, 75, 100, 255]));
    save(&img, "hearts.png", dir);

    // Rain cloud
    let mut img = new_canvas(24, 18);
    let mut cloud = Shape::new();
    for &(ccx, ccy, r) in &[(11.0, 3.0, 4.0), (8.0, 4.0, 3.0), (14.0, 4.0, 3.0), (11.0, 5.0, 5.0)]
    {
        cloud.extend(&ellipse_pixels(ccx, ccy, r, r * 0.6));
    }
    px_set(&mut img, &cloud, GRAY_CLOUD);
    flood_outline(&mut img, &cloud, OUTLINE);
    for &(rx, ry) in &[(8, 9), (11, 10), (14, 9), (9, 13), (13, 12)] {
        px(&mut img, &[(rx, ry), (rx, ry + 1), (rx, ry + 2)], TEAR);
    }
    save(&img, "rain_cloud.png", dir);

    // Dizzy stars
    let mut img = new_canvas(24, 24);
    for &(sx, sy) in &[(6, 5), (17, 7), (5, 15), (18, 17)] {
        let star = [
            (sx, sy - 2), (sx, sy - 1), (sx, sy), (sx, sy + 1), (sx, sy + 2),
            (sx - 2, sy), (sx - 1, sy), (sx + 1, sy), (sx + 2, sy),
        ];
        px(&mut img, &star, YELLOW);
        px(&mut img, &[(sx, sy)], WHITE);
    }
    save(&img, "stars_dizzy.png", dir);

    // Sparkle
    let mut img = new_canvas(24, 24);
    let sparkle_col = Rgba([255, 255, 200, 255]);
    for &(sx, sy) in &[(5, 4), (18, 6), (4, 18), (20, 17), (12, 2), (12, 21)] {
        let spark = [
            (sx, sy - 2), (sx, sy - 1), (sx, sy), (sx, sy + 1), (sx, sy + 2),
            (sx - 2, sy), (sx - 1, sy), (sx + 1, sy), (sx + 2, sy),
        ];
        px(&mut img, &spark, sparkle_col);
        px(&mut img, &[(sx, sy)], WHITE);
    }
    save(&img, "sparkle.png", dir);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let dir = out_dir();
    let edir = effects_dir();
    println!("Generating Moluun sprites → {}\n", dir.display());

    println!("[Body]");
    gen_body(&dir);

    println!("\n[Ears]");
    gen_ears(&dir);

    println!("\n[Eyes — all moods]");
    gen_eyes(&dir);

    println!("\n[Mouths — all moods]");
    gen_mouths(&dir);

    println!("\n[Shared effects]");
    gen_effects(&edir);

    println!("\nDone! All sprites saved.");
}
