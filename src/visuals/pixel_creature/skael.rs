//! Skael pixel art — emerald cave reptile with horns, scales, dorsal ridge, and tail.
//!
//! Visual identity: deep green scales with lighter pattern, brown pointed horns,
//! golden reptile eyes, muscular tail with spines. Stoic and armored.

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

const SCALE_HIGHLIGHT: Rgba<u8> = Rgba([110, 175, 135, 255]); // lighter scale spots
const CLAW: Rgba<u8> = Rgba([60, 50, 40, 255]); // dark brown claws
const HORN_TIP: Rgba<u8> = Rgba([130, 100, 70, 255]); // lighter horn tips

// --- Egg ---

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Crystalline green egg — slightly angular feel
    fill_ellipse(img, cx, cy, 12, 16, p.egg);
    // Crystal vein lines — branching pattern
    for i in 0..6 {
        let y = cy - 10 + i * 4;
        put(img, cx - 4 + i, y, p.egg_spot);
        put(img, cx - 3 + i, y, p.egg_spot);
        put(img, cx + 3 - i, y + 1, p.egg_spot);
    }
    // Mineral sparkle dots
    put(img, cx - 5, cy - 4, SCALE_HIGHLIGHT);
    put(img, cx + 4, cy + 3, SCALE_HIGHLIGHT);
    put(img, cx - 1, cy + 8, SCALE_HIGHLIGHT);
}

// --- Cub ---

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 17;

    // Small horns — nubs with lighter tips
    fill_rect(img, cx - 8, by - br + 1, 4, 3, p.accent);
    fill_rect(img, cx - 7, by - br - 1, 2, 2, p.accent);
    put(img, cx - 7, by - br - 2, HORN_TIP);
    fill_rect(img, cx + 5, by - br + 1, 4, 3, p.accent);
    fill_rect(img, cx + 6, by - br - 1, 2, 2, p.accent);
    put(img, cx + 7, by - br - 2, HORN_TIP);

    // Body
    fill_circle(img, cx, by, br, p.body);

    // Scale pattern — scattered lighter pixels
    for &(dx, dy) in &[(-7,-5), (6,-3), (-4,3), (8,1), (-2,-9), (5,6), (-9,2)] {
        put(img, cx + dx, by + dy, SCALE_HIGHLIGHT);
    }

    // Belly — lighter underbelly with scale rows
    fill_circle(img, cx, by + 5, 11, p.body_light);
    for &(dx, dy) in &[(-4, 3), (0, 5), (4, 3), (-2, 7), (2, 7)] {
        put(img, cx + dx, by + 5 + dy, fade(p.body_light, 0.15));
    }

    // Short tapered tail with tiny spine
    fill_rect(img, cx - 2, by + br - 2, 5, 3, p.accent);
    fill_rect(img, cx - 1, by + br + 1, 3, 2, p.accent);
    put(img, cx, by + br + 3, p.accent);
    put(img, cx, by + br - 3, HORN_TIP); // tiny dorsal nub

    // Stub feet with claws
    fill_rect(img, cx - 8, by + br - 3, 6, 5, p.body);
    fill_rect(img, cx + 3, by + br - 3, 6, 5, p.body);
    for dx in [-1, 1, 3] {
        put(img, cx - 8 + dx, by + br + 2, CLAW);
        put(img, cx + 3 + dx, by + br + 2, CLAW);
    }

    // HUGE golden eyes
    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);
    // Reptile pupil slit (vertical dark line in center of each eye)
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 2, Rgba([30, 25, 10, 255]));
        put(img, cx - 5, by + 3, Rgba([30, 25, 10, 255]));
        put(img, cx + 4, by + 2, Rgba([30, 25, 10, 255]));
        put(img, cx + 4, by + 3, Rgba([30, 25, 10, 255]));
    }

    // Snout
    if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 2, by + 9, 4, 2, p.mouth);
    }
}

// --- Young ---

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 15;

    // Growing horns — taller, with gradient
    fill_rect(img, cx - 9, by - br, 4, 4, p.accent);
    fill_rect(img, cx - 8, by - br - 2, 2, 2, p.accent);
    put(img, cx - 7, by - br - 4, HORN_TIP);
    put(img, cx - 8, by - br - 3, HORN_TIP);
    fill_rect(img, cx + 6, by - br, 4, 4, p.accent);
    fill_rect(img, cx + 7, by - br - 2, 2, 2, p.accent);
    put(img, cx + 8, by - br - 4, HORN_TIP);
    put(img, cx + 7, by - br - 3, HORN_TIP);

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-6,-4), (5,-2), (-3,3), (7,1), (-8,0), (4,-6)] {
        put(img, cx + dx, by + dy, SCALE_HIGHLIGHT);
    }
    fill_circle(img, cx, by + 4, 9, p.body_light);

    // Dorsal ridge emerging (3 small bumps along spine)
    for i in 0..3 {
        put(img, cx, by - br + 3 + i * 3, HORN_TIP);
    }

    // Tapered tail — longer, with spine nubs
    fill_rect(img, cx - 2, by + br - 2, 6, 3, p.accent);
    fill_rect(img, cx - 1, by + br + 1, 4, 3, p.accent);
    fill_rect(img, cx, by + br + 4, 2, 2, p.accent);
    put(img, cx, by + br + 6, p.accent);
    // Tail spines
    put(img, cx - 1, by + br - 1, HORN_TIP);
    put(img, cx, by + br + 2, HORN_TIP);

    // Feet with claws
    fill_rect(img, cx - 8, by + br - 3, 6, 6, p.body);
    fill_rect(img, cx + 3, by + br - 3, 6, 6, p.body);
    for dx in [-1, 1, 3] {
        put(img, cx - 8 + dx, by + br + 3, CLAW);
        put(img, cx + 3 + dx, by + br + 3, CLAW);
    }

    // Eyes — golden with slit pupil
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 2, Rgba([30, 25, 10, 255]));
        put(img, cx + 3, by + 2, Rgba([30, 25, 10, 255]));
    }

    // Snout — wider
    fill_rect(img, cx - 2, by + 8, 5, 3, p.mouth);
    // Nostrils
    put(img, cx - 1, by + 8, fade(p.mouth, 0.3));
    put(img, cx + 2, by + 8, fade(p.mouth, 0.3));
}

// --- Adult ---

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Full horns — tall, layered, with color gradient
    fill_rect(img, cx - 10, by - br - 1, 5, 5, p.accent);
    fill_rect(img, cx - 9,  by - br - 4, 3, 3, p.accent);
    fill_rect(img, cx - 8,  by - br - 6, 2, 2, HORN_TIP);
    put(img, cx - 8,  by - br - 8, HORN_TIP);
    fill_rect(img, cx + 6,  by - br - 1, 5, 5, p.accent);
    fill_rect(img, cx + 7,  by - br - 4, 3, 3, p.accent);
    fill_rect(img, cx + 8,  by - br - 6, 2, 2, HORN_TIP);
    put(img, cx + 8,  by - br - 8, HORN_TIP);

    // Body — slightly larger
    fill_circle(img, cx, by, br, p.body);

    // Scale pattern — denser for adult
    for &(dx, dy) in &[(-7,-6), (6,-4), (-4,3), (8,1), (-2,-10), (5,6), (-9,3), (3,-7), (-6,8), (7,-1)] {
        put(img, cx + dx, by + dy, SCALE_HIGHLIGHT);
    }

    // Belly with scale rows
    fill_circle(img, cx, by + 4, 10, p.body_light);
    for row in 0..3 {
        for col in -2..=2 {
            put(img, cx + col * 3, by + 2 + row * 3, fade(p.body_light, 0.12));
        }
    }

    // Dorsal ridge — prominent (5 spines along back)
    for i in 0..5 {
        let y = by - br + 2 + i * 3;
        put(img, cx, y, HORN_TIP);
        put(img, cx, y - 1, fade(HORN_TIP, 0.3));
    }

    // Full tapered tail — thick with spines
    fill_rect(img, cx - 3, by + br - 3, 7, 4, p.accent);
    fill_rect(img, cx - 2, by + br + 1, 5, 3, p.accent);
    fill_rect(img, cx - 1, by + br + 4, 3, 3, p.accent);
    fill_rect(img, cx, by + br + 7, 2, 2, p.accent);
    put(img, cx, by + br + 9, p.accent);
    // Tail spines
    put(img, cx - 2, by + br - 2, HORN_TIP);
    put(img, cx - 1, by + br + 2, HORN_TIP);
    put(img, cx, by + br + 5, HORN_TIP);

    // Sturdy feet with prominent claws
    fill_rect(img, cx - 10, by + br - 4, 7, 7, p.body);
    fill_rect(img, cx + 4, by + br - 4, 7, 7, p.body);
    for dx in [-2, 0, 2, 4] {
        put(img, cx - 10 + dx, by + br + 3, CLAW);
        put(img, cx + 4 + dx, by + br + 3, CLAW);
    }

    // Arms / front legs
    fill_rect(img, cx - br + 1, by, 3, 6, p.body);
    fill_rect(img, cx + br - 3, by, 3, 6, p.body);

    // Golden eyes with slit pupil
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 2, Rgba([30, 25, 10, 255]));
        put(img, cx + 3, by + 2, Rgba([30, 25, 10, 255]));
    }

    // Snout — wide, with nostrils
    fill_rect(img, cx - 3, by + 9, 6, 3, p.mouth);
    put(img, cx - 2, by + 9, fade(p.mouth, 0.3));
    put(img, cx + 3, by + 9, fade(p.mouth, 0.3));

    // Mood: threat display when hungry (open jaw)
    if *mood == MoodState::Hungry {
        fill_rect(img, cx - 2, by + 12, 4, 2, fade(p.mouth, 0.5));
    }
}

// --- Elder overlay ---

pub fn draw_elder_details(img: &mut RgbaImage, _p: &Palette, cx: i32) {
    let by = 24;
    let white = Rgba([200, 195, 180, 255]);

    // Worn/chipped horn tips
    put(img, cx - 8, by - 24, white);
    put(img, cx + 8, by - 24, white);

    // Scarred scale marks (lighter patches)
    put(img, cx - 6, 20, white);
    put(img, cx + 5, 22, white);
    put(img, cx - 4, 28, white);
    put(img, cx + 3, 30, white);
    put(img, cx - 8, 26, white);
    put(img, cx + 7, 24, white);

    // Faded dorsal ridge
    for i in 0..5 {
        put(img, cx, by - 14 + i * 3, white);
    }
}
