//! Pylum skin — golden highland bird (secretary bird / cassowary).
//!
//! Structural evolution:
//! - Cub:   fluffy down ball, tiny beak, short legs, no wings/crest
//! - Young: LEGS grow dramatically, body stays small on top, gangly, wing nubs
//! - Adult: tall imposing form, casque crown, full wings, powerful taloned legs
//! - Elder: white crest tip, faded wing edges

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade, NEAR_BLACK_PX};

const HIGHLIGHT: Rgba<u8> = Rgba([255, 255, 255, 180]);
const CLAW: Rgba<u8> = Rgba([40, 30, 20, 255]);
const RESONANCE: Rgba<u8> = Rgba([230, 200, 120, 70]);
const RESONANCE_BRIGHT: Rgba<u8> = Rgba([240, 210, 130, 110]);

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    fill_ellipse(img, cx, cy, 11, 15, p.egg);
    fill_ellipse(img, cx, cy - 3, 9, 10, fade(p.egg, 0.05));
    for &(dx, dy) in &[(-3,-7), (4,-4), (-5,1), (2,4), (5,-8), (-1,7), (3,0), (-4,-3), (6,2)] {
        put(img, cx + dx, cy + dy, p.egg_spot);
        put(img, cx + dx + 1, cy + dy, p.egg_spot);
    }
}

// ===================================================================
// CUB — fluffy down ball
// ===================================================================

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 22;
    let br = 17;

    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-7,-6), (6,-4), (-4,4), (8,2), (-2,-10), (5,7), (-9,1), (3,-8)] {
        put(img, cx + dx, by + dy, p.body_light);
    }
    fill_circle(img, cx, by + 5, 11, p.body_light);

    // Tiny tuft
    fill_rect(img, cx - 1, by - br, 3, 3, p.body_light);

    // Short stubby legs
    fill_rect(img, cx - 5, by + br - 2, 4, 5, p.mouth);
    fill_rect(img, cx + 2, by + br - 2, 4, 5, p.mouth);

    fill_circle(img, cx, by + 3, 3, RESONANCE);

    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 6, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    fill_rect(img, cx - 1, by + 9, 3, 2, NEAR_BLACK_PX);
    put(img, cx, by + 11, NEAR_BLACK_PX);
}

// ===================================================================
// YOUNG — gangly: LEGS explode, body small on top
// ===================================================================

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 16;
    let br = 11;

    let leg_top = by + br - 2;
    let leg_len = 28;
    fill_rect(img, cx - 5, leg_top, 4, leg_len, p.mouth);
    fill_rect(img, cx + 2, leg_top, 4, leg_len, p.mouth);
    fill_rect(img, cx - 6, leg_top + 14, 6, 3, p.mouth);
    fill_rect(img, cx + 1, leg_top + 14, 6, 3, p.mouth);
    fill_rect(img, cx - 6, leg_top + leg_len - 1, 5, 3, p.mouth);
    fill_rect(img, cx + 2, leg_top + leg_len - 1, 5, 3, p.mouth);
    for dx in [-1, 1, 3] {
        put(img, cx - 6 + dx, leg_top + leg_len + 2, CLAW);
        put(img, cx + 2 + dx, leg_top + leg_len + 2, CLAW);
    }

    fill_rect(img, cx - br - 1, by - 1, 4, 7, p.accent);
    fill_rect(img, cx + br - 2, by - 1, 4, 7, p.accent);

    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-4,-3), (3,-2), (-2,3), (5,1)] {
        put(img, cx + dx, by + dy, p.body_light);
    }
    fill_circle(img, cx, by + 3, 7, p.body_light);

    fill_rect(img, cx - 1, by - br - 2, 3, 4, p.body);
    put(img, cx, by - br - 3, p.accent);
    put(img, cx + 1, by - br - 3, p.accent);

    fill_circle(img, cx, by + 2, 4, RESONANCE);

    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    fill_rect(img, cx - 2, by + 7, 5, 3, NEAR_BLACK_PX);
    fill_rect(img, cx - 1, by + 10, 3, 2, NEAR_BLACK_PX);
}

// ===================================================================
// ADULT — tall imposing: casque, wings, talons
// ===================================================================

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let hy = 10;
    let hr = 9;
    let body_y = 24;
    let body_r = 12;

    let leg_top = body_y + body_r - 2;
    let leg_len = 22;
    fill_rect(img, cx - 5, leg_top, 5, leg_len, p.mouth);
    fill_rect(img, cx + 1, leg_top, 5, leg_len, p.mouth);
    fill_rect(img, cx - 6, leg_top + 10, 7, 4, p.mouth);
    fill_rect(img, cx, leg_top + 10, 7, 4, p.mouth);
    fill_rect(img, cx - 7, leg_top + leg_len - 1, 6, 4, p.mouth);
    fill_rect(img, cx + 2, leg_top + leg_len - 1, 6, 4, p.mouth);
    for dx in [-2, 0, 2, 4] {
        put(img, cx - 7 + dx, leg_top + leg_len + 3, CLAW);
        put(img, cx + 2 + dx, leg_top + leg_len + 3, CLAW);
    }

    fill_rect(img, cx - body_r - 2, body_y - 5, 6, 16, p.accent);
    fill_rect(img, cx + body_r - 3, body_y - 5, 6, 16, p.accent);
    for i in 0..4 {
        put(img, cx - body_r - 3, body_y + 7 + i * 2, fade(p.accent, 0.3));
        put(img, cx + body_r + 2, body_y + 7 + i * 2, fade(p.accent, 0.3));
    }
    fill_rect(img, cx - body_r - 1, body_y - 2, 1, 10, p.body_light);
    fill_rect(img, cx + body_r, body_y - 2, 1, 10, p.body_light);

    fill_circle(img, cx, body_y, body_r, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6)] {
        put(img, cx + dx, body_y + dy, p.body_light);
    }
    fill_circle(img, cx, body_y + 3, 8, p.body_light);

    fill_rect(img, cx - 4, hy + hr - 1, 9, 6, p.body);
    fill_rect(img, cx - 3, hy + hr, 7, 4, p.accent);

    fill_circle(img, cx, hy, hr, p.body);

    // Casque crown
    fill_rect(img, cx - 2, hy - hr - 4, 5, 7, p.body);
    fill_rect(img, cx - 1, hy - hr - 6, 3, 3, p.accent);
    put(img, cx, hy - hr - 7, fade(p.accent, 0.3));
    put(img, cx - 3, hy - hr - 3, p.accent);
    put(img, cx + 3, hy - hr - 3, p.accent);
    put(img, cx - 4, hy - hr - 1, p.accent);
    put(img, cx + 4, hy - hr - 1, p.accent);

    fill_circle(img, cx, body_y + 2, 5, RESONANCE_BRIGHT);
    fill_circle(img, cx, body_y + 2, 3, RESONANCE);
    put(img, cx - body_r - 1, body_y, RESONANCE);
    put(img, cx + body_r, body_y, RESONANCE);

    draw_eyes(img, cx, hy + 1, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 4, hy + 1, HIGHLIGHT);
        put(img, cx + 2, hy + 1, HIGHLIGHT);
    }

    fill_rect(img, cx - 3, hy + 5, 7, 3, NEAR_BLACK_PX);
    fill_rect(img, cx - 2, hy + 8, 5, 2, NEAR_BLACK_PX);
    fill_rect(img, cx - 1, hy + 10, 3, 2, NEAR_BLACK_PX);

    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 5, body_y + 1, p.body_light);
        put(img, cx + 5, body_y + 1, p.body_light);
    }
}

// ===================================================================
// ELDER overlay
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, _p: &Palette, cx: i32) {
    let hy = 10;
    let body_y = 24;
    let white = Rgba([230, 225, 210, 255]);
    let dim = Rgba([200, 180, 120, 50]);

    put(img, cx, hy - 16, white);
    put(img, cx, hy - 15, white);
    for i in 0..3 {
        put(img, cx - 15, body_y + 4 + i * 2, white);
        put(img, cx + 15, body_y + 4 + i * 2, white);
    }
    put(img, cx - 5, hy - 3, white);
    put(img, cx + 4, hy - 2, white);
    fill_circle(img, cx, body_y + 2, 4, dim);
}
