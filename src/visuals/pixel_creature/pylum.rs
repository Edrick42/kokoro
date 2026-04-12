//! Pylum pixel art — golden highland bird with wings, beak, crest, and talons.
//!
//! Visual identity: warm amber plumage, prominent dark beak, feathered tuft/crest,
//! wing stubs that grow into full wings. Beady dark eyes with tiny highlight.

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade, NEAR_BLACK_PX};

const HIGHLIGHT: Rgba<u8> = Rgba([255, 255, 255, 180]);
const BEAK_HIGHLIGHT: Rgba<u8> = Rgba([50, 40, 30, 255]); // dark beak shine line

// --- Egg ---

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Speckled bird egg — slightly pointed at top
    fill_ellipse(img, cx, cy, 11, 15, p.egg);
    fill_ellipse(img, cx, cy - 3, 9, 10, fade(p.egg, 0.05)); // subtle lighter top

    // Brown speckles scattered organically
    for &(dx, dy) in &[(-3,-7), (4,-4), (-5,1), (2,4), (5,-8), (-1,7), (3,0), (-4,-3), (6,2), (-2,5)] {
        put(img, cx + dx, cy + dy, p.egg_spot);
        put(img, cx + dx + 1, cy + dy, p.egg_spot);
    }
    // Cluster of speckles near bottom
    fill_rect(img, cx - 2, cy + 9, 2, 1, p.egg_spot);
    fill_rect(img, cx + 1, cy + 10, 2, 1, p.egg_spot);
}

// --- Cub ---

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 17;

    // Tiny wing nubs (fluffy stubs)
    fill_rect(img, cx - br + 2, by - 2, 4, 7, p.accent);
    fill_rect(img, cx + br - 5, by - 2, 4, 7, p.accent);
    // Feather tips on nubs
    put(img, cx - br + 2, by + 4, fade(p.accent, 0.2));
    put(img, cx + br - 2, by + 4, fade(p.accent, 0.2));

    // Body — round, fluffy
    fill_circle(img, cx, by, br, p.body);

    // Plumage texture — lighter feather highlights
    for &(dx, dy) in &[(-6,-5), (5,-3), (-3,4), (7,2), (-8,1), (4,-7)] {
        put(img, cx + dx, by + dy, p.body_light);
    }

    // Belly
    fill_circle(img, cx, by + 5, 11, p.body_light);

    // Fluffy tuft on top (baby down)
    fill_rect(img, cx - 1, by - br - 1, 3, 4, p.body);
    put(img, cx, by - br - 2, p.accent);

    // Stub feet with tiny talons
    fill_rect(img, cx - 6, by + br - 3, 5, 4, p.accent);
    fill_rect(img, cx + 2, by + br - 3, 5, 4, p.accent);
    // Talon dots
    for dx in [-1, 1, 3] {
        put(img, cx - 6 + dx, by + br + 1, NEAR_BLACK_PX);
        put(img, cx + 2 + dx, by + br + 1, NEAR_BLACK_PX);
    }

    // HUGE eyes with highlight
    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 6, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    // Beak — dark, triangular, always visible
    fill_rect(img, cx - 2, by + 9, 5, 3, NEAR_BLACK_PX);
    fill_rect(img, cx - 1, by + 12, 3, 2, NEAR_BLACK_PX);
    // Beak shine
    put(img, cx - 1, by + 9, BEAK_HIGHLIGHT);
}

// --- Young ---

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 15;

    // Growing wings — longer, feathered edges
    fill_rect(img, cx - br - 1, by - 2, 5, 9, p.accent);
    fill_rect(img, cx + br - 3, by - 2, 5, 9, p.accent);
    // Feather tip detail
    for i in 0..3 {
        put(img, cx - br - 1, by + 5 + i, fade(p.accent, 0.3));
        put(img, cx + br + 1, by + 5 + i, fade(p.accent, 0.3));
    }

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0)] {
        put(img, cx + dx, by + dy, p.body_light);
    }
    fill_circle(img, cx, by + 4, 9, p.body_light);

    // Tuft growing into crest
    fill_rect(img, cx - 1, by - br - 2, 3, 4, p.body);
    fill_rect(img, cx, by - br - 3, 1, 2, p.accent);

    // Feet with talons
    fill_rect(img, cx - 6, by + br - 3, 5, 5, p.mouth);
    fill_rect(img, cx + 2, by + br - 3, 5, 5, p.mouth);
    for dx in [-1, 1, 3] {
        put(img, cx - 6 + dx, by + br + 2, NEAR_BLACK_PX);
        put(img, cx + 2 + dx, by + br + 2, NEAR_BLACK_PX);
    }

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    // Beak — growing, still dark
    fill_rect(img, cx - 3, by + 8, 6, 3, NEAR_BLACK_PX);
    fill_rect(img, cx - 2, by + 11, 4, 2, NEAR_BLACK_PX);
    fill_rect(img, cx - 1, by + 13, 2, 1, NEAR_BLACK_PX);
    put(img, cx - 2, by + 8, BEAK_HIGHLIGHT);
}

// --- Adult ---

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Full wings — layered feathers
    fill_rect(img, cx - br, by - 4, 6, 14, p.accent);
    fill_rect(img, cx + br - 5, by - 4, 6, 14, p.accent);
    // Primary feather tips (darker, longer)
    for i in 0..4 {
        let y = by + 6 + i * 2;
        put(img, cx - br - 1, y, fade(p.accent, 0.3));
        put(img, cx + br, y, fade(p.accent, 0.3));
    }
    // Wing highlight stripe
    fill_rect(img, cx - br + 1, by - 1, 1, 8, p.body_light);
    fill_rect(img, cx + br - 1, by - 1, 1, 8, p.body_light);

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-6,-5), (5,-3), (-3,4), (7,2), (-8,1), (4,-7), (-1,6)] {
        put(img, cx + dx, by + dy, p.body_light);
    }
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // Tall crest — signature adult feature
    fill_rect(img, cx - 1, by - br - 3, 3, 6, p.body);
    fill_rect(img, cx, by - br - 5, 1, 3, p.accent);
    put(img, cx, by - br - 6, fade(p.body, 0.2));

    // Sturdy feet with talons
    fill_rect(img, cx - 7, by + br - 3, 6, 6, p.mouth);
    fill_rect(img, cx + 2, by + br - 3, 6, 6, p.mouth);
    for dx in [-2, 0, 2, 4] {
        put(img, cx - 7 + dx, by + br + 3, NEAR_BLACK_PX);
        put(img, cx + 2 + dx, by + br + 3, NEAR_BLACK_PX);
    }

    // Eyes — confident
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    // Full beak — layered, prominent
    fill_rect(img, cx - 3, by + 8, 7, 3, NEAR_BLACK_PX);
    fill_rect(img, cx - 2, by + 11, 5, 2, NEAR_BLACK_PX);
    fill_rect(img, cx - 1, by + 13, 3, 2, NEAR_BLACK_PX);
    put(img, cx, by + 15, NEAR_BLACK_PX);
    // Beak shine line
    put(img, cx - 2, by + 8, BEAK_HIGHLIGHT);
    put(img, cx - 1, by + 8, BEAK_HIGHLIGHT);

    // Mood expression: fluffed up when happy
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        // Puffed chest feathers
        put(img, cx - 5, by + 2, p.body_light);
        put(img, cx + 5, by + 2, p.body_light);
        put(img, cx - 4, by + 4, p.body_light);
        put(img, cx + 4, by + 4, p.body_light);
    }
}

// --- Elder overlay ---

pub fn draw_elder_details(img: &mut RgbaImage, _p: &Palette, cx: i32) {
    let by = 24;
    let white = Rgba([230, 225, 210, 255]);
    // Faded feather tips at wing edges
    for i in 0..3 {
        put(img, cx - 17, by + 4 + i * 2, white);
        put(img, cx + 17, by + 4 + i * 2, white);
    }
    // White crest tip
    put(img, cx, by - 22, white);
    put(img, cx, by - 21, white);
    // Lighter plumage patches (aging)
    put(img, cx - 6, by - 4, white);
    put(img, cx + 5, by - 3, white);
}
