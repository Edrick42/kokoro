//! Pylum skin — golden highland bird (secretary bird / cassowary).
//!
//! Structural evolution:
//! - Cub:   fluffy down ball, tiny beak, short legs, no wings/crest
//! - Young: LEGS grow dramatically, body stays small on top, gangly, wing nubs
//! - Adult: tall imposing form, casque crown, full wings, powerful taloned legs
//! - Elder: white crest tip, faded wing edges

use image::{RgbaImage, Rgba};
use bevy::prelude::Res;
use crate::creature::interaction::soft_body::SoftBody;
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

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let (_, by) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 22));
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

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
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

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 10));
    let hr = 9;
    let (_, body_y) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 24));
    let body_r = 12;
    let (wl_x, wl_y) = sb.as_ref().map(|b| b.point("wing_l").px()).unwrap_or((cx - 14, 20));
    let (wr_x, wr_y) = sb.as_ref().map(|b| b.point("wing_r").px()).unwrap_or((cx + 14, 20));
    let (wtl_x, wtl_y) = sb.as_ref().map(|b| b.point("wingtip_l").px()).unwrap_or((cx - 20, 26));
    let (wtr_x, wtr_y) = sb.as_ref().map(|b| b.point("wingtip_r").px()).unwrap_or((cx + 20, 26));
    let (_, _tail_y) = sb.as_ref().map(|b| b.point("tail").px()).unwrap_or((cx, 34));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 5, 54));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 5, 54));

    // Legs — from body down to soft body foot positions
    let leg_top = body_y + body_r - 2;
    let ll_h = (fl_y - leg_top).max(4);
    let lr_h = (fr_y - leg_top).max(4);
    fill_rect(img, fl_x - 2, leg_top, 5, ll_h, p.mouth);
    fill_rect(img, fr_x - 2, leg_top, 5, lr_h, p.mouth);
    // Knee joints
    fill_rect(img, fl_x - 3, leg_top + ll_h / 2, 7, 4, p.mouth);
    fill_rect(img, fr_x - 3, leg_top + lr_h / 2, 7, 4, p.mouth);
    // Feet
    fill_rect(img, fl_x - 4, fl_y - 1, 6, 4, p.mouth);
    fill_rect(img, fr_x - 3, fr_y - 1, 6, 4, p.mouth);
    // Claws
    for dx in [-2, 0, 2, 4] {
        put(img, fl_x - 4 + dx, fl_y + 3, CLAW);
        put(img, fr_x - 3 + dx, fr_y + 3, CLAW);
    }

    // Wings — from soft body wing positions to wingtip positions
    let wing_h = ((wtl_y - wl_y).abs().max(2)) as i32;
    fill_rect(img, wl_x, wl_y, (cx - wl_x).max(1), wing_h.min(16), p.accent);
    fill_rect(img, cx, wr_y, (wr_x - cx).max(1), wing_h.min(16), p.accent);
    // Wing tips (feathered edges)
    fill_rect(img, wtl_x, wtl_y, (wl_x - wtl_x).max(1), (wing_h / 2).max(3), p.accent);
    fill_rect(img, wr_x, wtr_y, (wtr_x - wr_x).max(1), (wing_h / 2).max(3), p.accent);
    for i in 0..4 {
        put(img, wtl_x, wtl_y + i * 2, fade(p.accent, 0.3));
        put(img, wtr_x, wtr_y + i * 2, fade(p.accent, 0.3));
    }
    // Wing inner edge highlights
    fill_rect(img, wl_x + 1, wl_y, 1, wing_h.min(10), p.body_light);
    fill_rect(img, wr_x - 1, wr_y, 1, wing_h.min(10), p.body_light);

    fill_circle(img, cx, body_y, body_r, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6)] {
        put(img, cx + dx, body_y + dy, p.body_light);
    }
    fill_circle(img, cx, body_y + 3, 8, p.body_light);

    // Neck (from head to body)
    let neck_cx = (hx + cx) / 2;
    let neck_top = hy.min(body_y);
    let neck_h = (body_y - hy).max(1);
    fill_rect(img, neck_cx - 4, neck_top, 9, neck_h, p.body);
    fill_rect(img, neck_cx - 3, neck_top + 1, 7, neck_h.max(1) - 1, p.accent);

    // Head (on top of neck)
    fill_circle(img, hx, hy, hr, p.body);

    // Casque crown (moves with head)
    fill_rect(img, hx - 2, hy - hr - 4, 5, 7, p.body);
    fill_rect(img, hx - 1, hy - hr - 6, 3, 3, p.accent);
    put(img, hx, hy - hr - 7, fade(p.accent, 0.3));
    put(img, hx - 3, hy - hr - 3, p.accent);
    put(img, hx + 3, hy - hr - 3, p.accent);
    put(img, hx - 4, hy - hr - 1, p.accent);
    put(img, hx + 4, hy - hr - 1, p.accent);

    // Kokoro-sac
    fill_circle(img, cx, body_y + 2, 5, RESONANCE_BRIGHT);
    fill_circle(img, cx, body_y + 2, 3, RESONANCE);
    put(img, cx - body_r - 1, body_y, RESONANCE);
    put(img, cx + body_r, body_y, RESONANCE);

    // Eyes (on head)
    draw_eyes(img, hx, hy + 1, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 4, hy + 1, HIGHLIGHT);
        put(img, hx + 2, hy + 1, HIGHLIGHT);
    }

    // Beak (on head)
    fill_rect(img, hx - 3, hy + 5, 7, 3, NEAR_BLACK_PX);
    fill_rect(img, hx - 2, hy + 8, 5, 2, NEAR_BLACK_PX);
    fill_rect(img, hx - 1, hy + 10, 3, 2, NEAR_BLACK_PX);

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
