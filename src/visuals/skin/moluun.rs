//! Moluun skin — warm forest mammal (koala/wombat/red panda).
//!
//! Structural evolution (pokemon-style body plan change):
//! - Cub:   almost ALL head, tiny body underneath, no arms, stubs for feet
//! - Young: head shrinks relative, body elongates, short arms appear, ears grow out
//! - Adult: body dominates, defined limbs, broad shoulders, full features
//! - Elder: slightly hunched, thinner, fading resonance

use bevy::prelude::Res;
use image::{RgbaImage, Rgba};
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

const HIGHLIGHT: Rgba<u8> = Rgba([255, 255, 255, 200]);
const NOSE_COLOR: Rgba<u8> = Rgba([50, 35, 30, 255]);
const BLUSH: Rgba<u8> = Rgba([220, 150, 140, 255]);
const EAR_INNER: Rgba<u8> = Rgba([200, 160, 155, 255]);
const RESONANCE: Rgba<u8> = Rgba([160, 210, 230, 80]);
const RESONANCE_BRIGHT: Rgba<u8> = Rgba([170, 220, 240, 120]);
const EAR_GLOW: Rgba<u8> = Rgba([140, 200, 220, 100]);

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    fill_ellipse(img, cx, cy, 13, 17, p.egg);
    fill_ellipse(img, cx, cy + 8, 10, 5, fade(p.egg, 0.15));
    fill_circle(img, cx - 5, cy - 6, 3, p.egg_spot);
    fill_circle(img, cx + 6, cy - 2, 3, p.egg_spot);
    fill_circle(img, cx - 2, cy + 5, 2, p.egg_spot);
    fill_circle(img, cx + 3, cy + 8, 2, p.egg_spot);
    put(img, cx + 1, cy - 12, fade(p.egg, 0.3));
    put(img, cx + 2, cy - 11, fade(p.egg, 0.3));
}

// ===================================================================
// CUB — 80% head, tiny body, stubs, no arms
// ===================================================================
// Think Togepi / baby Kirby — a round head with a small body attached below.
// Head radius ~16px, body just a small bump underneath (~8px).
// Ears tiny. Feet are round stubs. No arms at all.

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions (cub uses head and feet)
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 22));
    let (bx, body_y) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 42));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 3, 45));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 3, 45));
    let hr = 16;

    // === DRAW ORDER: feet → body → neck → head LAST so head silhouette is intact ===

    // Tiny stub feet (drawn BEFORE body so body+head can overlap them subtly)
    fill_circle(img, fl_x, fl_y, 3, p.body);
    fill_circle(img, fr_x, fr_y, 3, p.body);

    // Small body ellipse underneath the head
    fill_ellipse(img, bx, body_y, 10, 7, p.body);
    fill_ellipse(img, bx, body_y + 1, 7, 5, p.body_light);

    // Kokoro-sac glow on body
    fill_circle(img, bx, body_y, 3, RESONANCE);

    // Neck — bridges head bottom to body center
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy.min(body_y);
    let neck_h = (hy - body_y).unsigned_abs() as i32 + 1;
    fill_rect(img, neck_cx - 5, neck_top, 11, neck_h, p.body);

    // Tiny ears (drawn BEFORE head so head silhouette covers their inner edge cleanly)
    fill_circle(img, hx - 10, hy - 12, 4, p.accent);
    fill_circle(img, hx + 10, hy - 12, 4, p.accent);
    fill_circle(img, hx - 10, hy - 12, 2, EAR_INNER);
    fill_circle(img, hx + 10, hy - 12, 2, EAR_INNER);

    // BIG round head ON TOP of everything else — head IS the creature for cub
    fill_circle(img, hx, hy, hr, p.body);

    // Soft fur on head (drawn AFTER head so dots show on the head)
    for &(dx, dy) in &[(-7,-5), (7,-4), (-3,-10), (6,7)] {
        put(img, hx + dx, hy + dy, p.accent);
    }

    // === FACE (on head) ===

    // HUGE eyes on the big head
    draw_eyes(img, hx, hy + 2, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 6, hy + 2, HIGHLIGHT);
        put(img, hx - 5, hy + 2, HIGHLIGHT);
        put(img, hx + 2, hy + 2, HIGHLIGHT);
        put(img, hx + 3, hy + 2, HIGHLIGHT);
    }

    // Big blush
    fill_rect(img, hx - 12, hy + 6, 3, 2, BLUSH);
    fill_rect(img, hx + 10, hy + 6, 3, 2, BLUSH);

    // Tiny nose
    put(img, hx, hy + 10, NOSE_COLOR);
    put(img, hx + 1, hy + 10, NOSE_COLOR);
}

// ===================================================================
// YOUNG — head shrinks relative, body grows, arms appear
// ===================================================================
// Head and body are now ~equal size. The creature is elongating.
// Short arms sprout. Ears grow noticeably. Feet get bigger.
// Think Charmeleon — awkward middle stage, clearly transitioning.

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 18));
    let (bx, by) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 32));
    let (sl_x, sl_y) = sb.as_ref().map(|b| b.point("shoulder_l").px()).unwrap_or((cx - 13, 29));
    let (sr_x, sr_y) = sb.as_ref().map(|b| b.point("shoulder_r").px()).unwrap_or((cx + 13, 29));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 6, 43));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 6, 43));
    let hr = 13;
    let body_r = 12;

    // Ears (on head, soft body position)
    fill_circle(img, hx - 10, hy - 10, 5, p.accent);
    fill_circle(img, hx + 10, hy - 10, 5, p.accent);
    fill_circle(img, hx - 10, hy - 10, 3, EAR_INNER);
    fill_circle(img, hx + 10, hy - 10, 3, EAR_INNER);
    put(img, hx - 12, hy - 13, EAR_GLOW);
    put(img, hx + 12, hy - 13, EAR_GLOW);

    // Head (soft body position)
    fill_circle(img, hx, hy, hr, p.body);
    for &(dx, dy) in &[(-6,-4), (5,-3), (-3,-8)] {
        put(img, hx + dx, hy + dy, p.accent);
    }

    // Neck — filled zone between head and body centers (NEVER gaps)
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy.min(by);
    let neck_bottom = hy.max(by);
    let neck_height = (neck_bottom - neck_top).max(1);
    fill_rect(img, neck_cx - 5, neck_top, 11, neck_height, p.body);

    // Body (soft body position)
    fill_circle(img, bx, by, body_r, p.body);
    for &(dx, dy) in &[(-5,-3), (4,2), (-3,5), (6,-1), (-7,1)] {
        put(img, bx + dx, by + dy, p.accent);
    }
    fill_circle(img, bx, by + 2, 8, p.body_light);

    // Kokoro-sac
    fill_circle(img, bx, by + 1, 4, RESONANCE);

    // Arms (soft body positions)
    fill_rect(img, sl_x, sl_y, 4, 6, p.body);
    fill_rect(img, sr_x - 3, sr_y, 4, 6, p.body);

    // Feet (soft body positions)
    fill_circle(img, fl_x, fl_y, 4, p.body);
    fill_circle(img, fr_x, fr_y, 4, p.body);
    for dx in [-1, 1] {
        put(img, fl_x + dx, fl_y + 3, p.accent);
        put(img, fr_x + dx, fr_y + 3, p.accent);
    }

    // Face (on head)
    draw_eyes(img, hx, hy + 2, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 5, hy + 2, HIGHLIGHT);
        put(img, hx + 2, hy + 2, HIGHLIGHT);
    }

    fill_rect(img, hx - 10, hy + 5, 2, 2, BLUSH);
    fill_rect(img, hx + 9, hy + 5, 2, 2, BLUSH);

    fill_rect(img, hx - 1, hy + 8, 3, 2, NOSE_COLOR);
    put(img, hx, hy + 10, NOSE_COLOR);

    // Mouth (mood-reactive)
}

// ===================================================================
// ADULT — body dominates, full limbs, broad build
// ===================================================================
// Body is now clearly larger than head. Defined neck/shoulders.
// Full arms with paws. Thick legs with toe pads. Dense fur everywhere.
// Think Charizard-level transformation — powerful, mature, complete.

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // === SOFT BODY POSITIONS ===
    // All positions come from physics simulation. Fallback to defaults if no soft body.
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 14));
    let (bx, by) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 30));
    let (belly_x, belly_y) = sb.as_ref().map(|b| b.point("belly").px()).unwrap_or((cx, 33));
    let (sl_x, sl_y) = sb.as_ref().map(|b| b.point("shoulder_l").px()).unwrap_or((cx - 17, 26));
    let (pl_x, pl_y) = sb.as_ref().map(|b| b.point("paw_l").px()).unwrap_or((cx - 17, 36));
    let (sr_x, sr_y) = sb.as_ref().map(|b| b.point("shoulder_r").px()).unwrap_or((cx + 17, 26));
    let (pr_x, pr_y) = sb.as_ref().map(|b| b.point("paw_r").px()).unwrap_or((cx + 17, 36));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 8, 46));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 8, 46));
    let (ear_x, ear_y) = sb.as_ref().map(|b| b.point("ear_anchor").px()).unwrap_or((cx, 6));

    let body_r = 13;
    let hr = 11;

    // === DRAW ORDER: body first (bottom), then neck, then head on top ===

    // Body FIRST (pinned anchor — always in position)
    fill_circle(img, bx, by, body_r, p.body);

    // Neck — thick rectangle connecting body center to head center.
    // Drawn OVER body, UNDER head. ALWAYS visible, NEVER a gap.
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy;               // from head center
    let neck_bottom = by;            // to body center
    let neck_y = neck_top.min(neck_bottom);
    let neck_h = (neck_top - neck_bottom).unsigned_abs() as i32 + 1;
    fill_rect(img, neck_cx - 7, neck_y, 15, neck_h, p.body);

    // Head ON TOP of neck (so it covers the joint)
    fill_circle(img, hx, hy, hr, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-3), (-2,-7), (3,5)] {
        put(img, hx + dx, hy + dy, p.accent);
    }

    // Ears (on head)
    fill_circle(img, ear_x - 10, ear_y, 7, p.accent);
    fill_circle(img, ear_x + 10, ear_y, 7, p.accent);
    fill_circle(img, ear_x - 10, ear_y, 4, EAR_INNER);
    fill_circle(img, ear_x + 10, ear_y, 4, EAR_INNER);
    put(img, ear_x - 12, ear_y - 5, EAR_GLOW);
    put(img, ear_x + 12, ear_y - 5, EAR_GLOW);
    // Body fur dots — only those clearly OUTSIDE the head circle, so the head
    // never gets accent-color pixels splashed on top of it.
    for &(dx, dy) in &[(-8,-6), (7,-4), (-5,3), (9,1), (-3,-10), (6,7), (-10,4), (4,-7), (-6,9)] {
        let px = bx + dx;
        let py = by + dy;
        let dist_sq = (px - hx).pow(2) + (py - hy).pow(2);
        if dist_sq > hr * hr {
            put(img, px, py, p.accent);
        }
    }

    // Belly (oscillates with breathing via soft body)
    fill_circle(img, belly_x, belly_y, 9, p.body_light);
    for &(dx, dy) in &[(-3, 2), (4, 3), (-1, 5), (2, 7)] {
        put(img, belly_x + dx, belly_y + dy, fade(p.body_light, 0.1));
    }

    // Kokoro-sac glow (on belly)
    fill_circle(img, belly_x, belly_y - 1, 5, RESONANCE_BRIGHT);
    fill_circle(img, belly_x, belly_y - 1, 3, RESONANCE);

    // Left arm (shoulder → paw, soft body positions)
    let al_h = (pl_y - sl_y).max(3);
    fill_rect(img, sl_x, sl_y, 5, al_h, p.body);
    fill_circle(img, pl_x, pl_y, 3, p.accent);

    // Right arm
    let ar_h = (pr_y - sr_y).max(3);
    fill_rect(img, sr_x - 4, sr_y, 5, ar_h, p.body);
    fill_circle(img, pr_x, pr_y, 3, p.accent);

    // Left foot (soft body position)
    fill_circle(img, fl_x, fl_y, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, fl_x + dx, fl_y + 5, p.accent);
    }

    // Right foot
    fill_circle(img, fr_x, fr_y, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, cx + 8 + dx, fr_y + 5, p.accent);
    }

    // === FACE (on head, moves with head) ===

    // Eyes
    draw_eyes(img, hx, hy + 2, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 4, hy + 2, HIGHLIGHT);
        put(img, hx + 2, hy + 2, HIGHLIGHT);
        put(img, hx - 7, hy + 3, RESONANCE);
        put(img, hx + 7, hy + 3, RESONANCE);
    }

    // Blush (on head)
    fill_rect(img, hx - 8, hy + 5, 2, 2, BLUSH);
    fill_rect(img, hx + 7, hy + 5, 2, 2, BLUSH);

    // Nose (on head)
    fill_rect(img, hx - 2, hy + 8, 5, 2, NOSE_COLOR);
    fill_rect(img, hx - 1, hy + 10, 3, 1, NOSE_COLOR);
    put(img, hx, hy + 11, NOSE_COLOR);

    // Mouth drawn centrally in draw_creature
}

// ===================================================================
// ELDER overlay — fading, hunched, wise
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let hy = 14; // same as adult head
    let body_y = 30;
    let white = Rgba([230, 225, 215, 255]);
    let thin = Rgba([200, 170, 165, 150]);
    let dim_res = Rgba([140, 180, 200, 50]);

    // Gray ear tips
    fill_circle(img, cx - 12, hy - 13, 3, white);
    fill_circle(img, cx + 12, hy - 13, 3, white);

    // Wisdom marks on forehead
    put(img, cx - 2, hy - 5, white);
    put(img, cx + 1, hy - 6, white);
    put(img, cx + 3, hy - 4, white);

    // White whisker dots
    put(img, cx - 8, hy + 4, white);
    put(img, cx - 9, hy + 6, white);
    put(img, cx + 8, hy + 4, white);
    put(img, cx + 9, hy + 6, white);

    // Thinning fur patches
    put(img, cx - 7, body_y - 5, thin);
    put(img, cx + 6, body_y - 3, thin);
    put(img, cx - 10, body_y + 3, thin);
    put(img, cx + 9, body_y + 5, thin);

    // Fading kokoro-sac
    fill_circle(img, cx, body_y + 2, 4, dim_res);
    put(img, cx - 13, body_y, dim_res);
    put(img, cx + 13, body_y, dim_res);
}
