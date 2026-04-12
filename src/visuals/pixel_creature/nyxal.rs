//! Nyxal pixel art — deep-sea cephalopod with tentacles and bioluminescence.
//!
//! Visual identity: dark purple body, mantle dome, 4 tentacles with glowing tips,
//! bioluminescent cyan eyes, pulsing inner glow. Alien and beautiful.

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, put, draw_eyes, fade};

const GLOW_BRIGHT: Rgba<u8> = Rgba([80, 220, 240, 200]); // bright cyan glow
const GLOW_DIM: Rgba<u8> = Rgba([50, 140, 160, 150]);     // softer glow
const SPOT: Rgba<u8> = Rgba([100, 60, 140, 255]);          // body pattern spots

// --- Egg ---

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Translucent gelatinous sphere — rounder than other eggs
    fill_circle(img, cx, cy, 14, p.egg);

    // Inner membrane layers (translucent effect)
    fill_circle(img, cx, cy, 10, fade(p.egg, 0.15));
    fill_circle(img, cx, cy, 6, p.body_light);

    // Bioluminescent spots (pulsing glow impression)
    fill_circle(img, cx - 4, cy - 4, 2, p.egg_spot);
    fill_circle(img, cx + 5, cy + 2, 2, p.egg_spot);
    fill_circle(img, cx - 1, cy + 6, 2, p.egg_spot);
    fill_circle(img, cx + 3, cy - 6, 1, p.egg_spot);

    // Nucleus hint (the forming creature inside)
    fill_circle(img, cx, cy, 3, p.body);
    put(img, cx, cy - 1, p.eye); // tiny cyan eye dot
}

// --- Cub ---

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 20;
    let br = 16;

    // Small mantle bump with glow rim
    fill_circle(img, cx, by - 7, 7, p.accent);
    // Glow rim on mantle edge
    for dx in -5..=5 {
        put(img, cx + dx, by - 13, GLOW_DIM);
    }

    // 4 tentacle nubs — with curl at tip
    fill_rect(img, cx - 12, by + br - 3, 3, 10, p.accent);
    fill_rect(img, cx - 5,  by + br - 2, 3, 12, p.accent);
    fill_rect(img, cx + 3,  by + br - 2, 3, 12, p.accent);
    fill_rect(img, cx + 10, by + br - 3, 3, 10, p.accent);
    // Tiny glow dots at tentacle tips
    put(img, cx - 11, by + br + 6, GLOW_DIM);
    put(img, cx - 4,  by + br + 9, GLOW_DIM);
    put(img, cx + 4,  by + br + 9, GLOW_DIM);
    put(img, cx + 11, by + br + 6, GLOW_DIM);

    // Body (on top of tentacle bases)
    fill_circle(img, cx, by, br, p.body);

    // Body pattern spots (bioluminescent freckles)
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0)] {
        put(img, cx + dx, by + dy, SPOT);
    }

    // Inner belly glow (faint lighter area)
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // HUGE cyan eyes
    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);
    // Eye glow aura (faint ring)
    if *mood != MoodState::Sleeping {
        put(img, cx - 9, by + 3, GLOW_DIM);
        put(img, cx + 8, by + 3, GLOW_DIM);
    }
}

// --- Young ---

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 20;
    let br = 13;

    // Growing mantle with glow rim
    fill_circle(img, cx, by - 5, 9, p.accent);
    for dx in -6..=6 {
        put(img, cx + dx, by - 13, GLOW_DIM);
    }
    // Mantle pattern
    put(img, cx - 3, by - 8, SPOT);
    put(img, cx + 2, by - 6, SPOT);

    // 4 tentacles — longer, with curl and glow tips
    fill_rect(img, cx - 12, by + br - 3, 3, 15, p.accent);
    fill_rect(img, cx - 5,  by + br - 2, 3, 17, p.accent);
    fill_rect(img, cx + 3,  by + br - 2, 3, 17, p.accent);
    fill_rect(img, cx + 10, by + br - 3, 3, 15, p.accent);
    // Curl at outer tentacle tips
    put(img, cx - 13, by + br + 11, p.accent);
    put(img, cx + 12, by + br + 11, p.accent);
    // Glow tips
    fill_rect(img, cx - 12, by + br + 10, 3, 2, GLOW_DIM);
    fill_rect(img, cx - 5,  by + br + 13, 3, 2, GLOW_DIM);
    fill_rect(img, cx + 3,  by + br + 13, 3, 2, GLOW_DIM);
    fill_rect(img, cx + 10, by + br + 10, 3, 2, GLOW_DIM);

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-4,-3), (3,-1), (-1,2), (5,0), (-6,1)] {
        put(img, cx + dx, by + dy, SPOT);
    }
    fill_circle(img, cx, by + 3, 8, p.body_light);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 8, by + 3, GLOW_DIM);
        put(img, cx + 7, by + 3, GLOW_DIM);
    }
}

// --- Adult ---

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 18;
    let br = 14;

    // Full mantle dome with glow rim and pattern
    fill_circle(img, cx, by - 6, 11, p.accent);
    for dx in -8..=8 {
        put(img, cx + dx, by - 16, GLOW_BRIGHT);
    }
    // Mantle bioluminescent spots
    put(img, cx - 4, by - 10, GLOW_DIM);
    put(img, cx + 3, by - 8, GLOW_DIM);
    put(img, cx - 1, by - 12, GLOW_DIM);

    // 4 full tentacles — graceful, with curled tips and bright glow
    fill_rect(img, cx - 14, by + br - 4, 4, 19, p.accent);
    fill_rect(img, cx - 5,  by + br - 3, 4, 23, p.accent);
    fill_rect(img, cx + 2,  by + br - 3, 4, 23, p.accent);
    fill_rect(img, cx + 11, by + br - 4, 4, 19, p.accent);
    // Curled outer tips
    put(img, cx - 15, by + br + 13, p.accent);
    put(img, cx - 15, by + br + 14, p.accent);
    put(img, cx + 14, by + br + 13, p.accent);
    put(img, cx + 14, by + br + 14, p.accent);
    // Bright glow tips
    fill_rect(img, cx - 14, by + br + 13, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx - 5,  by + br + 18, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx + 2,  by + br + 18, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx + 11, by + br + 13, 4, 3, GLOW_BRIGHT);
    // Sucker dots along inner tentacles
    for i in 0..4 {
        put(img, cx - 4, by + br + 2 + i * 4, SPOT);
        put(img, cx + 4, by + br + 2 + i * 4, SPOT);
    }

    // Body
    fill_circle(img, cx, by, br, p.body);
    // Bioluminescent body pattern (more dense)
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6), (-4,5), (7,-3)] {
        put(img, cx + dx, by + dy, SPOT);
    }
    fill_circle(img, cx, by + 3, 9, p.body_light);

    // Side fin/membrane (thin, translucent)
    for i in 0..4 {
        put(img, cx - br - 1, by - 2 + i * 2, fade(p.accent, 0.3));
        put(img, cx + br + 1, by - 2 + i * 2, fade(p.accent, 0.3));
    }

    // Eyes — large, luminous cyan
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    // Eye glow aura
    if *mood != MoodState::Sleeping {
        put(img, cx - 9, by + 2, GLOW_DIM);
        put(img, cx - 9, by + 3, GLOW_DIM);
        put(img, cx + 8, by + 2, GLOW_DIM);
        put(img, cx + 8, by + 3, GLOW_DIM);
    }

    // Mood: pulsing glow when happy
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        // Extra glow spots on body
        put(img, cx - 3, by - 2, GLOW_BRIGHT);
        put(img, cx + 2, by + 1, GLOW_BRIGHT);
        put(img, cx, by + 5, GLOW_BRIGHT);
    }
}

// --- Elder overlay ---

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let by = 18;
    // Dimmer glow tips (aging bioluminescence)
    let dim = Rgba([140, 130, 115, 200]);
    fill_rect(img, cx - 14, by + 23, 4, 3, dim);
    fill_rect(img, cx - 5,  by + 28, 4, 3, dim);
    fill_rect(img, cx + 2,  by + 28, 4, 3, dim);
    fill_rect(img, cx + 11, by + 23, 4, 3, dim);
    // Faded mantle glow
    for dx in -6..=6 {
        put(img, cx + dx, by - 16, dim);
    }
    // Wisdom markings — concentric rings on mantle
    put(img, cx - 2, by - 9, Rgba([120, 100, 150, 255]));
    put(img, cx + 1, by - 7, Rgba([120, 100, 150, 255]));
    put(img, cx, by - 11, Rgba([120, 100, 150, 255]));
}
