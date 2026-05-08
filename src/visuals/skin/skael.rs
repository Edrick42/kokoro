//! Skael skin — emerald cave reptile (komodo dragon / pangolin).
//!
//! Structural evolution:
//! - Cub:   smooth-skinned, vertical/upright, no armor, no horns — agile tree-climber
//! - Young: thickening, first scale plates on back, horn nubs, tail growing, posture shifting
//! - Adult: horizontal tank, full osteoderm armor, dorsal ridge, massive tail, thick legs
//! - Elder: chipped armor, battle scars, worn horns

use image::{RgbaImage, Rgba};
use bevy::prelude::Res;
use kokoro_art_palette::Palette;
use kokoro_art_palette::dsl::{TaperedTail, BioluminescentSpeck, BumpyDome};
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{SpeciesSkin, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

// Master palette anchors (docs/aesthetic-targets.md §3 + §4.3)
#[allow(dead_code)]
const HIGHLIGHT:        Rgba<u8> = Rgba(Palette::OffWhite.rgba(200));   // eye glint
const CLAW:             Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
const HORN:             Rgba<u8> = Rgba(Palette::Brown.rgba(255));
const SCALE_LIGHT:      Rgba<u8> = Rgba(Palette::CyanBright.rgba(255)); // scale crests
const PUPIL:            Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const RESONANCE:        Rgba<u8> = Rgba(Palette::Teal.rgba(70));
const RESONANCE_BRIGHT: Rgba<u8> = Rgba(Palette::CyanBright.rgba(110));
// Blush ramps with kawaii_factor (§4b/§4c): full Cub, subtle Young, absent past.
const BLUSH:            Rgba<u8> = Rgba(Palette::CoralPink.rgba(255));
const BLUSH_SUBTLE:     Rgba<u8> = Rgba(Palette::CoralPink.rgba(128));

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32) {
    let cy = 30;
    fill_ellipse(img, cx, cy, 12, 16, p.egg);
    for i in 0..6 {
        let y = cy - 10 + i * 4;
        put(img, cx - 4 + i, y, p.egg_spot);
        put(img, cx - 3 + i, y, p.egg_spot);
        put(img, cx + 3 - i, y + 1, p.egg_spot);
    }
    put(img, cx - 5, cy - 4, SCALE_LIGHT);
    put(img, cx + 4, cy + 3, SCALE_LIGHT);
}

// ===================================================================
// CUB — smooth, vertical, agile: looks like a different species
// ===================================================================
// Based on juvenile Komodo dragons that live in trees to avoid adults.
// Upright posture, smooth skin (no armor), thin limbs, big head, long thin tail.

pub fn draw_cub(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    // VERTICAL posture — tall, thin (opposite of adult's horizontal bulk)
    let hy = 16;    // head center
    let hr = 12;    // head (big for cub)
    let body_y = 32; // body below
    let body_rx = 8; // narrow body (vertical/upright)
    let body_ry = 10;

    // Thin tail (whip-like, not muscular) — DSL TaperedTail gives a clean
    // taper from base to a single-pixel tip with the species accent.
    let tail_base = (cx, body_y + body_ry);
    let tail_tip = (cx, body_y + body_ry + 11);
    TaperedTail::new(tail_base, tail_tip, 1, Palette::Teal)
        .paint_with(img, p.body, None);
    // Whip end accent (single-pixel tip in scale-light tone)
    put(img, cx, body_y + body_ry + 11, p.accent);

    // Smooth body (NO scales — skin is soft at this age) — BumpyDome with
    // independent height now that the DSL supports ellipses. Vertical/upright
    // posture per the species notes; very low bumpiness so the skin still
    // reads as smooth juvenile, just slightly organic.
    BumpyDome::new(cx, body_y, body_rx as u32, Palette::Teal)
        .with_height(body_ry as u32)
        .with_bumpiness(0.12)
        .with_bumps(8)
        .with_seed(23)
        .paint_with(img, p.body, None);
    fill_ellipse(img, cx, body_y + 2, 5, 6, p.body_light);

    // Thin limbs (tree-climbing limbs — long, light)
    fill_rect(img, cx - body_rx - 1, body_y - 2, 3, 7, p.body);
    fill_rect(img, cx + body_rx - 1, body_y - 2, 3, 7, p.body);
    // Small clawed feet
    fill_rect(img, cx - 4, body_y + body_ry - 2, 3, 4, p.body);
    fill_rect(img, cx + 2, body_y + body_ry - 2, 3, 4, p.body);

    // Neck (thin)
    fill_rect(img, cx - 3, hy + hr - 2, 7, 5, p.body);

    // Head (round, smooth — no horns!)
    fill_circle(img, cx, hy, hr, p.body);

    // Kokoro-sac glow
    fill_circle(img, cx, body_y + 1, 3, RESONANCE);

    // Big golden eyes with slit pupil
    draw_eyes(img, cx, hy + 1, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, hy + 2, PUPIL);
        put(img, cx - 5, hy + 3, PUPIL);
        put(img, cx + 4, hy + 2, PUPIL);
        put(img, cx + 4, hy + 3, PUPIL);
    }

    // Cub blush
    fill_rect(img, cx - 8, hy + 5, 2, 2, BLUSH);
    fill_rect(img, cx + 6, hy + 5, 2, 2, BLUSH);

    // Small snout
    fill_rect(img, cx - 1, hy + 8, 3, 2, p.mouth);
}

// ===================================================================
// YOUNG — transitioning: thickening, first armor, horn nubs
// ===================================================================
// Getting heavier. Can't climb trees anymore. First scale plates on back.
// Small horn nubs. Tail thickening. Posture tilting toward horizontal.

pub fn draw_young(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 16));
    let (bx, body_y) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 30));
    let (hl_x, hl_y) = sb.as_ref().map(|b| b.point("horn_l").px()).unwrap_or((cx - 5, 8));
    let (hr_x, hr_y) = sb.as_ref().map(|b| b.point("horn_r").px()).unwrap_or((cx + 5, 8));
    let (ll_x, ll_y) = sb.as_ref().map(|b| b.point("leg_l").px()).unwrap_or((cx - 12, 38));
    let (lr_x, lr_y) = sb.as_ref().map(|b| b.point("leg_r").px()).unwrap_or((cx + 12, 38));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 6, 45));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 6, 45));
    let (t1_x, t1_y) = sb.as_ref().map(|b| b.point("tail_1").px()).unwrap_or((cx, 40));
    let (t2_x, t2_y) = sb.as_ref().map(|b| b.point("tail_2").px()).unwrap_or((cx, 47));
    let (_t3_x, t3_y) = sb.as_ref().map(|b| b.point("tail_3").px()).unwrap_or((cx, 52));
    let hr = 11;
    let body_r = 12;

    // Horn nubs (move with horn points) — short TaperedTails pointing up.
    // Base 1 (3px wide) at the head plate, tip 1px ~5px above.
    TaperedTail::new((hl_x, hl_y + 2), (hl_x, hl_y - 3), 1, Palette::Brown)
        .paint_with(img, HORN, None);
    TaperedTail::new((hr_x, hr_y + 2), (hr_x, hr_y - 3), 1, Palette::Brown)
        .paint_with(img, HORN, None);

    // Tail chain (segmented) — two TaperedTails in succession so the chain
    // tapers from a 5px base at the body down through a single-pixel tip.
    TaperedTail::new((t1_x, t1_y), (t2_x, t2_y), 2, Palette::Teal)
        .paint_with(img, p.accent, None);
    TaperedTail::new((t2_x, t2_y), (t2_x, t3_y), 1, Palette::Teal)
        .paint_with(img, p.accent, None);
    put(img, t1_x, t1_y + 1, HORN); // first tail spine

    // Legs
    fill_rect(img, ll_x - 1, body_y + 2, 4, (ll_y - body_y - 2).max(2), p.body);
    fill_rect(img, lr_x - 2, body_y + 2, 4, (lr_y - body_y - 2).max(2), p.body);
    // Feet
    fill_rect(img, fl_x - 2, fl_y, 5, 5, p.body);
    fill_rect(img, fr_x - 2, fr_y, 5, 5, p.body);
    for dx in [-1, 1, 3] {
        put(img, fl_x - 2 + dx, fl_y + 5, CLAW);
        put(img, fr_x - 2 + dx, fr_y + 5, CLAW);
    }

    // Body
    fill_circle(img, bx, body_y, body_r, p.body);
    fill_circle(img, bx, body_y + 3, 8, p.body_light);

    // First scale plates on back — proto-bioluminescent crests. Each plate
    // is a pinpoint Cyan core with a faded TealDark halo so they read as
    // emerging light points, not flat marks. Adult version (§4.3) makes the
    // crests fully bright; here they're early hints.
    let crest_halo = Rgba(Palette::TealDark.rgba(120));
    for &(ox, oy) in &[(-3_i32, -6_i32), (0, -7), (3, -6)] {
        BioluminescentSpeck::new(bx + ox, body_y + oy, Palette::CyanBright)
            .with_core_radius(0)
            .with_halo(Palette::TealDark)
            .paint_with(img, SCALE_LIGHT, Some(crest_halo));
    }
    put(img, bx - 1, body_y - 8, HORN);

    // Neck
    let neck_top = hy.min(body_y);
    let neck_h = (body_y - hy).max(1);
    fill_rect(img, hx - 4, neck_top, 9, neck_h, p.body);

    // Head
    fill_circle(img, hx, hy, hr, p.body);

    // Kokoro-sac
    fill_circle(img, bx, body_y + 2, 4, RESONANCE);

    // Eyes
    draw_eyes(img, hx, hy + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 5, hy + 2, PUPIL);
        put(img, hx + 3, hy + 2, PUPIL);
    }

    // Young blush — sutil
    fill_rect(img, hx - 7, hy + 4, 2, 2, BLUSH_SUBTLE);
    fill_rect(img, hx + 5, hy + 4, 2, 2, BLUSH_SUBTLE);

    // Wider snout
    fill_rect(img, hx - 2, hy + 7, 5, 3, p.mouth);
    put(img, hx - 1, hy + 7, fade(p.mouth, 0.3));
    put(img, hx + 2, hy + 7, fade(p.mouth, 0.3));
}

// ===================================================================
// ADULT — horizontal armored tank: complete transformation
// ===================================================================
// Body is now HORIZONTAL (wider than tall). Full armor plates. Tall horns.
// Massive muscular tail with spines. Dorsal ridge. Thick powerful legs.

pub fn draw_adult(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // HORIZONTAL body plan — wide, low, heavy
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 14));
    let hr = 10;
    let (_, body_y) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 28));
    let body_rx = 16; // WIDE
    let body_ry = 12; // shorter than wide
    let (t1_x, t1_y) = sb.as_ref().map(|b| b.point("tail_1").px()).unwrap_or((cx, 40));
    let (t2_x, t2_y) = sb.as_ref().map(|b| b.point("tail_2").px()).unwrap_or((cx, 47));
    let (t3_x, t3_y) = sb.as_ref().map(|b| b.point("tail_3").px()).unwrap_or((cx, 53));

    // Full horns — tall, layered
    fill_rect(img, cx - 8, hy - hr - 1, 4, 4, HORN);
    fill_rect(img, cx - 7, hy - hr - 4, 2, 3, HORN);
    put(img, cx - 7, hy - hr - 6, fade(HORN, 0.3));
    fill_rect(img, cx + 5, hy - hr - 1, 4, 4, HORN);
    fill_rect(img, cx + 6, hy - hr - 4, 2, 3, HORN);
    put(img, cx + 6, hy - hr - 6, fade(HORN, 0.3));

    // Massive tail with spines — three TaperedTails segmented through the
    // soft-body anchor points so a swaying tail arcs through them. Base
    // half-widths step down (3 → 2 → 1) giving the doc's "muscular near
    // body, fine at tip" silhouette.
    TaperedTail::new((t1_x, t1_y), (t2_x, t2_y), 3, Palette::Teal)
        .paint_with(img, p.accent, None);
    TaperedTail::new((t2_x, t2_y), (t3_x, t3_y), 2, Palette::Teal)
        .paint_with(img, p.accent, None);
    TaperedTail::new((t3_x, t3_y), (t3_x, t3_y + 2), 1, Palette::Teal)
        .paint_with(img, p.accent, None);
    // Tail spines (on each segment)
    put(img, t1_x, t1_y - 3, HORN);
    put(img, t2_x, t2_y - 2, HORN);
    put(img, t3_x, t3_y - 2, HORN);

    // Thick powerful legs
    fill_rect(img, cx - body_rx + 1, body_y + 2, 5, 10, p.body);
    fill_rect(img, cx + body_rx - 5, body_y + 2, 5, 10, p.body);
    fill_rect(img, cx - body_rx + 1, body_y + body_ry - 2, 6, 6, p.body);
    fill_rect(img, cx + body_rx - 6, body_y + body_ry - 2, 6, 6, p.body);
    // Prominent claws
    for dx in [-2, 0, 2, 4] {
        put(img, cx - body_rx + 1 + dx, body_y + body_ry + 4, CLAW);
        put(img, cx + body_rx - 6 + dx, body_y + body_ry + 4, CLAW);
    }

    // Front legs / arms
    fill_rect(img, cx - body_rx, body_y - 3, 4, 7, p.body);
    fill_rect(img, cx + body_rx - 3, body_y - 3, 4, 7, p.body);

    // WIDE body (the tank)
    fill_ellipse(img, cx, body_y, body_rx, body_ry, p.body);

    // Armor plates (osteoderm pattern covering the back) — bioluminescent
    // specks now that the doc (§4.3) calls these out as full-bright crests
    // at adult stage. Pinpoint Cyan core + faded TealDark halo so each
    // plate reads as a small light source on the armor.
    let plate_halo = Rgba(Palette::TealDark.rgba(110));
    for &(dx, dy) in &[(-8_i32, -5_i32), (-4,-7), (0,-8), (4,-7), (8,-5), (-6,-3), (6,-3),
                        (-10,-1), (10,-1), (-4,2), (4,2)] {
        BioluminescentSpeck::new(cx + dx, body_y + dy, Palette::CyanBright)
            .with_core_radius(0)
            .with_halo(Palette::TealDark)
            .paint_with(img, SCALE_LIGHT, Some(plate_halo));
    }

    // Belly (lighter underbelly with scale rows)
    fill_ellipse(img, cx, body_y + 3, 10, 7, p.body_light);
    for row in 0..2 {
        for col in -2..=2 {
            put(img, cx + col * 3, body_y + 1 + row * 3, fade(p.body_light, 0.12));
        }
    }

    // Dorsal ridge (5 spines along back)
    for i in 0..5 {
        let x = cx - 6 + i * 3;
        put(img, x, body_y - body_ry, HORN);
        put(img, x, body_y - body_ry - 1, fade(HORN, 0.4));
    }

    // Neck (thick, connects head to body)
    let neck_cx = (hx + cx) / 2;
    let neck_top = hy.min(body_y);
    let neck_h = (body_y - hy).max(1);
    fill_rect(img, neck_cx - 5, neck_top, 11, neck_h, p.body);

    // Head (armored, moves with soft body)
    fill_circle(img, hx, hy, hr, p.body);
    // Head armor plates
    put(img, hx - 4, hy - 5, SCALE_LIGHT);
    put(img, hx + 3, hy - 5, SCALE_LIGHT);
    put(img, hx, hy - 6, SCALE_LIGHT);

    // Kokoro-sac glow
    fill_circle(img, cx, body_y + 2, 5, RESONANCE_BRIGHT);
    fill_circle(img, cx, body_y + 2, 3, RESONANCE);
    // Resonance along dorsal
    for i in 0..3 {
        put(img, cx - 3 + i * 3, body_y - body_ry + 1, RESONANCE);
    }

    // Golden eyes with slit pupil (on head)
    draw_eyes(img, hx, hy + 1, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 4, hy + 2, PUPIL);
        put(img, hx + 3, hy + 2, PUPIL);
    }

    // Wide snout with nostrils (on head)
    fill_rect(img, hx - 3, hy + 7, 7, 3, p.mouth);
    put(img, hx - 2, hy + 7, fade(p.mouth, 0.3));
    put(img, hx + 3, hy + 7, fade(p.mouth, 0.3));
}

// ===================================================================
// ELDER overlay
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, _p: &SpeciesSkin, cx: i32) {
    let hy = 14;
    let body_y = 28;
    let white = Rgba(Palette::Cream.rgba(255));
    let dim = Rgba(Palette::Teal.rgba(50));

    // Chipped horn tips
    put(img, cx - 7, hy - 16, white);
    put(img, cx + 6, hy - 16, white);
    // Battle scars (lighter patches on armor)
    for &(dx, dy) in &[(-6,20), (5,22), (-4,28), (3,30), (-8,26), (7,24)] {
        put(img, cx + dx, dy, white);
    }
    // Faded dorsal ridge
    for i in 0..5 {
        put(img, cx - 6 + i * 3, body_y - 13, white);
    }
    fill_circle(img, cx, body_y + 2, 4, dim);
}
