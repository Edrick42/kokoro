//! Nyxal skin — deep-sea cephalopod (cuttlefish / nautilus).
//!
//! Structural evolution:
//! - Cub:   almost ALL mantle dome, tiny tentacle stubs, translucent, minimal glow
//! - Young: tentacles growing FAST (allometric), dome shrinking proportionally, glow appearing
//! - Adult: tentacles dominate, dome proportionally small, rich bioluminescence, side fins
//! - Elder: dimmer glow, wisdom rings on mantle, thinner tentacles

use image::{RgbaImage, Rgba};
use bevy::prelude::Res;
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, put, draw_eyes, fade};

const GLOW_BRIGHT: Rgba<u8> = Rgba([80, 220, 240, 200]);
const GLOW_DIM: Rgba<u8> = Rgba([50, 140, 160, 150]);
const GLOW_FAINT: Rgba<u8> = Rgba([40, 100, 120, 80]);
const SPOT: Rgba<u8> = Rgba([100, 60, 140, 255]);

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    fill_circle(img, cx, cy, 14, p.egg);
    fill_circle(img, cx, cy, 10, fade(p.egg, 0.15));
    fill_circle(img, cx, cy, 6, p.body_light);
    fill_circle(img, cx - 4, cy - 4, 2, p.egg_spot);
    fill_circle(img, cx + 5, cy + 2, 2, p.egg_spot);
    fill_circle(img, cx - 1, cy + 6, 2, p.egg_spot);
    fill_circle(img, cx, cy, 3, p.body);
    put(img, cx, cy - 1, p.eye);
}

// ===================================================================
// CUB — almost all dome, tiny stubs, translucent
// ===================================================================
// Planktonic larva: huge mantle dome with tiny tentacle nubs underneath.
// Nearly transparent. Almost no glow. Alien blob.

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (mx, my) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 16));
    let (mtx, mty) = sb.as_ref().map(|b| b.point("mantle_top").px()).unwrap_or((cx, 4));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("tent_fl").px()).unwrap_or((cx - 3, 27));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("tent_fr").px()).unwrap_or((cx + 3, 27));
    let (bl_x, bl_y) = sb.as_ref().map(|b| b.point("tent_bl").px()).unwrap_or((cx - 7, 26));
    let (br_x, br_y) = sb.as_ref().map(|b| b.point("tent_br").px()).unwrap_or((cx + 7, 26));
    let (ex_l, ey_l) = sb.as_ref().map(|b| b.point("eye_l").px()).unwrap_or((cx - 5, 17));
    let mr = 16;

    // Tiny tentacle stubs (move with soft body)
    fill_rect(img, bl_x - 1, bl_y, 3, 6, p.accent);
    fill_rect(img, fl_x - 1, fl_y, 3, 8, p.accent);
    fill_rect(img, fr_x - 1, fr_y, 3, 8, p.accent);
    fill_rect(img, br_x - 1, br_y, 3, 6, p.accent);

    // Big dome (mantle is "body" anchor — moves with breathing pulse)
    fill_circle(img, mx, my, mr, p.body);
    // Mantle cap on top (moves with mantle_top)
    fill_circle(img, mtx, mty + 4, 8, p.accent);

    // Inner lighter area (translucent — can see through)
    fill_circle(img, mx, my + 2, 10, p.body_light);

    // Body spots
    put(img, mx - 4, my - 3, SPOT);
    put(img, mx + 3, my - 1, SPOT);

    // Faint kokoro-sac glow
    fill_circle(img, mx, my + 1, 3, GLOW_FAINT);

    // Eyes follow soft body
    let eye_cy = ey_l;
    let eye_cx = (ex_l + (mx + (mx - ex_l).abs())) / 2; // approximate center
    draw_eyes(img, eye_cx, eye_cy, 6, 4, mood, p.eye);
}

// ===================================================================
// YOUNG — tentacles explode in growth, dome shrinks proportionally
// ===================================================================
// Allometric growth: tentacles grow MUCH faster than the dome.
// The creature is transitioning from blob to cephalopod.

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (mx, my) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 16));
    let (mtx, mty) = sb.as_ref().map(|b| b.point("mantle_top").px()).unwrap_or((cx, 4));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("tent_fl").px()).unwrap_or((cx - 4, 22));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("tent_fr").px()).unwrap_or((cx + 4, 22));
    let (bl_x, bl_y) = sb.as_ref().map(|b| b.point("tent_bl").px()).unwrap_or((cx - 11, 22));
    let (br_x, br_y) = sb.as_ref().map(|b| b.point("tent_br").px()).unwrap_or((cx + 11, 22));
    let (tfl_x, tfl_y) = sb.as_ref().map(|b| b.point("tip_fl").px()).unwrap_or((cx - 4, 42));
    let (tfr_x, tfr_y) = sb.as_ref().map(|b| b.point("tip_fr").px()).unwrap_or((cx + 4, 42));
    let (tbl_x, tbl_y) = sb.as_ref().map(|b| b.point("tip_bl").px()).unwrap_or((cx - 11, 39));
    let (tbr_x, tbr_y) = sb.as_ref().map(|b| b.point("tip_br").px()).unwrap_or((cx + 11, 39));
    let (ex_l, ey_l) = sb.as_ref().map(|b| b.point("eye_l").px()).unwrap_or((cx - 4, 16));
    let mr = 12;

    // Mantle cap (on top, moves with mantle_top)
    fill_circle(img, mtx, mty + 4, 8, p.accent);
    for dx in -5..=5 {
        put(img, mtx + dx, mty - 6, GLOW_FAINT);
    }

    // Tentacles — root → tip rectangles
    fill_rect(img, bl_x - 1, bl_y, 3, (tbl_y - bl_y).max(2), p.accent);
    fill_rect(img, fl_x - 1, fl_y, 3, (tfl_y - fl_y).max(2), p.accent);
    fill_rect(img, fr_x - 1, fr_y, 3, (tfr_y - fr_y).max(2), p.accent);
    fill_rect(img, br_x - 1, br_y, 3, (tbr_y - br_y).max(2), p.accent);
    // Glow tips
    fill_rect(img, tbl_x - 1, tbl_y - 2, 3, 2, GLOW_DIM);
    fill_rect(img, tfl_x - 1, tfl_y - 2, 3, 2, GLOW_DIM);
    fill_rect(img, tfr_x - 1, tfr_y - 2, 3, 2, GLOW_DIM);
    fill_rect(img, tbr_x - 1, tbr_y - 2, 3, 2, GLOW_DIM);

    // Body dome
    fill_circle(img, mx, my, mr, p.body);
    for &(dx, dy) in &[(-4,-3), (3,-1), (-1,2), (5,0)] {
        put(img, mx + dx, my + dy, SPOT);
    }
    fill_circle(img, mx, my + 3, 7, p.body_light);

    // Kokoro-sac (brighter)
    fill_circle(img, mx, my + 2, 4, GLOW_DIM);

    // Eyes follow soft body
    let eye_cy = ey_l;
    let eye_cx = (ex_l + (mx * 2 - ex_l)) / 2;
    draw_eyes(img, eye_cx, eye_cy, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, eye_cx - 8, eye_cy + 2, GLOW_FAINT);
        put(img, eye_cx + 7, eye_cy + 2, GLOW_FAINT);
    }
}

// ===================================================================
// ADULT — tentacle-dominant, rich bioluminescence, side fins
// ===================================================================
// Complete transformation: tentacles are now the defining feature.
// Dome is proportionally small. Rich chromatophore patterns. Side fins.

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let (_, my) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 14));
    let mr = 11;
    let (mt_x, mt_y) = sb.as_ref().map(|b| b.point("mantle_top").px()).unwrap_or((cx, 4));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("tent_fl").px()).unwrap_or((cx - 5, 22));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("tent_fr").px()).unwrap_or((cx + 5, 22));
    let (bl_x, bl_y) = sb.as_ref().map(|b| b.point("tent_bl").px()).unwrap_or((cx - 14, 22));
    let (br_x, br_y) = sb.as_ref().map(|b| b.point("tent_br").px()).unwrap_or((cx + 14, 22));
    let (tfl_x, tfl_y) = sb.as_ref().map(|b| b.point("tip_fl").px()).unwrap_or((cx - 5, 48));
    let (tfr_x, tfr_y) = sb.as_ref().map(|b| b.point("tip_fr").px()).unwrap_or((cx + 5, 48));
    let (tbl_x, tbl_y) = sb.as_ref().map(|b| b.point("tip_bl").px()).unwrap_or((cx - 14, 44));
    let (tbr_x, tbr_y) = sb.as_ref().map(|b| b.point("tip_br").px()).unwrap_or((cx + 14, 44));
    let (finl_x, _) = sb.as_ref().map(|b| b.point("fin_l").px()).unwrap_or((cx - 12, 12));
    let (finr_x, _) = sb.as_ref().map(|b| b.point("fin_r").px()).unwrap_or((cx + 12, 12));

    // Mantle cap (moves with mantle_top)
    fill_circle(img, mt_x, mt_y + 4, 10, p.accent);
    for dx in -8..=8 {
        put(img, mt_x + dx, mt_y, GLOW_BRIGHT);
    }
    put(img, mt_x - 4, mt_y + 2, GLOW_DIM);
    put(img, mt_x + 3, mt_y + 4, GLOW_DIM);

    // Tentacles — each pair: root → tip, drawn as rects between soft body positions
    // Inner pair (front)
    draw_tentacle(img, fl_x, fl_y, tfl_x, tfl_y, 4, p.accent, GLOW_BRIGHT, true);
    draw_tentacle(img, fr_x, fr_y, tfr_x, tfr_y, 4, p.accent, GLOW_BRIGHT, true);
    // Outer pair (back, shorter)
    draw_tentacle(img, bl_x, bl_y, tbl_x, tbl_y, 4, p.accent, GLOW_BRIGHT, false);
    draw_tentacle(img, br_x, br_y, tbr_x, tbr_y, 4, p.accent, GLOW_BRIGHT, false);

    // Side fins (flutter with soft body)
    for i in 0..4 {
        put(img, finl_x, my - 2 + i * 2, fade(p.accent, 0.3));
        put(img, finr_x, my - 2 + i * 2, fade(p.accent, 0.3));
    }

    // Body dome
    fill_circle(img, cx, my, mr, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6), (-4,5), (7,-3)] {
        put(img, cx + dx, my + dy, SPOT);
    }
    fill_circle(img, cx, my + 3, 7, p.body_light);

    // Kokoro-sac
    fill_circle(img, cx, my + 2, 5, GLOW_BRIGHT);
    fill_circle(img, cx, my + 2, 3, GLOW_DIM);

    // Eyes
    draw_eyes(img, cx, my + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 9, my + 2, GLOW_DIM);
        put(img, cx - 9, my + 3, GLOW_DIM);
        put(img, cx + 8, my + 2, GLOW_DIM);
        put(img, cx + 8, my + 3, GLOW_DIM);
    }

    // Happy: pulsing glow
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 3, my - 2, GLOW_BRIGHT);
        put(img, cx + 2, my + 1, GLOW_BRIGHT);
        put(img, cx, my + 5, GLOW_BRIGHT);
    }
}

/// Draws a tentacle from root to tip using a simple line of rects.
fn draw_tentacle(img: &mut RgbaImage, rx: i32, ry: i32, tx: i32, ty: i32, w: i32, color: Rgba<u8>, glow: Rgba<u8>, has_suckers: bool) {
    let dx = tx - rx;
    let dy = ty - ry;
    let len = ((dx * dx + dy * dy) as f32).sqrt().max(1.0);
    let steps = (len / 3.0) as i32;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = rx + (dx as f32 * t) as i32;
        let y = ry + (dy as f32 * t) as i32;
        // Tentacle gets thinner toward the tip
        let seg_w = (w as f32 * (1.0 - t * 0.5)) as i32;
        fill_rect(img, x - seg_w / 2, y, seg_w.max(1), 3, color);

        // Sucker dots on inner tentacles
        if has_suckers && i % 2 == 0 && i > 0 && i < steps {
            put(img, x, y + 1, super::NEAR_BLACK_PX);
        }
    }

    // Glow tip
    fill_rect(img, tx - 1, ty - 1, 3, 3, glow);
}

// ===================================================================
// ELDER overlay
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let my = 14;
    let dim = Rgba([140, 130, 115, 200]);
    let ring = Rgba([120, 100, 150, 255]);

    // Dimmer glow tips (overwrite bright with faded)
    let tent_y = my + 8;
    fill_rect(img, cx - 14, tent_y + 16, 4, 3, dim);
    fill_rect(img, cx - 5,  tent_y + 26, 4, 3, dim);
    fill_rect(img, cx + 2,  tent_y + 26, 4, 3, dim);
    fill_rect(img, cx + 11, tent_y + 16, 4, 3, dim);
    // Faded mantle glow
    for dx in -6..=6 {
        put(img, cx + dx, my - 13, dim);
    }
    // Wisdom rings on mantle (concentric age marks)
    put(img, cx - 2, my - 7, ring);
    put(img, cx + 1, my - 5, ring);
    put(img, cx, my - 9, ring);
    put(img, cx - 3, my - 3, ring);
    put(img, cx + 3, my - 4, ring);
}
